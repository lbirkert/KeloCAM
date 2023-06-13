use std::io::Cursor;

use eframe::wgpu;
use nalgebra::{ArrayStorage, Matrix4, Vector3};
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

pub struct Object {
    pub triangles: Vec<Triangle>,

    pub translation: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,

    pub name: Option<String>,
}

impl Object {
    pub fn from_stl(data: Vec<u8>) -> std::io::Result<Self> {
        stl::read_stl(&mut Cursor::new(data)).map(|stl| {
            let mut object = Self {
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
                translation: Vector3::new(0.0, 0.0, 0.0),
                rotation: Vector3::new(0.0, 0.0, 0.0),
                scale: Vector3::new(1.0, 1.0, 1.0),
                name: None,
            };

            // A unit in KeloCAM is 1CM not 1MM
            object.scale(Vector3::new(0.1, 0.1, 0.1));

            let (min, max) = object.bounding_box();

            let delta = Vector3::new(
                -min.x - (max.x - min.x) / 2.0,
                -min.y - (max.y - min.y) / 2.0,
                -min.z,
            );

            object.translate(delta);

            // Reset scale and translation to 1
            object.scale = Vector3::from_element(1.0);
            object.translation = Vector3::from_element(0.0);

            object
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

    pub fn translate(&mut self, translation: Vector3<f32>) {
        let delta = translation - self.translation;

        for triangle in self.triangles.iter_mut() {
            triangle.v1 += delta;
            triangle.v2 += delta;
            triangle.v3 += delta;
        }

        self.translation = translation;
    }

    pub fn scale(&mut self, scale: Vector3<f32>) {
        let delta = scale.component_div(&self.scale);

        for triangle in self.triangles.iter_mut() {
            triangle.v1.component_mul_assign(&delta);
            triangle.v2.component_mul_assign(&delta);
            triangle.v3.component_mul_assign(&delta);
        }

        self.scale = scale;
    }

    pub fn rotate(&mut self, rotation: Vector3<f32>) {
        let old_inverse =
            Matrix4::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z)
                .try_inverse()
                .unwrap();
        let new = Matrix4::from_euler_angles(rotation.x, rotation.y, rotation.z);

        let delta = new * old_inverse;

        for triangle in self.triangles.iter_mut() {
            triangle.v1 = delta.transform_vector(&triangle.v1);
            triangle.v2 = delta.transform_vector(&triangle.v2);
            triangle.v3 = delta.transform_vector(&triangle.v3);
            triangle.normal = delta.transform_vector(&triangle.normal);
        }

        self.rotation = rotation;
    }

    /// Returns the bounding box of the object as min and max vector
    pub fn bounding_box(&self) -> (Vector3<f32>, Vector3<f32>) {
        let mut min = Vector3::from_element(std::f32::INFINITY);
        let mut max = Vector3::from_element(std::f32::NEG_INFINITY);

        for triangle in self.triangles.iter() {
            min = min.inf(&triangle.v1.inf(&triangle.v2.inf(&triangle.v3)));
            max = max.sup(&triangle.v1.sup(&triangle.v2.sup(&triangle.v3)));
        }

        (min, max)
    }
}
