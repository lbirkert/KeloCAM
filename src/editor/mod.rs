use eframe::{egui, egui_wgpu, wgpu};
use egui::ScrollArea;
use nalgebra::Vector3;
use std::{collections::HashSet, sync::Arc};

pub mod object;
pub mod toolpath;

pub mod path;
pub mod ray;
pub mod tool;

pub mod state;

pub mod grid;

pub mod camera;

#[derive(Default)]
pub struct Editor {
    camera: camera::Camera,

    pub id_counter: u32,

    pub objects: Vec<object::Object>,
    pub object_changed: bool,
    pub object_vertex_count: u32,
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

        let path_renderer = path::Renderer::new(
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
                path_renderer,
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
            } else {
                state.selection.clear(&mut self.objects);
            }
        }

        // Keyboard shortcuts
        if ui.rect_contains_pointer(rect) {
            // Handle viewport delete
            if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
                for object in self
                    .objects
                    .iter()
                    .filter(|i| state.selection.contains(&i.id))
                {
                    messages.push(state::Message::Delete(object.id));
                }
            } else if ui.input(|i| i.key_pressed(egui::Key::G)) {
                state.switch_tool(tool::Tool::Move, &mut self.objects);
            } else if ui.input(|i| i.key_pressed(egui::Key::R)) {
                state.switch_tool(tool::Tool::Rotate, &mut self.objects);
            } else if ui.input(|i| i.key_pressed(egui::Key::S)) {
                if ui.input(|i| i.modifiers.shift) {
                    state.switch_tool(tool::Tool::Scale { uniform: false }, &mut self.objects);
                } else {
                    state.switch_tool(tool::Tool::Scale { uniform: true }, &mut self.objects);
                }
            }
        }

        if state.selection.valid() {
            // Handle viewport transformation
            if let Some(hover_pos) = response.hover_pos() {
                let pos = hover_pos - response.rect.left_top();
                let camera_ray = self.camera.screen_ray(pos.x, pos.y);
                if let Some(tool::Action::Transform { ref axis }) = state.action {
                    if response.dragged_by(egui::PointerButton::Primary) {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);

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
                                tool::Tool::Scale { uniform } => {
                                    let fac = 2.0
                                        / if uniform {
                                            (state.selection.pre_sup - state.selection.pre_inf)
                                                .magnitude()
                                        } else {
                                            (state.selection.pre_sup - state.selection.pre_inf)
                                                .dot(&axis.vector())
                                        };

                                    let delta = if uniform {
                                        Vector3::from_element(1.0)
                                    } else {
                                        axis.vector()
                                    }
                                    .scale((after - before).dot(&axis.vector()) * fac);

                                    if let object::Transformation::Scale { ref mut scale } =
                                        &mut state.selection.transformation
                                    {
                                        *scale += delta;
                                    }
                                }
                                tool::Tool::Rotate => {
                                    let delta = axis
                                        .vector()
                                        .scale((after - before).dot(&axis.vector()) * 0.1);

                                    if let object::Transformation::Rotate { ref mut rotate } =
                                        &mut state.selection.transformation
                                    {
                                        *rotate += delta;
                                    }
                                }
                            }
                        }
                    } else {
                        state.selection.apply(&mut self.objects);
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

        // Generate object verticies
        let object_verticies = {
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

            self.object_vertex_count = verticies.len() as u32;
            verticies
        };

        let mut path_verticies = Vec::new();
        let mut path_indicies = Vec::new();

        // Generate tool verticies
        let mut tool_verticies = Vec::new();

        if state.selection.valid() {
            state.tool.generate(
                &mut tool_verticies,
                &mut path_indicies,
                &mut path_verticies,
                &state.selection.origin,
                0.02 / self.camera.zoom,
                &state.action,
            );
        }

        let path_vertex_count = path_verticies.len() as u32;
        let path_index_count = path_indicies.len() as u32;

        let object_vertex_count = self.object_vertex_count;
        let tool_vertex_count = tool_verticies.len() as u32;

        let uniform = self.camera.uniform();

        let cb = egui_wgpu::CallbackFn::new()
            .prepare(move |_device, queue, _encoder, paint_callback_resources| {
                let renderer: &Renderer = paint_callback_resources.get().unwrap();

                renderer.camera_uniform.update(queue, uniform);

                // Update object vertex buffer
                queue.write_buffer(
                    &renderer.object_renderer.vertex_buffer,
                    0,
                    bytemuck::cast_slice(object_verticies.as_slice()),
                );

                // Update tool vertex buffer
                queue.write_buffer(
                    &renderer.tool_renderer.vertex_buffer,
                    0,
                    bytemuck::cast_slice(tool_verticies.as_slice()),
                );

                // Update path vertex buffer
                queue.write_buffer(
                    &renderer.path_renderer.vertex_buffer,
                    0,
                    bytemuck::cast_slice(path_verticies.as_slice()),
                );
                queue.write_buffer(
                    &renderer.path_renderer.index_buffer,
                    0,
                    bytemuck::cast_slice(path_indicies.as_slice()),
                );

                Vec::new()
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let renderer: &Renderer = paint_callback_resources.get().unwrap();
                renderer.render(
                    render_pass,
                    object_vertex_count,
                    tool_vertex_count,
                    path_vertex_count,
                    path_index_count,
                );
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
        if self.objects.is_empty() {
            ui.label("Click on File > Open to import a model");
        }

        let row_height = ui.text_style_height(&egui::TextStyle::Button) + 5.0;
        let max_height =
            ui.available_height() - ui.text_style_height(&egui::TextStyle::Button) - 10.0;
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_height(max_height)
            .show_rows(ui, row_height, self.objects.len(), |ui, row_range| {
                for i in row_range {
                    self.objects[i].ui(ui, state, messages);
                }
            });

        let mut tmove = false;
        let mut tscale = false;
        let mut trotate = false;

        match state.tool {
            tool::Tool::Move => tmove = true,
            tool::Tool::Scale { .. } => tscale = true,
            tool::Tool::Rotate => trotate = true,
        };

        ui.horizontal(|ui| {
            if ui.selectable_label(tmove, "G").clicked() {
                state.switch_tool(tool::Tool::Move, &mut self.objects);
            }

            if ui.selectable_label(tscale, "S").clicked() {
                state.switch_tool(tool::Tool::Scale { uniform: true }, &mut self.objects);
            }

            if ui.selectable_label(trotate, "R").clicked() {
                state.switch_tool(tool::Tool::Rotate, &mut self.objects);
            }
        });
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
    path_renderer: path::Renderer,
}

impl Renderer {
    fn render<'rp>(
        &'rp self,
        render_pass: &mut wgpu::RenderPass<'rp>,
        object_vertex_count: u32,
        tool_vertex_count: u32,
        path_vertex_count: u32,
        path_index_count: u32,
    ) {
        render_pass.set_bind_group(0, &self.camera_uniform.bind_group, &[]);
        self.grid_renderer.render(render_pass);

        if object_vertex_count != 0 {
            self.object_renderer
                .render(render_pass, object_vertex_count);
        }

        if path_vertex_count != 0 {
            self.path_renderer
                .render(render_pass, path_vertex_count, path_index_count);
        }

        if tool_vertex_count != 0 {
            self.tool_renderer.render(render_pass, tool_vertex_count);
        }
    }
}
