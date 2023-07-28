use eframe::{egui, egui_wgpu, wgpu};
use egui::{ScrollArea, Vec2};
use nalgebra::{UnitVector3, Vector3};
use std::{collections::HashSet, sync::Arc};

use crate::{core::primitives::Plane, renderer};

pub mod camera;
pub mod object;
pub mod state;
pub mod tool;

#[derive(Default)]
pub struct Editor {
    camera: camera::Camera,

    pub id_counter: u32,

    pub objects: Vec<object::Object>,

    pub z_slice: f32,
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

    pub fn inf_sup(&self, objects: &HashSet<u32>) -> (Vector3<f32>, Vector3<f32>) {
        let mut inf = Vector3::from_element(std::f32::INFINITY);
        let mut sup = Vector3::from_element(std::f32::NEG_INFINITY);
        for object in self.objects.iter().filter(|o| objects.contains(&o.id)) {
            let (oinf, osup) = object.mesh.inf_sup();

            inf = inf.inf(&oinf);
            sup = sup.sup(&osup);
        }

        (inf, sup)
    }

    pub fn move_delta(&self, plane: &Plane, before: Vec2, after: Vec2) -> Option<Vector3<f32>> {
        Some(
            plane.intersect(&self.camera.screen_ray(after.x, after.y))?
                - plane.intersect(&self.camera.screen_ray(before.x, before.y))?,
        )
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
            let mut intersection_dist = std::f32::INFINITY;

            for object in self.objects.iter() {
                if let Some(point) = object.mesh.intersection(&camera_ray) {
                    // It is not important to calculate the exact distance as we only want
                    // to compare the distance with other intersection points
                    let dist = (camera_ray.origin - point).magnitude_squared();
                    if dist < intersection_dist {
                        intersection_dist = dist;
                        intersection_id = object.id;
                    }
                }
            }

            if intersection_id != 0 {
                messages.push(state::Message::Select(intersection_id));
            } else {
                state.selection.clear();
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
                state.tool = tool::Tool::Move;
            } else if ui.input(|i| i.key_pressed(egui::Key::R)) {
                state.tool = tool::Tool::Rotate;
            } else if ui.input(|i| i.key_pressed(egui::Key::S)) {
                if ui.input(|i| i.modifiers.shift) {
                    state.tool = tool::Tool::Scale { uniform: false };
                } else {
                    state.tool = tool::Tool::Scale { uniform: true };
                }
            }
        }

        if state.selection.valid() {
            // Handle viewport transformation
            if let Some(hover_pos) = response.hover_pos() {
                let pos = hover_pos - response.rect.left_top();
                let camera_ray = self.camera.screen_ray(pos.x, pos.y);
                if let Some(tool::Action::Transform(ref axis)) = state.action {
                    if response.dragged_by(egui::PointerButton::Primary) {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);

                        let after =
                            response.interact_pointer_pos().unwrap() - response.rect.left_top();
                        let before = after - response.drag_delta();

                        let delta = self.move_delta(
                            &Plane::new(
                                state.selection.origin,
                                UnitVector3::new_normalize(
                                    (self.camera.eye() - state.selection.origin)
                                        .cross(&axis.vector)
                                        .cross(&axis.vector),
                                ),
                            ),
                            before,
                            after,
                        );

                        if let Some(delta) = delta {
                            match state.tool {
                                tool::Tool::Move => {
                                    let delta = axis.vector.scale(delta.dot(&axis.vector));

                                    for object in self.objects.iter_mut() {
                                        if state.selection.contains(&object.id) {
                                            object.mesh.translate(&delta);
                                        }
                                    }
                                }
                                tool::Tool::Scale { uniform } => {
                                    let fac = 2.0
                                        / if uniform {
                                            (state.selection.sup - state.selection.inf).magnitude()
                                        } else {
                                            (state.selection.sup - state.selection.inf)
                                                .dot(&axis.vector)
                                        };

                                    let delta = if uniform {
                                        Vector3::from_element(1.0)
                                    } else {
                                        axis.vector.into_inner()
                                    }
                                    .scale(delta.dot(&axis.vector) * fac);

                                    for object in self.objects.iter_mut() {
                                        if state.selection.contains(&object.id) {
                                            object.mesh.translate(&-state.selection.origin);
                                            object.mesh.scale_non_uniformly(&delta);
                                            object.mesh.translate(&state.selection.origin);
                                        }
                                    }
                                }
                                tool::Tool::Rotate => {
                                    let delta = axis.vector.scale(delta.dot(&axis.vector) * 0.1);

                                    for object in self.objects.iter_mut() {
                                        if state.selection.contains(&object.id) {
                                            object.mesh.translate(&-state.selection.origin);
                                            object.mesh.rotate(&delta);
                                            object.mesh.translate(&state.selection.origin);
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        state.action = None
                    }
                } else if let Some(axis) = state.tool.intersect(
                    &state.selection.origin,
                    0.02 / self.camera.zoom,
                    &camera_ray,
                ) {
                    if response.dragged_by(egui::PointerButton::Primary) {
                        state.action = Some(tool::Action::Transform(axis));
                    } else {
                        state.action = Some(tool::Action::Hover(axis));
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
                    }
                } else {
                    state.action = None;
                }
            }

            state.selection.update_origin(&self.objects);
        }

        let mut object_verticies = Vec::new();
        let mut entity_verticies: Vec<renderer::entity::Vertex> = Vec::new();
        let mut path_verticies = Vec::new();
        let mut path_indicies = Vec::new();

        // Generate object verticies
        for object in self.objects.iter_mut() {
            if state.selection.contains(&object.id) {
                renderer::object::generate(object, [1.0, 0.0, 0.0], &mut object_verticies);
            } else {
                renderer::object::generate(object, [1.0, 1.0, 1.0], &mut object_verticies);
            }

            for points in object
                .mesh
                .z_slice(self.z_slice, &mut entity_verticies)
                .iter()
            {
                renderer::path::generate_closed(
                    &points
                        .extend3(
                            &Vector3::new(0.0, 0.0, self.z_slice),
                            &Vector3::x_axis(),
                            &Vector3::y_axis(),
                        )
                        .points,
                    [1.0, 0.0, 1.0],
                    0.01,
                    &mut path_verticies,
                    &mut path_indicies,
                );
            }
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

    pub fn sidebar(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut state::State,
        messages: &mut Vec<state::Message>,
    ) {
        if self.objects.is_empty() {
            ui.label("Click on File > Open to import a model");
        }

        ui.add(egui::DragValue::new(&mut self.z_slice).speed(0.01));

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
                state.tool = tool::Tool::Move;
            }

            if ui.selectable_label(tscale, "S").clicked() {
                state.tool = tool::Tool::Scale { uniform: true };
            }

            if ui.selectable_label(trotate, "R").clicked() {
                state.tool = tool::Tool::Rotate;
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

        if entity_vertex_count != 0 {
            self.entity_renderer
                .render(render_pass, entity_vertex_count);
        }

        if path_vertex_count != 0 {
            self.path_renderer
                .render(render_pass, path_vertex_count, path_index_count);
        }
    }
}
