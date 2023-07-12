use eframe::{egui, egui_wgpu, wgpu};
use nalgebra::{Matrix4, Vector3};
use std::{collections::HashSet, sync::Arc};

pub mod object;
pub mod tool;
pub mod toolpath;

pub mod ray;

pub mod state;

pub mod grid;

pub mod camera;

pub enum Entity {
    Object(object::Object),
    Toolpath(toolpath::Toolpath),
}

impl Entity {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut state::State,
        messages: &mut Vec<state::Message>,
    ) {
        match self {
            Entity::Object(v) => v.ui(ui, state, messages),
            Entity::Toolpath(v) => v.ui(ui, state, messages),
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            Entity::Object(v) => v.id,
            Entity::Toolpath(v) => v.id,
        }
    }

    pub fn set_id(&mut self, id: u32) {
        match self {
            Entity::Object(v) => v.id = id,
            Entity::Toolpath(v) => v.id = id,
        }
    }
}

const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;

#[derive(Default)]
pub struct Editor {
    camera: camera::Camera,

    pub id_counter: u32,

    pub objects: Vec<object::Object>,
    pub object_changed: bool,
    pub object_verticies: u32,
}

impl Editor {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        let device = &wgpu_render_state.device;

        let camera = camera::Camera::default();
        let camera_uniform = camera::Uniform::new(device, camera.uniform());
        let grid_renderer = grid::Renderer::new(
            device,
            wgpu_render_state.target_format,
            &camera_uniform.bind_group_layout,
        );

        let object_renderer = object::Renderer::new(
            device,
            wgpu_render_state.target_format,
            &camera_uniform.bind_group_layout,
        );

        let arrow_renderer = tool::Renderer::new(
            device,
            wgpu_render_state.target_format,
            &camera_uniform.bind_group_layout,
        );

        wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(Renderer {
                grid_renderer,
                object_renderer,
                arrow_renderer,
                camera_uniform,
            });

