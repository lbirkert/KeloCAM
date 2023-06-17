use std::io::Cursor;

use eframe::wgpu;
use nalgebra::{ArrayStorage, Matrix4, Vector2, Vector3};
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
                name: None,
            };

            let (min, max) = object.inf_sup();

            let delta = Vector3::new(
                -min.x - (max.x - min.x) / 2.0,
                -min.y - (max.y - min.y) / 2.0,
                -min.z,
            );

            // Move object to center. TODO: find free space for object
            object.translate(delta);
            // Convert to KeloCAM Units
            object.scale(Vector3::from_element(0.1));

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

    /// Z-Slice the model. TODO: Also slice on other axies
    pub fn slice(&self, z: f32) -> Vec<(Vector2<f32>, Vector2<f32>)> {
        let mut lines = Vec::new();

        for triangle in self.triangles.iter() {
            let mut points: Vec<Vector2<f32>> = Vec::with_capacity(2);

            let a = triangle.v1;
            let b = triangle.v2;
            let c = triangle.v3;

            if (a.z > z) != (b.z > z) {
                points.push(a.xy().lerp(&b.xy(), (z - b.z) / (a.z - b.z)));
            }

            if (b.z > z) != (c.z > z) {
                points.push(b.xy().lerp(&c.xy(), (z - c.z) / (b.z - c.z)));
            }

            if (c.z > z) != (a.z > z) {
                points.push(c.xy().lerp(&a.xy(), (z - a.z) / (c.z - a.z)));
            }

            if let Some(p1) = points.pop() {
                if let Some(p2) = points.pop() {
                    // TODO: Add normal vector of line
                    lines.push((p1, p2));
                }
            }
        }

        lines
    }

    pub fn translate(&mut self, delta: Vector3<f32>) {
        for triangle in self.triangles.iter_mut() {
            triangle.v1 += delta;
            triangle.v2 += delta;
            triangle.v3 += delta;
        }
    }

    pub fn scale(&mut self, delta: Vector3<f32>) {
        for triangle in self.triangles.iter_mut() {
            triangle.v1.component_mul_assign(&delta);
            triangle.v2.component_mul_assign(&delta);
            triangle.v3.component_mul_assign(&delta);
        }
    }

    pub fn rotate(&mut self, delta: Vector3<f32>) {
        let delta = Matrix4::from_euler_angles(delta.x, delta.y, delta.z);

        for triangle in self.triangles.iter_mut() {
            triangle.v1 = delta.transform_vector(&triangle.v1);
            triangle.v2 = delta.transform_vector(&triangle.v2);
            triangle.v3 = delta.transform_vector(&triangle.v3);
            triangle.normal = delta.transform_vector(&triangle.normal);
        }
    }

    pub fn inf_sup(&self) -> (Vector3<f32>, Vector3<f32>) {
        let mut inf = Vector3::from_element(std::f32::INFINITY);
        let mut sup = Vector3::from_element(std::f32::NEG_INFINITY);

        for triangle in self.triangles.iter() {
            inf = inf.inf(&triangle.v1.inf(&triangle.v2.inf(&triangle.v3)));
            sup = sup.sup(&triangle.v1.sup(&triangle.v2.sup(&triangle.v3)));
        }

        (inf, sup)
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Some Object");
    }
}
