#![allow(unused_imports)]

use cgmath::*;

use crate::OPENGL_TO_WGPU_MATRIX;

use std::num::NonZeroU64;
use std::sync::Arc;

use bytemuck;

use eframe::wgpu;

use eframe::wgpu::util::DeviceExt;

// We have got 2 seperate classes for the Camera to seperate
// the actual state of the camera which will sit in the frontend
// and the wgpu state required to perform paint calls.

pub struct CameraUniform {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
}

impl CameraUniform {
    pub fn new(device: &Arc<wgpu::Device>, view_proj: &[f32; 16]) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera"),
            contents: bytemuck::cast_slice(view_proj),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    // mat4x4
                    min_binding_size: NonZeroU64::new(16 * 4),
                },
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera"),
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
    pub fn update(&self, queue: &wgpu::Queue, view_proj: &[f32; 16]) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(view_proj));
    }
}

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,

    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,

    pub has_changed: bool,
}

impl Camera {
    pub fn resize(&mut self, width: f32, height: f32) {
        self.aspect = width / height;

        self.has_changed = true;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        OPENGL_TO_WGPU_MATRIX
            * perspective(self.fovy, self.aspect, self.znear, self.zfar)
            * Matrix4::look_to_rh(
                self.position,
                Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
                Vector3::unit_y(),
            )
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, -2.0),
            pitch: Deg(0.0).into(),
            yaw: Deg(0.0).into(),

            aspect: 1.0,
            fovy: Deg(50.0).into(),
            znear: 0.1,
            zfar: 100.0,

            has_changed: false,
        }
    }
}
