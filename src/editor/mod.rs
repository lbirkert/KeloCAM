use eframe::{egui, egui_wgpu, wgpu};
use nalgebra::Vector3;
use std::{collections::HashSet, sync::Arc};

pub mod object;
pub mod tool;
pub mod toolpath;

pub mod ray;

pub mod state;

pub mod grid;

pub mod camera;

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

        let tool_renderer = tool::Renderer::new(
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
                tool_renderer,
                camera_uniform,
            });

        Some(Self {
            camera,
            ..Default::default()
        })
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

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut state::State,
        messages: &mut Vec<state::Message>,
    ) {
        let available_size = ui.available_size();

        let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::drag());

        self.camera.handle(ui, rect, &response);

        // Handle selection
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
                messages.push(state::Message::Select(intersection_id));
            }
        }

        if state.selection.valid() {
            // Handle viewport delete
            if ui.rect_contains_pointer(rect) && ui.input(|i| i.key_pressed(egui::Key::Delete)) {
                for object in self
                    .objects
                    .iter()
                    .filter(|i| state.selection.contains(&i.id))
                {
                    messages.push(state::Message::Delete(object.id));
                }
            }

            // Handle viewport transformation
            if let Some(hover_pos) = response.hover_pos() {
                let pos = hover_pos - response.rect.left_top();
                let camera_ray = self.camera.screen_ray(pos.x, pos.y);
                if let Some(tool::Action::Transform { ref axis }) = state.action {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
                    if response.dragged_by(egui::PointerButton::Primary) {
                        let after =
                            response.interact_pointer_pos().unwrap() - response.rect.left_top();
                        let before = after - response.drag_delta();

                        let plane_normal = (camera_ray.origin - state.selection.origin)
                            .cross(&axis.vector())
                            .cross(&axis.vector());

                        let before = self
                            .camera
                            .screen_ray(before.x, before.y)
                            .plane_intersect(&state.selection.origin, &plane_normal);
                        let after = self
                            .camera
                            .screen_ray(after.x, after.y)
                            .plane_intersect(&state.selection.origin, &plane_normal);

                        if before.is_some() && after.is_some() {
                            let before = unsafe { before.as_ref().unwrap_unchecked() };
                            let after = unsafe { after.as_ref().unwrap_unchecked() };

                            match state.tool {
                                tool::Tool::Move => {
                                    let delta =
                                        axis.vector().scale((after - before).dot(&axis.vector()));

                                    if let object::Transformation::Translate { ref mut translate } =
                                        &mut state.selection.transformation
                                    {
                                        *translate += delta;
                                    }
                                }
                                tool::Tool::Scale => {
                                    let delta = axis
                                        .vector()
                                        .scale((after - before).dot(&axis.vector()) * 2.0)
                                        .component_div(
                                            &(state.selection.pre_sup - state.selection.pre_inf),
                                        );

                                    if let object::Transformation::Scale { ref mut scale } =
                                        &mut state.selection.transformation
                                    {
                                        *scale += delta;
                                    }
                                }
                                _ => panic!(),
                            }
                        }
                    } else {
                        state.action = None
                    }
                } else if let Some(axis) = state.tool.intersect(
                    &state.selection.origin,
                    &camera_ray,
                    0.02 / self.camera.zoom,
                ) {
                    if response.dragged_by(egui::PointerButton::Primary) {
                        state.action = Some(tool::Action::Transform { axis });
                    } else {
                        state.action = Some(tool::Action::Hover { axis });
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
                    }
                } else {
                    state.action = None;
                }
            }

            state.selection.update_origin(&self.objects);
        }

        // Generate tool verticies
        let tool_vertex_data = {
            let mut verticies = Vec::new();

            if state.selection.valid() {
                verticies.append(&mut state.tool.verticies(
                    &state.selection.origin,
                    0.02 / self.camera.zoom,
                    &state.action,
                ));
            }

            verticies
        };

        // Generate object verticies
        let object_vertex_data = {
            let mut verticies = Vec::new();

            let selection_applyable = state
                .selection
                .transformation
                .to_applyable(state.selection.origin);

            for object in self.objects.iter_mut() {
                if state.selection.contains(&object.id) {
                    object.color = [0.7, 0.7, 1.0];
                    verticies.append(&mut object.transformed_verticies(&selection_applyable));
                } else {
                    object.color = [0.7, 0.7, 0.7];
                    verticies.append(&mut object.verticies());
                }
            }

            self.object_verticies = verticies.len() as u32;
            verticies
        };

        let object_verticies = self.object_verticies;
        let tool_verticies = tool_vertex_data.len() as u32;

        let uniform = self.camera.uniform();

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

                // Update tool vertex buffer
                queue.write_buffer(
                    &renderer.tool_renderer.vertex_buffer,
                    0,
                    bytemuck::cast_slice(tool_vertex_data.as_slice()),
                );

                Vec::new()
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let renderer: &Renderer = paint_callback_resources.get().unwrap();
                renderer.render(render_pass, object_verticies, tool_verticies);
            });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };

        ui.painter().add(callback);
    }

    pub fn sidebar(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut state::State,
        messages: &mut Vec<state::Message>,
    ) {
        for object in self.objects.iter_mut() {
            object.ui(ui, state, messages);
        }

        if ui.button("Move").clicked() {
            state.switch_tool(tool::Tool::Move, &mut self.objects);
        }

        if ui.button("Scale").clicked() {
            state.switch_tool(tool::Tool::Scale, &mut self.objects);
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
    tool_renderer: tool::Renderer,
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
            self.tool_renderer.render(render_pass, arrow_verticies);
        }
    }
}
