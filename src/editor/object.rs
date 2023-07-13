use std::{io::Cursor, sync::Arc};

use eframe::wgpu;
use egui::vec2;
use nalgebra::{ArrayStorage, Matrix4, Vector2, Vector3};
use stl;

use super::state;

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
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 4 * 6,
            shader_location: 2,
        },
    ],
    step_mode: wgpu::VertexStepMode::Vertex,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex {
    pos: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3],
}

pub struct Triangle {
    pub normal: Vector3<f32>,
    pub v1: Vector3<f32>,
    pub v2: Vector3<f32>,
    pub v3: Vector3<f32>,
}

impl Triangle {
    pub fn append_verticies(&self, verticies: &mut Vec<Vertex>, color: [f32; 3]) {
        verticies.push(Vertex {
            pos: self.v1.into(),
            normal: self.normal.into(),
            color,
        });
        verticies.push(Vertex {
            pos: self.v2.into(),
            normal: self.normal.into(),
            color,
        });
        verticies.push(Vertex {
            pos: self.v3.into(),
            normal: self.normal.into(),
            color,
        });
    }
}

pub struct Object {
    pub triangles: Vec<Triangle>,

    pub color: [f32; 3],

    pub name: String,
    pub id: u32,
}

impl Object {
    pub fn from_stl(name: String, data: Vec<u8>, id_counter: &mut u32) -> std::io::Result<Self> {
        stl::read_stl(&mut Cursor::new(data)).map(|stl| {
            *id_counter += 1;

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
                color: [0.7, 0.7, 0.7],
                id: *id_counter,
                name,
            };

            // Convert to KeloCAM Units
            object.scale(&Vector3::from_element(0.1));

            // Move object to center. TODO: find free space for object
            let (min, max) = object.inf_sup();
            let delta = Vector3::new(
                -min.x - (max.x - min.x) / 2.0,
                -min.y - (max.y - min.y) / 2.0,
                -min.z,
            );
            object.translate(&delta);

            object
        })
    }

    pub fn verticies(&self) -> Vec<Vertex> {
        let mut verticies: Vec<Vertex> =
            Vec::with_capacity(std::mem::size_of::<Vertex>() * self.triangles.len() * 3);

        for triangle in &self.triangles {
            triangle.append_verticies(&mut verticies, self.color);
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

    pub fn translate(&mut self, delta: &Vector3<f32>) {
        for triangle in self.triangles.iter_mut() {
            triangle.v1 += delta;
            triangle.v2 += delta;
            triangle.v3 += delta;
        }
    }

    pub fn scale(&mut self, delta: &Vector3<f32>) {
        for triangle in self.triangles.iter_mut() {
            triangle.v1.component_mul_assign(delta);
            triangle.v2.component_mul_assign(delta);
            triangle.v3.component_mul_assign(delta);
        }
    }

    pub fn rotate(&mut self, delta: &Vector3<f32>) {
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

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut state::State,
        messages: &mut Vec<state::Message>,
    ) {
        ui.horizontal(|ui| {
            ui.add(egui::Image::new(&state.object_icon, vec2(16.0, 16.0)));

            let response =
                ui.selectable_label(state.selected.contains(&self.id), self.name.as_str());

            if response.clicked() {
                if !ui.input(|i| i.modifiers.contains(egui::Modifiers::SHIFT)) {
                    state.selected.clear();
                }

                if state.selected.contains(&self.id) {
                    state.selected.remove(&self.id);
                } else {
                    state.selected.insert(self.id);
                }
            }

            response.context_menu(|ui| {
                if ui.button("Delete").clicked() {
                    ui.close_menu();
                    messages.push(state::Message::Delete(self.id));
                }
            });
        });
    }
}

pub struct Renderer {
    pub vertex_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(
        device: &Arc<wgpu::Device>,
        format: wgpu::TextureFormat,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("object"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: 100000 * VERTEX_SIZE as u64,
            mapped_at_creation: false,
        });

        let color_target = wgpu::ColorTargetState {
            format,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("object"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/object.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("grid"),
            bind_group_layouts: &[camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("object"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VERTEX_BUFFER_LAYOUT],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(color_target)],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            vertex_buffer,
            pipeline,
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, verticies: u32) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(
            0,
            self.vertex_buffer
                .slice(0..verticies as u64 * VERTEX_SIZE as u64),
        );
        render_pass.draw(0..verticies, 0..1);
    }
}
