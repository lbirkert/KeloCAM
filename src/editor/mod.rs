use eframe::{egui, egui_wgpu, wgpu};
use nalgebra::{Matrix4, Vector3};
use std::sync::Arc;

pub mod arrow;
pub mod object;
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

    pub entities: Vec<Entity>,
    pub id_counter: u32,

    pub object_changed: bool,
    pub object_verticies: u32,

    pub arrows: Vec<arrow::Arrow>,
    pub arrow_changed: bool,
    pub arrow_verticies: u32,
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

        let arrow_renderer = arrow::Renderer::new(
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
        let index = self.entities.iter().position(|x| x.id() == id).unwrap();
        self.entities.remove(index);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
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

        let mut camera_ray = None;

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

        if let Some(hover_pos) = response.hover_pos() {
            let pos = hover_pos - response.rect.left_top();
            camera_ray = Some(self.camera.screen_ray(pos.x, pos.y));
        }

        let uniform = self.camera.uniform();

        // Generate arrow verticies
        let mut arrow_vertex_data = {
            let mut verticies = Vec::new();
            for arrow in self.arrows.iter() {
                verticies.append(&mut arrow.verticies());
            }

            self.arrow_verticies = verticies.len() as u32;
            verticies
        };

        // Generate object verticies
        let object_vertex_data = {
            let mut verticies = Vec::new();
            for entity in self.entities.iter_mut() {
                if let Entity::Object(ref mut object) = entity {
                    if let Some(ref camera_ray) = camera_ray {
                        for triangle in object.triangles.iter_mut() {
                            if camera_ray.normal.dot(&triangle.normal) > 0.0 {
                                continue;
                            }

                            if let Some(point) = camera_ray.triangle_intersect(
                                &triangle.v1,
                                &triangle.v2,
                                &triangle.v3,
                                &triangle.normal,
                            ) {
                                arrow_vertex_data.append(
                                    &mut arrow::Arrow {
                                        origin: point,
                                        normal: triangle.normal,
                                        color: [0.4, 0.4, 1.0],
                                        scale: 0.3,
                                    }
                                    .verticies(),
                                );
                                triangle.color = [1.0, 1.0, 0.0];
                            } else {
                                triangle.color = [0.7, 0.7, 0.7];
                            }
                        }
                    }

                    let mut ov = object.verticies();
                    verticies.append(&mut ov);
                }
            }

            self.object_verticies = verticies.len() as u32;
            verticies
        };

        self.arrow_verticies = arrow_vertex_data.len() as u32;

        let object_verticies = self.object_verticies;
        let arrow_verticies = self.arrow_verticies;

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
        for entity in self.entities.iter_mut() {
            entity.ui(ui, state, &mut messages);
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
    arrow_renderer: arrow::Renderer,
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
