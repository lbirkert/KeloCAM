use eframe::{egui, egui_wgpu, wgpu};
use egui::{ScrollArea, Vec2};
use nalgebra::{UnitVector3, Vector3};
use std::sync::Arc;

pub mod camera;
pub mod icons;
pub mod log;
pub mod object;
pub mod state;
pub mod tool;
pub mod renderer;

pub use camera::Camera;
pub use icons::Icons;
pub use log::Log;
pub use log::Message;
pub use state::State;
pub use tool::Action;
pub use tool::Tool;
use kelocam_core::{BoundingBox, Plane};

#[derive(Default)]
pub struct Editor {
    camera: Camera,

    pub action: Option<Action>,

    pub state: State,
    pub log: Log,
}

impl Editor {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        let device = &wgpu_render_state.device;

        let camera = camera::Camera::default();
        let camera_uniform = camera::Uniform::new(device, camera.uniform());
        let grid_renderer = renderer::grid::Renderer::new(
            device,
            wgpu_render_state.target_format,
            &camera_uniform.bind_group_layout,
        );

        let object_renderer = renderer::object::Renderer::new(
            device,
            wgpu_render_state.target_format,
            &camera_uniform.bind_group_layout,
        );

        let entity_renderer = renderer::entity::Renderer::new(
            device,
            wgpu_render_state.target_format,
            false,
            &camera_uniform.bind_group_layout,
        );

        let path_renderer = renderer::path::Renderer::new(
            device,
            wgpu_render_state.target_format,
            false,
            &camera_uniform.bind_group_layout,
        );

        wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(Renderer {
                grid_renderer,
                object_renderer,
                entity_renderer,
                path_renderer,
                camera_uniform,
            });

