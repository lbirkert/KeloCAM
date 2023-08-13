use std::sync::Arc;

use eframe::wgpu;

use kelocam_core::primitives::Triangle;

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
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex {
    pos: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3],
}

/// Generate the verticies for this object.
pub fn generate(triangles: &[Triangle], color: [f32; 3], verticies: &mut Vec<Vertex>) {
    for triangle in triangles.iter() {
        let normal = triangle.normal.into_inner();
        verticies.push(Vertex {
            pos: triangle.a.into(),
            normal: normal.into(),
            color,
        });
        verticies.push(Vertex {
            pos: triangle.b.into(),
            normal: normal.into(),
            color,
        });
        verticies.push(Vertex {
            pos: triangle.c.into(),
            normal: normal.into(),
            color,
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
            label: Some("object"),
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
            primitive: wgpu::PrimitiveState {
                //cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
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
