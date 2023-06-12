use nalgebra::*;
use nalgebra_glm;

use std::sync::Arc;

use bytemuck;

use eframe::wgpu;

use eframe::wgpu::util::DeviceExt;

// We have got 2 seperate structs for the Camera to seperate
// the actual state of the camera which will sit in the frontend
// and the wgpu state required to perform paint calls.

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct CameraUniformData {
    // The projection of the camera
    proj: [[f32; 4]; 4],
    // The position of the camera
    pos: [f32; 4],
}

pub struct CameraUniform {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
}

impl CameraUniform {
    pub fn new(device: &Arc<wgpu::Device>, data: CameraUniformData) -> Self {
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
    pub fn update(&self, queue: &wgpu::Queue, data: CameraUniformData) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
    }
}

#[derive(Debug)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,

    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn resize(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }

    fn projection(&self) -> Matrix4<f32> {
        nalgebra_glm::perspective_lh(self.aspect, self.fovy, self.znear, self.zfar)
    }

    fn view(&self, eye: Vector3<f32>) -> Matrix4<f32> {
        nalgebra_glm::look_at_lh(
            &eye,
            &(Vector3::new(0.0, 0.0, 0.0) + self.position),
            &Vector3::y_axis(),
        )
    }

    fn eye(&self) -> Vector3<f32> {
        Matrix4::from_euler_angles(self.pitch, self.yaw, 0.0)
            .transform_vector(&Vector3::new(0.0, 0.0, 1.0))
            / self.zoom
            + self.position
    }

    pub fn uniform(&self) -> CameraUniformData {
        let eye = self.eye();
        let proj = (self.projection() * self.view(eye)).transpose();

        CameraUniformData {
            proj: proj.into(),
            pos: Vector4::new(eye.x, eye.y, eye.z, self.zoom).into(),
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,

            zoom: 0.1,

            aspect: 1.0,
            fovy: std::f32::consts::FRAC_PI_4,
            znear: 0.01,
            zfar: 100.0,
        }
    }
}
