use std::io::Cursor;

use eframe::wgpu;
use eframe::wgpu::util::DeviceExt;
use nalgebra::{ArrayStorage, Matrix4, Vector3};
use std::sync::Arc;
use stl;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex {
    pos: [f32; 3],
    normal: [f32; 3],
}

pub struct Triangle {
    pub normal: Vector3<f32>,
    pub v1: Vector3<f32>,
    pub v2: Vector3<f32>,
    pub v3: Vector3<f32>,
}

pub const VERTEX_SIZE: usize = std::mem::size_of::<Vertex>();
pub const VERTEX_BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: VERTEX_SIZE as u64,
    attributes: &[
        // Position
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 0,
            shader_location: 0,
        },
        // Normal vector
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 4 * 3,
            shader_location: 1,
        },
    ],
    step_mode: wgpu::VertexStepMode::Vertex,
};

// Each object gets its own draw call because transfering the
// projection matrix on each vertex is memory intensive

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ObjectUniformData {
    proj: [[f32; 4]; 4],
}

impl Default for ObjectUniformData {
    fn default() -> Self {
        Self {
            proj: Matrix4::zeros().into(),
        }
    }
}

pub struct ObjectUniform {
    pub bind_group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
}

impl ObjectUniform {
    pub fn new(
        device: &Arc<wgpu::Device>,
        data: ObjectUniformData,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("object"),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("object"),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                resource: buffer.as_entire_binding(),
                binding: 0,
            }],
        });

        Self { buffer, bind_group }
    }

    // TODO: Find out if method is really necessary
    pub fn update(&self, queue: &wgpu::Queue, data: ObjectUniformData) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
    }
}

pub struct Object {
    pub triangles: Vec<Triangle>,

    pub offset: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,
}

impl Object {
    pub fn from_stl(data: Vec<u8>) -> std::io::Result<Self> {
        stl::read_stl(&mut Cursor::new(data)).map(|stl| Self {
            triangles: stl
                .triangles
                .into_iter()
                .map(|t| Triangle {
                    normal: Vector3::from_data(ArrayStorage([t.normal])),
                    v1: Vector3::from_data(ArrayStorage([t.v1])),
                    v2: Vector3::from_data(ArrayStorage([t.v2])),
                    v3: Vector3::from_data(ArrayStorage([t.v3])),
                })
                .collect(),
            offset: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(0.1, 0.1, 0.1),
            rotation: Vector3::new(0.0, 0.0, 0.0),
        })
    }

    pub fn verticies(&self) -> Vec<Vertex> {
        let mut verticies: Vec<Vertex> =
            Vec::with_capacity(std::mem::size_of::<Vertex>() * self.triangles.len() * 3);

        for triangle in &self.triangles {
            verticies.push(Vertex {
                pos: triangle.v1.into(),
                normal: triangle.normal.into(),
            });
            verticies.push(Vertex {
                pos: triangle.v2.into(),
                normal: triangle.normal.into(),
            });
            verticies.push(Vertex {
                pos: triangle.v3.into(),
                normal: triangle.normal.into(),
            });
        }

        verticies
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::new_translation(&self.offset);
        let scaling = Matrix4::new_nonuniform_scaling(&self.scale);
        let rotation =
            Matrix4::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z);

        (translation * scaling * rotation).transpose()
    }

    pub fn uniform(&self) -> ObjectUniformData {
        ObjectUniformData {
            proj: self.calc_matrix().into(),
        }
    }
}
