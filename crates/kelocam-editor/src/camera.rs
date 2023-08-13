use bytemuck;
use eframe::wgpu::{self, util::DeviceExt};
use nalgebra::{Matrix4, UnitVector3, Vector3};
use nalgebra_glm;
use std::sync::Arc;

use kelocam_core::primitives::Ray;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct UniformData {
    // The projection matrix of the camera
    view_proj: [[f32; 4]; 4],
    // The position of the camera
    view_pos: [f32; 4],
    // width, height, zoom, unused (consider using for DPR)
    dimensions: [f32; 4],
}

pub struct Uniform {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
}

impl Uniform {
    pub fn new(device: &Arc<wgpu::Device>, data: UniformData) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera"),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                resource: buffer.as_entire_binding(),
                binding: 0,
            }],
        });

        Self {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    // TODO: Find out if method is really necessary
    pub fn update(&self, queue: &wgpu::Queue, data: UniformData) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
    }
}

pub enum Projection {
    Perspective {
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    },
    Orthographic {
        aspect: f32,
        znear: f32,
        zfar: f32,
    },
}

impl Projection {
    pub fn matrix(&self, zoom: f32) -> Matrix4<f32> {
        match self {
            Projection::Perspective {
                aspect,
                fovy,
                znear,
                zfar,
            } => nalgebra_glm::perspective_lh(*aspect, *fovy, *znear, *zfar),
            /* Find out why tf this does not work */
            Projection::Orthographic {
                aspect,
                znear,
                zfar,
            } => Self::ortho(*aspect, zoom, *znear, *zfar),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        match self {
            Projection::Perspective { ref mut aspect, .. } => *aspect = width / height,
            Projection::Orthographic { ref mut aspect, .. } => *aspect = width / height,
        }
    }

    pub fn normal(&self, x: f32, y: f32, _zoom: f32) -> Vector3<f32> {
        match self {
            Projection::Perspective { fovy, .. } => {
                Vector3::new(x, y, 1.0 / (fovy / 2.0).tan()).normalize()
            }
            Projection::Orthographic { .. } => Vector3::new(0.0, 0.0, 1.0),
        }
    }

    pub fn pos(&self, x: f32, y: f32, zoom: f32) -> Vector3<f32> {
        match self {
            Projection::Perspective { .. } => Vector3::zeros(),
            Projection::Orthographic { .. } => Vector3::new(-x, -y, 0.0).scale(0.5 / zoom),
        }
    }

    // TODO: find out why builtin orthographic projection is not working and DRY this
    fn ortho(aspect: f32, zoom: f32, _znear: f32, _zfar: f32) -> Matrix4<f32> {
        let mut mat = Matrix4::new_translation(&Vector3::new(0.0, 0.0, 0.5));
        mat.append_nonuniform_scaling_mut(&Vector3::new(zoom / (aspect * 0.5), zoom / 0.5, 0.001));
        mat
    }
}

pub struct Camera {
    pub position: Vector3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,

    pub width: f32,
    pub height: f32,

    pub projection: Projection,
}

impl Camera {
    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;

        self.projection.resize(width, height);
    }

    pub fn view(&self, eye: Vector3<f32>) -> Matrix4<f32> {
        nalgebra_glm::look_at_lh(
            &eye,
            &(Vector3::new(0.0, 0.0, 0.0) + self.position),
            &Vector3::y_axis(),
        )
    }

    pub fn proj(&self) -> Matrix4<f32> {
        self.projection.matrix(self.zoom)
    }

    pub fn eye(&self) -> Vector3<f32> {
        Matrix4::from_euler_angles(self.pitch, self.yaw, 0.0)
            .transform_vector(&Vector3::new(0.0, 0.0, 1.0))
            / self.zoom
            + self.position
    }

    pub fn uniform(&self) -> UniformData {
        let eye = self.eye();
        let mut view_proj = self.proj() * self.view(eye);
        view_proj.transpose_mut();

        UniformData {
            view_proj: view_proj.into(),
            view_pos: eye.to_homogeneous().into(),
            dimensions: [self.width, self.height, self.zoom, 0.0],
        }
    }

    pub fn screen_ray(&self, x: f32, y: f32) -> Ray {
        let x = (2.0 * x - self.width) / self.height;
        let y = (2.0 * y - self.height) / self.height;
        let rot = Matrix4::from_euler_angles(self.pitch, self.yaw, 0.0);
        let normal = -rot.transform_vector(&self.projection.normal(x, y, self.zoom));
        let pos = self.eye() + rot.transform_vector(&self.projection.pos(x, y, self.zoom));

        Ray::new(pos.xzy(), UnitVector3::new_unchecked(normal.xzy()))
    }

    pub fn handle(&mut self, ui: &egui::Ui, rect: egui::Rect, response: &egui::Response) {
        self.resize(rect.size().x, rect.size().y);

        const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;

        // Rotation
        if response.dragged_by(egui::PointerButton::Secondary) {
            self.yaw += response.drag_delta().x * 0.005;
            self.pitch += response.drag_delta().y * -0.005;

            self.pitch = self.pitch.clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2);
        }

        // Translation
        if response.dragged_by(egui::PointerButton::Middle) {
            let delta = Matrix4::from_euler_angles(self.pitch, self.yaw, 0.0).transform_vector(
                &Vector3::new(
                    response.drag_delta().x * 0.001 / self.zoom,
                    response.drag_delta().y * 0.001 / self.zoom,
                    0.0,
                ),
            );

            self.position += delta;
        }

        // Zoom
        if ui.rect_contains_pointer(rect) {
            ui.ctx().input(|i| {
                for event in &i.events {
                    if let egui::Event::Scroll(v) = event {
                        if v[0] == 0.0 {
                            if v[1] > 0.0 {
                                //self.camera.zoom += 0.001 * v[1];
                                self.zoom *= 1.0 + 0.001 * v[1];
                            } else if v[1] < 0.0 {
                                //self.camera.zoom += 0.001 * v[1];
                                self.zoom /= 1.0 + 0.001 * -v[1];
                            }
                        }
                    }
                }
            });
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            yaw: std::f32::consts::FRAC_PI_4,
            pitch: -std::f32::consts::FRAC_PI_6,

            zoom: 0.04,
            height: 400.0,
            width: 400.0,

            projection: Projection::Perspective {
                aspect: 1.0,
                fovy: std::f32::consts::FRAC_PI_4,
                znear: 0.01,
                zfar: 300.0,
            },
            /*
            projection: Projection::Orthographic {
                aspect: 1.0,
                znear: 0.01,
                zfar: 100.0,
            },
            */
        }
    }
}