        Some(Self {
            camera,
            ..Default::default()
        })
    }

    pub fn remove(&mut self, id: u32) {
        let index = self.objects.iter().position(|x| x.id == id).unwrap();
        self.objects.remove(index);
    }

    pub fn inf_sup(&self, objects: &HashSet<u32>) -> (Vector3<f32>, Vector3<f32>) {
        let mut inf = Vector3::from_element(std::f32::INFINITY);
        let mut sup = Vector3::from_element(std::f32::NEG_INFINITY);
        for object in self.objects.iter().filter(|o| objects.contains(&o.id)) {
            let (oinf, osup) = object.inf_sup();

            inf = inf.inf(&oinf);
            sup = sup.sup(&osup);
        }

        (inf, sup)
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, state: &mut state::State) {
        let available_size = ui.available_size();

        self.camera.resize(available_size.x, available_size.y);

        let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::drag());

        // Rotation
        if response.dragged_by(egui::PointerButton::Secondary) {
            self.camera.yaw += response.drag_delta().x * 0.005;
            self.camera.pitch += response.drag_delta().y * -0.005;

            if self.camera.pitch < -SAFE_FRAC_PI_2 {
                self.camera.pitch = -SAFE_FRAC_PI_2;
            } else if self.camera.pitch > SAFE_FRAC_PI_2 {
                self.camera.pitch = SAFE_FRAC_PI_2;
            }
        }

        // Translation
        if response.dragged_by(egui::PointerButton::Middle) {
            let delta = Matrix4::from_euler_angles(self.camera.pitch, self.camera.yaw, 0.0)
                .transform_vector(&Vector3::new(
                    response.drag_delta().x * 0.001 / self.camera.zoom,
                    response.drag_delta().y * 0.001 / self.camera.zoom,
                    0.0,
                ));

            self.camera.position += delta;
        }

        // Zoom
        if ui.rect_contains_pointer(rect) {
            ui.ctx().input(|i| {
                for event in &i.events {
                    if let egui::Event::Scroll(v) = event {
                        if v[0] == 0.0 {
                            if v[1] > 0.0 {
                                //self.camera.zoom += 0.001 * v[1];
                                self.camera.zoom *= 1.0 + 0.001 * v[1];
                            } else if v[1] < 0.0 {
                                //self.camera.zoom += 0.001 * v[1];
                                self.camera.zoom /= 1.0 + 0.001 * -v[1];
                            }
                        }
                    }
                }
            });
        }

        if response.clicked() {
            let pos = response.interact_pointer_pos().unwrap() - response.rect.left_top();
            let camera_ray = self.camera.screen_ray(pos.x, pos.y);

            // Find the closest intersection point
            let mut intersection_id = 0;
            let mut intersection_dist = std::f32::MAX;

            for object in self.objects.iter() {
                for triangle in object.triangles.iter() {
                    if (camera_ray.origin - triangle.v1).dot(&triangle.normal) < 0.0 {
                        continue;
                    }

                    if let Some(point) = camera_ray.triangle_intersect(
                        &triangle.v1,
                        &triangle.v2,
                        &triangle.v3,
                        &triangle.normal,
                    ) {
                        // It is not important to calculate the exact distance as we only want
                        // to compare the distance with other intersection points
                        let dist = (camera_ray.origin - point).magnitude_squared();
                        if dist < intersection_dist {
                            intersection_dist = dist;
                            intersection_id = object.id;
                        }
                    }
                }
            }

            if intersection_id != 0 {
                if !ui.input(|i| i.modifiers.contains(egui::Modifiers::SHIFT)) {
                    state.selected.clear();
                }

                if state.selected.contains(&intersection_id) {
                    state.selected.remove(&intersection_id);
                } else {
                    state.selected.insert(intersection_id);
                }
            }
        }

        let uniform = self.camera.uniform();

        let mut action = None;
        if let Some(hover_pos) = response.hover_pos() {
            if !state.selected.is_empty() {
                let pos = hover_pos - response.rect.left_top();
                let camera_ray = self.camera.screen_ray(pos.x, pos.y);

                let (inf, sup) = self.inf_sup(&state.selected);
                let origin = (sup - inf).scale(0.5) + inf;

                if let Some(axis) =
                    (tool::Tool::Move { origin }.intersect(&camera_ray, 0.02 / self.camera.zoom))
                {
                    action = Some(tool::Action::Hover { axis });
                }
            }
        }

        // Generate arrow verticies
        let arrow_vertex_data = {
            let mut verticies = Vec::new();

            if !state.selected.is_empty() {
                let (inf, sup) = self.inf_sup(&state.selected);
                let origin = (sup - inf).scale(0.5) + inf;

                verticies.append(
                    &mut tool::Tool::Move { origin }.verticies(0.02 / self.camera.zoom, &action),
                );
            }

            verticies
        };

        // Generate object verticies
        let object_vertex_data = {
            let mut verticies = Vec::new();
            for object in self.objects.iter_mut() {
                if state.selected.contains(&object.id) {
                    object.color = [1.0, 1.0, 0.0];
                } else {
                    object.color = [0.7, 0.7, 0.7];
                }
                verticies.append(&mut object.verticies());
            }

            self.object_verticies = verticies.len() as u32;
            verticies
        };

        let object_verticies = self.object_verticies;
        let arrow_verticies = arrow_vertex_data.len() as u32;

        let cb = egui_wgpu::CallbackFn::new()
            .prepare(move |_device, queue, _encoder, paint_callback_resources| {
                let renderer: &Renderer = paint_callback_resources.get().unwrap();

                renderer.camera_uniform.update(queue, uniform);

                // Update object vertex buffer
                queue.write_buffer(
                    &renderer.object_renderer.vertex_buffer,
                    0,
                    bytemuck::cast_slice(object_vertex_data.as_slice()),
                );

                // Update arrow vertex buffer
                queue.write_buffer(
                    &renderer.arrow_renderer.vertex_buffer,
                    0,
                    bytemuck::cast_slice(arrow_vertex_data.as_slice()),
                );

                Vec::new()
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let renderer: &Renderer = paint_callback_resources.get().unwrap();
                renderer.render(render_pass, object_verticies, arrow_verticies);
            });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };

        ui.painter().add(callback);
    }

    pub fn sidebar(&mut self, state: &mut state::State, ui: &mut egui::Ui) {
        let mut messages = Vec::new();
        for object in self.objects.iter_mut() {
            object.ui(ui, state, &mut messages);
        }

        for message in messages.iter() {
            message.process(self, state);
        }
    }

    pub fn uid(&mut self) -> u32 {
        self.id_counter += 1;
        self.id_counter
    }
}

pub struct Renderer {
    camera_uniform: camera::Uniform,
    grid_renderer: grid::Renderer,
    object_renderer: object::Renderer,
    arrow_renderer: tool::Renderer,
}

impl Renderer {
    fn render<'rp>(
        &'rp self,
        render_pass: &mut wgpu::RenderPass<'rp>,
        object_verticies: u32,
        arrow_verticies: u32,
    ) {
        render_pass.set_bind_group(0, &self.camera_uniform.bind_group, &[]);
        self.grid_renderer.render(render_pass);
        if object_verticies != 0 {
            self.object_renderer.render(render_pass, object_verticies);
        }
        if arrow_verticies != 0 {
            self.arrow_renderer.render(render_pass, arrow_verticies);
        }
    }
}
