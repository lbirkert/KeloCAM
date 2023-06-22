use eframe::{egui, egui_wgpu, wgpu};
use nalgebra::{Matrix4, Vector3};
use std::sync::Arc;

pub mod object;
pub mod toolpath;

pub mod state;

pub mod grid;

pub mod camera;

pub enum Entity {
    Object(object::Object),
}

impl Entity {
    /// Scale an entity by a given vector.
    pub fn scale(&mut self, delta: Vector3<f32>) {
        match self {
            Entity::Object(v) => v.scale(delta),
        }
    }

    /// Translate an entity by a given vector.
    pub fn translate(&mut self, delta: Vector3<f32>) {
        match self {
            Entity::Object(v) => v.translate(delta),
        }
    }
    /// Rotate an entity using euler axies in radians.
    pub fn rotate(&mut self, delta: Vector3<f32>) {
        match self {
            Entity::Object(v) => v.rotate(delta),
        }
    }

    /// Returns the entity's infimum (aka. componentwise min) and the
    /// supremum (aka. componentwise max) vector (can be used as bounding box).
    pub fn inf_sup(&self) -> (Vector3<f32>, Vector3<f32>) {
        match self {
            Entity::Object(v) => v.inf_sup(),
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut state::State,
        messages: &mut Vec<state::Message>,
    ) {
        match self {
            Entity::Object(v) => v.ui(ui, state, messages),
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            Entity::Object(v) => v.id,
        }
    }

    pub fn set_id(&mut self, id: u32) {
        match self {
            Entity::Object(v) => v.id = id,
        }
    }

    pub fn scale_at(&mut self, origin: Vector3<f32>, delta: Vector3<f32>) {
        self.translate(-origin);
        self.scale(delta);
        self.translate(origin);
    }

    pub fn rotate_at(&mut self, origin: Vector3<f32>, delta: Vector3<f32>) {
        self.translate(-origin);
        self.rotate(delta);
        self.translate(origin);
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

        wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(Renderer {
                grid_renderer,
                object_renderer,
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

        let uniform = self.camera.uniform();

        let object_vertex_buffer = if self.object_changed {
            let mut verticies = Vec::new();
            for entity in self.entities.iter() {
                if let Entity::Object(object) = entity {
                    verticies.append(&mut object.verticies());
                }
            }
            self.object_verticies = verticies.len() as u32;
            Some(verticies)
        } else {
            None
        };
        let object_verticies = self.object_verticies;

        let cb = egui_wgpu::CallbackFn::new()
            .prepare(move |_device, queue, _encoder, paint_callback_resources| {
                let renderer: &Renderer = paint_callback_resources.get().unwrap();

                renderer.camera_uniform.update(queue, uniform);
                if let Some(ref object_vertex_buffer) = object_vertex_buffer {
                    queue.write_buffer(
                        &renderer.object_renderer.vertex_buffer,
                        0,
                        bytemuck::cast_slice(object_vertex_buffer.as_slice()),
                    );
                }

                Vec::new()
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let renderer: &Renderer = paint_callback_resources.get().unwrap();
                renderer.render(render_pass, object_verticies);
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
}

impl Renderer {
    fn render<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>, object_verticies: u32) {
        render_pass.set_bind_group(0, &self.camera_uniform.bind_group, &[]);
        self.grid_renderer.render(render_pass);
        if object_verticies != 0 {
            self.object_renderer.render(render_pass, object_verticies);
        }
    }
}