        Some(Self {
            camera,
            ..Default::default()
        })
    }

    pub fn move_delta(&self, plane: &Plane, before: Vec2, after: Vec2) -> Option<Vector3<f32>> {
        Some(
            self.camera.screen_ray(after.x, after.y).intersect(plane)?
                - self.camera.screen_ray(before.x, before.y).intersect(plane)?,
        )
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, messages: &mut Vec<Message>) {
        let available_size = ui.available_size();

        let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::drag());

        self.camera.handle(ui, rect, &response);

        // Handle selection
        if response.clicked() {
            let pos = response.interact_pointer_pos().unwrap() - response.rect.left_top();
            let camera_ray = self.camera.screen_ray(pos.x, pos.y);

            // Find the closest intersection point
            let mut intersection_id = 0;
            let mut intersection_dist = std::f32::INFINITY;

            for (id, object) in self.state.objects.iter() {
                if let Some(point) = camera_ray.intersect(&object.mesh) {
                    // It is not important to calculate the exact distance as we only want
                    // to compare the distance with other intersection points
                    let dist = (camera_ray.origin - point).magnitude_squared();
                    if dist < intersection_dist {
                        intersection_dist = dist;
                        intersection_id = *id;
                    }
                }
            }

            if intersection_id != 0 {
                messages.push(self.state.perform_selection(ui, intersection_id));
            } else {
                messages.push(self.state.unselect_all());
            }
        }

        // Keyboard shortcuts
        if ui.rect_contains_pointer(rect) {
            // Handle viewport delete
            if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
                for (id, _) in self.state.iter_selection() {
                    messages.push(self.state.delete_object(*id));
                }
            } else if ui.input(|i| i.key_pressed(egui::Key::G)) {
                messages.push(Message::Tool(Tool::translate()));
            } else if ui.input(|i| i.key_pressed(egui::Key::R)) {
                messages.push(Message::Tool(Tool::rotate()));
            } else if ui.input(|i| i.key_pressed(egui::Key::S)) {
                messages.push(Message::Tool(Tool::scale()));
            } else if ui.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.command) {
                if ui.input(|i| i.modifiers.shift) {
                    if self.log.can_redo() {
                        self.log.redo();
                        self.state.apply(self.log.cursor_mut());
                    }
                } else if self.log.can_undo() {
                    self.state.apply(self.log.cursor_mut());
                    self.log.undo();
                }
            }
        }

        let mut selection_inf = Vector3::from_element(std::f32::INFINITY);
        let mut selection_sup = Vector3::from_element(std::f32::NEG_INFINITY);
        let selection_origin = {
            if self.state.selected() {
                for (_, object) in self.state.iter_selection() {
                    let (min, max) = object.mesh.bb_min_max();
                    selection_inf = selection_inf.inf(&min);
                    selection_sup = selection_sup.sup(&max);
                }

                (selection_inf + selection_sup).scale(0.5)
            } else {
                Vector3::zeros()
            }
        };

        if self.state.selected() {
            // Handle viewport transformation
            if let Some(hover_pos) = response.hover_pos() {
                let pos = hover_pos - response.rect.left_top();
                let camera_ray = self.camera.screen_ray(pos.x, pos.y);
                if let Some(Action::Transform(ref axis)) = self.action {
                    if response.dragged_by(egui::PointerButton::Primary) {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);

                        let after =
                            response.interact_pointer_pos().unwrap() - response.rect.left_top();
                        let before = after - response.drag_delta();

                        let delta = self.move_delta(
                            &Plane::new(
                                selection_origin,
                                UnitVector3::new_normalize(
                                    (self.camera.eye() - selection_origin)
                                        .cross(axis.vector())
                                        .cross(axis.vector()),
                                ),
                            ),
                            before,
                            after,
                        );

                        if let Some(delta) = delta {
                            match self.state.tool {
                                Tool::Translate(ref mut tdelta) => {
                                    *tdelta += axis.vector().scale(delta.dot(axis.vector()));
                                }
                                Tool::Scale(ref mut tdelta) => {
                                    *tdelta += (2.0 * delta.dot(axis.vector()))
                                        / (selection_sup - selection_inf).magnitude();
                                }
                                Tool::ScaleNonUniformly(ref mut tdelta) => {
                                    *tdelta += axis.vector().scale(
                                        (2.0 * delta.dot(axis.vector()))
                                            / (selection_sup - selection_inf).dot(axis.vector()),
                                    );
                                }
                                Tool::Rotate(ref mut tdelta) => {
                                    *tdelta += axis.vector().scale(delta.dot(axis.vector()) * 0.1);
                                }
                            }
                        }
                    } else {
                        // Apply transformation on objects
                        for (id, object) in self.state.iter_selection() {
                            let mut mesh = object.mesh.clone();
                            mesh.translate(&-selection_origin);
                            self.state.tool.apply(&mut mesh);
                            // Snap to plate
                            let min  = mesh.bb_min();
                            mesh.translate(
                                &(selection_origin
                                    + Vector3::new(0.0, 0.0, -min.z - selection_origin.z)),
                            );

                            messages.push(Message::Mesh { id: *id, mesh });
                        }

                        self.action = None;
                    }
                } else if let Some(axis) = self.state.tool.intersect(
                    &selection_origin,
                    0.15 / self.camera.zoom,
                    &camera_ray,
                ) {
                    if response.dragged_by(egui::PointerButton::Primary) {
                        self.action = Some(tool::Action::Transform(axis));
                    } else {
                        self.action = Some(tool::Action::Hover(axis));
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
                    }
                } else {
                    self.action = None;
                }
            }
        }

        let mut object_verticies = Vec::new();
        let mut entity_verticies: Vec<renderer::entity::Vertex> = Vec::new();
        let mut path_verticies = Vec::new();
        let mut path_indicies = Vec::new();

        // Generate tool verticies
        if self.state.selected() {
            self.state.tool.generate(
                &selection_origin,
                0.15 / self.camera.zoom,
                &self.action,
                &mut path_verticies,
                &mut path_indicies,
                &mut entity_verticies,
            )
        }

        // Generate object verticies
        for (id, object) in self.state.objects.iter() {
            if self.state.selection.contains(id) {
                let mut mesh = object.mesh.clone();
                mesh.translate(&-selection_origin);
                self.state.tool.apply(&mut mesh);
                mesh.translate(&selection_origin);
                renderer::object::generate(&mesh.triangles, [1.0, 0.5, 0.0], &mut object_verticies);
            } else {
                renderer::object::generate(
                    &object.mesh.triangles,
                    [1.0, 1.0, 1.0],
                    &mut object_verticies,
                );
            }
        }

        // Generate visual camera center
        {
            let o = self.camera.position.xzy();
            let scale = 1.0 / self.camera.zoom * 10.0 / self.camera.height;
            let x = Vector3::x_axis().scale(scale);
            let y = Vector3::y_axis().scale(scale);
            let z = Vector3::z_axis().scale(scale);

            renderer::path::generate_open(
                &[o + x, o - x],
                [1.0, 0.3, 0.3, 0.9],
                6.0 / self.camera.height,
                &mut path_verticies,
                &mut path_indicies,
            );
            renderer::path::generate_open(
                &[o + y, o - y],
                [1.0, 0.3, 0.3, 0.9],
                6.0 / self.camera.height,
                &mut path_verticies,
                &mut path_indicies,
            );
            renderer::path::generate_open(
                &[o + z, o - z],
                [1.0, 0.3, 0.3, 0.9],
                6.0 / self.camera.height,
                &mut path_verticies,
                &mut path_indicies,
            );
        }

        let entity_vertex_count = entity_verticies.len() as u32;
        let object_vertex_count = object_verticies.len() as u32;
        let path_vertex_count = path_verticies.len() as u32;
        let path_index_count = path_indicies.len() as u32;

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

                // Update arrow vertex buffer
                queue.write_buffer(
                    &renderer.entity_renderer.vertex_buffer,
                    0,
                    bytemuck::cast_slice(entity_verticies.as_slice()),
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
                    entity_vertex_count,
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

    pub fn sidebar(&mut self, ui: &mut egui::Ui, icons: &Icons, messages: &mut Vec<Message>) {
        if self.state.objects.is_empty() {
            ui.label("Click on File > Open to import a model");
        }

        let row_height = ui.text_style_height(&egui::TextStyle::Button) + 5.0;
        let max_height =
            ui.available_height() - ui.text_style_height(&egui::TextStyle::Button) - 10.0;

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_height(max_height)
            .show_rows(
                ui,
                row_height,
                self.state.object_ids.len(),
                |ui, row_range| {
                    for i in row_range {
                        let id = self.state.object_ids[i];
                        self.state.objects[&id].ui(ui, id, &self.state, icons, messages);
                    }
                },
            );

        let mut translate = false;
        let mut scale = false;
        let mut rotate = false;

        match self.state.tool {
            Tool::Translate(_) => translate = true,
            Tool::Scale(_) | Tool::ScaleNonUniformly(_) => scale = true,
            Tool::Rotate(_) => rotate = true,
        };

        ui.horizontal(|ui| {
            if ui.selectable_label(translate, "G").clicked() {
                messages.push(Message::Tool(Tool::translate()));
            }

            if ui.selectable_label(scale, "S").clicked() {
                messages.push(Message::Tool(Tool::scale()));
            }

            if ui.selectable_label(rotate, "R").clicked() {
                messages.push(Message::Tool(Tool::rotate()));
            }
        });
    }
}

pub struct Renderer {
    camera_uniform: camera::Uniform,
    grid_renderer: renderer::grid::Renderer,
    object_renderer: renderer::object::Renderer,
    path_renderer: renderer::path::Renderer,
    entity_renderer: renderer::entity::Renderer,
}

impl Renderer {
    fn render<'rp>(
        &'rp self,
        render_pass: &mut wgpu::RenderPass<'rp>,
        object_vertex_count: u32,
        entity_vertex_count: u32,
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

        if entity_vertex_count != 0 {
            self.entity_renderer
                .render(render_pass, entity_vertex_count);
        }
    }
}

