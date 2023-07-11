use nalgebra::*;
use nalgebra_glm;

use std::sync::Arc;

use bytemuck;

use eframe::wgpu;

use eframe::wgpu::util::DeviceExt;

use super::ray::Ray;

// We have got 2 seperate structs for the Camera to seperate
// the actual state of the camera which will sit in the frontend
// and the wgpu state required to perform paint calls.

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct UniformData {
    // The projection of the camera
    proj: [[f32; 4]; 4],
    // The position of the camera
    pos: [f32; 4],
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

#[derive(Debug)]
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

    pub fn normal(&self, x: f32, y: f32) -> Vector3<f32> {
        match self {
            Projection::Perspective { fovy, .. } => {
                Vector3::new(x, y, 1.0 / (fovy / 2.0).tan()).normalize()
            }
            Projection::Orthographic { .. } => Vector3::new(0.0, 0.0, 1.0),
        }
    }

    pub fn pos(&self, x: f32, y: f32) -> Vector3<f32> {
        match self {
            Projection::Perspective { .. } => Vector3::zeros(),
            Projection::Orthographic { .. } => Vector3::new(x, y, 0.0),
        }
    }

    // TODO: find out why builtin orthographic projection is not working and DRY this
    fn ortho(aspect: f32, zoom: f32, _znear: f32, _zfar: f32) -> Matrix4<f32> {
        Matrix4::new_translation(&Vector3::new(0.0, 0.0, 0.5))
            * Matrix4::new_nonuniform_scaling(&Vector3::new(zoom / aspect, zoom, 0.01))
    }
}

#[derive(Debug)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,

    width: f32,
    height: f32,

    pub projection: Projection,
}

impl Camera {
    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;

        self.projection.resize(width, height);
    }

    fn view(&self, eye: Vector3<f32>) -> Matrix4<f32> {
        nalgebra_glm::look_at_lh(
            &eye,
            &(Vector3::new(0.0, 0.0, 0.0) + self.position),
            &Vector3::y_axis(),
        )
    }

    pub fn eye(&self) -> Vector3<f32> {
        Matrix4::from_euler_angles(self.pitch, self.yaw, 0.0)
            .transform_vector(&Vector3::new(0.0, 0.0, 1.0))
            / self.zoom
            + self.position
    }

    pub fn uniform(&self) -> UniformData {
        let eye = self.eye();
        let proj = (self.projection.matrix(self.zoom) * self.view(eye)).transpose();

        UniformData {
            proj: proj.into(),
            pos: Vector4::new(eye.x, eye.y, eye.z, self.zoom).into(),
        }
    }

    pub fn screen_ray(&self, x: f32, y: f32) -> Ray {
        let x = (2.0 * x - self.width) / self.height;
        let y = (2.0 * y - self.height) / self.height;
        let rot = Matrix4::from_euler_angles(self.pitch, self.yaw, 0.0);
        let normal = -rot.transform_vector(&self.projection.normal(x, y));
        let pos = self.eye() + rot.transform_vector(&self.projection.pos(x, y));

        Ray::new(pos.xzy(), normal.xzy())
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,
            zoom: 0.1,
            height: 400.0,
            width: 400.0,

            projection: Projection::Perspective {
                aspect: 1.0,
                fovy: std::f32::consts::FRAC_PI_4,
                znear: 0.01,
                zfar: 100.0,
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
