use std::sync::Arc;

use nalgebra::Vector3;

use eframe::wgpu;

pub type Index = u32;
pub const INDEX_SIZE: usize = std::mem::size_of::<Index>();

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
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 4 * 9,
            shader_location: 3,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32,
            offset: 4 * 13,
            shader_location: 4,
        },
    ],
    step_mode: wgpu::VertexStepMode::Vertex,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex {
    before: [f32; 3],
    pos: [f32; 3],
    after: [f32; 3],
    color: [f32; 4],
    thickness: f32,
}

/// Generates verticies for a closed path.
pub fn generate_closed(
    points: &[Vector3<f32>],
    color: [f32; 4],
    thickness: f32,
    verticies: &mut Vec<Vertex>,
    indicies: &mut Vec<Index>,
) {
    let len = points.len() as i32;
    let base = verticies.len() as Index;

    for i in 0..len {
        let before = points[(i - 1).rem_euclid(len) as usize];
        let pos = points[i as usize];
        let after_i = (i + 1).rem_euclid(len) as usize;
        let after = points[after_i];

        verticies.push(Vertex {
            before: before.into(),
            pos: pos.into(),
            after: after.into(),
            color,
            thickness,
        });
        verticies.push(Vertex {
            before: after.into(),
            pos: pos.into(),
            after: before.into(),
            color,
            thickness,
        });

        verticies.push(Vertex {
            before: before.into(),
            pos: pos.into(),
            after: after.into(),
            color,
            thickness,
        });
        verticies.push(Vertex {
            before: after.into(),
            pos: pos.into(),
            after: before.into(),
            color,
            thickness,
        });

        indicies.push(base + 4 * i as Index + 3);
        indicies.push(base + 4 * after_i as Index);
        indicies.push(base + 4 * after_i as Index + 1);

        indicies.push(base + 4 * after_i as Index);
        indicies.push(base + 4 * i as Index + 3);
        indicies.push(base + 4 * i as Index + 2);

        indicies.push(base + 4 * i as Index + 2);
        indicies.push(base + 4 * i as Index + 1);
        indicies.push(base + 4 * i as Index);
        indicies.push(base + 4 * i as Index + 1);
        indicies.push(base + 4 * i as Index + 2);
        indicies.push(base + 4 * i as Index + 3);
    }
}

/// Generates verticies for an open path.
pub fn generate_open(
    points: &[Vector3<f32>],
    color: [f32; 4],
    thickness: f32,
    verticies: &mut Vec<Vertex>,
    indicies: &mut Vec<Index>,
) {
    let len = points.len() as i32;

    for i in 0..len {
        let base = verticies.len() as Index;

        let pos = points[i as usize];
        let after = if i == len - 1 {
            pos + (pos - points[i as usize - 1])
        } else {
            points[i as usize + 1]
        };
        let before = if i == 0 {
            pos + (pos - points[i as usize + 1])
        } else {
            points[i as usize - 1]
        };

        verticies.push(Vertex {
            before: before.into(),
            pos: pos.into(),
            after: after.into(),
            color,
            thickness,
        });
        verticies.push(Vertex {
            before: after.into(),
            pos: pos.into(),
            after: before.into(),
            color,
            thickness,
        });

        verticies.push(Vertex {
            before: before.into(),
            pos: pos.into(),
            after: after.into(),
            color,
            thickness,
        });
        verticies.push(Vertex {
            before: after.into(),
            pos: pos.into(),
            after: before.into(),
            color,
            thickness,
        });

        if i == 0 {
            indicies.push(base + 3);
            indicies.push(base + 4);
            indicies.push(base + 5);
            indicies.push(base + 4);
            indicies.push(base + 3);
            indicies.push(base + 2);
        } else if i != len - 1 {
            indicies.push(base + 3);
            indicies.push(base + 4);
            indicies.push(base + 5);
            indicies.push(base + 4);
            indicies.push(base + 3);
            indicies.push(base + 2);

            indicies.push(base + 2);
            indicies.push(base + 1);
            indicies.push(base);
            indicies.push(base + 1);
            indicies.push(base + 2);
            indicies.push(base + 3);
        }
    }
}

pub struct Renderer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(
        device: &Arc<wgpu::Device>,
        format: wgpu::TextureFormat,
        depth_enabled: bool,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("path"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: 20000 * VERTEX_SIZE as u64,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("path"),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            size: 40000 * std::mem::size_of::<u32>() as u64,
            mapped_at_creation: false,
        });

        let color_target = wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("path"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/path.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("path"),
            bind_group_layouts: &[camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("path"),
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
                depth_compare: if depth_enabled {
                    wgpu::CompareFunction::Less
                } else {
                    wgpu::CompareFunction::Always
                },
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            index_buffer,
            vertex_buffer,
            pipeline,
        }
    }

    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        vertex_count: u32,
        index_count: u32,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_index_buffer(
            self.index_buffer
                .slice(0..index_count as u64 * std::mem::size_of::<u32>() as u64),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.set_vertex_buffer(
            0,
            self.vertex_buffer
                .slice(0..vertex_count as u64 * VERTEX_SIZE as u64),
        );
        render_pass.draw_indexed(0..index_count, 0, 0..1);
    }
}
