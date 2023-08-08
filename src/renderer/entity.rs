use eframe::wgpu;
use nalgebra::{UnitVector3, Vector3};
use std::sync::Arc;

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
            format: wgpu::VertexFormat::Float32x4,
            offset: 4 * 3,
            shader_location: 1,
        },
    ],
    step_mode: wgpu::VertexStepMode::Vertex,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex {
    pos: [f32; 3],
    color: [f32; 4],
}

/// Generates a cube entity
#[rustfmt::skip]
pub fn generate_cube(
    scale: f32,
    origin: &Vector3<f32>,
    color: [f32; 4],
    verticies: &mut Vec<Vertex>,
) {
    let a = Vector3::new(scale * 0.5, scale * 0.5, scale * 0.5);
    let b = Vector3::new(scale * 0.5, -scale * 0.5, scale * 0.5);
    let c = Vector3::new(-scale * 0.5, -scale * 0.5, scale * 0.5);
    let d = Vector3::new(-scale * 0.5, scale * 0.5, scale * 0.5);

    verticies.push(Vertex { pos: (origin + a).into(), color });
    verticies.push(Vertex { pos: (origin + c).into(), color });
    verticies.push(Vertex { pos: (origin + d).into(), color });
    verticies.push(Vertex { pos: (origin + c).into(), color });
    verticies.push(Vertex { pos: (origin + a).into(), color });
    verticies.push(Vertex { pos: (origin + b).into(), color });
    
    verticies.push(Vertex { pos: (origin - a).into(), color });
    verticies.push(Vertex { pos: (origin - c).into(), color });
    verticies.push(Vertex { pos: (origin - d).into(), color });
    verticies.push(Vertex { pos: (origin - c).into(), color });
    verticies.push(Vertex { pos: (origin - a).into(), color });
    verticies.push(Vertex { pos: (origin - b).into(), color });
    
    verticies.push(Vertex { pos: (origin + b).into(), color });
    verticies.push(Vertex { pos: (origin + a).into(), color });
    verticies.push(Vertex { pos: (origin - d).into(), color });
    verticies.push(Vertex { pos: (origin - d).into(), color });
    verticies.push(Vertex { pos: (origin + a).into(), color });
    verticies.push(Vertex { pos: (origin - c).into(), color });
    
    verticies.push(Vertex { pos: (origin - b).into(), color });
    verticies.push(Vertex { pos: (origin - a).into(), color });
    verticies.push(Vertex { pos: (origin + d).into(), color });
    verticies.push(Vertex { pos: (origin + d).into(), color });
    verticies.push(Vertex { pos: (origin - a).into(), color });
    verticies.push(Vertex { pos: (origin + c).into(), color });
    
    verticies.push(Vertex { pos: (origin + a).into(), color });
    verticies.push(Vertex { pos: (origin + d).into(), color });
    verticies.push(Vertex { pos: (origin - b).into(), color });
    verticies.push(Vertex { pos: (origin - b).into(), color });
    verticies.push(Vertex { pos: (origin - c).into(), color });
    verticies.push(Vertex { pos: (origin + a).into(), color });
    
    verticies.push(Vertex { pos: (origin - a).into(), color });
    verticies.push(Vertex { pos: (origin - d).into(), color });
    verticies.push(Vertex { pos: (origin + b).into(), color });
    verticies.push(Vertex { pos: (origin + b).into(), color });
    verticies.push(Vertex { pos: (origin + c).into(), color });
    verticies.push(Vertex { pos: (origin - a).into(), color });
}

/// Generates an arrow entity
#[rustfmt::skip]
pub fn generate_arrow(
    scale: f32,
    origin: &Vector3<f32>,
    direction: &UnitVector3<f32>,
    color: [f32; 4],
    verticies: &mut Vec<Vertex>,
) {
    let mut na = direction.cross(&Vector3::new(-direction.z, direction.x, direction.y));
    na.normalize_mut();
    let mut nb = direction.cross(&na);
    nb.normalize_mut();

    na.scale_mut(scale / 2.0);
    nb.scale_mut(scale / 2.0);
    let nc = direction.scale(scale);

    verticies.push(Vertex { pos: (origin + na).into(), color });
    verticies.push(Vertex { pos: (origin + nb).into(), color });
    verticies.push(Vertex { pos: (origin + nc).into(), color });
    
    verticies.push(Vertex { pos: (origin + nb).into(), color });
    verticies.push(Vertex { pos: (origin - na).into(), color });
    verticies.push(Vertex { pos: (origin + nc).into(), color });
    
    verticies.push(Vertex { pos: (origin - na).into(), color });
    verticies.push(Vertex { pos: (origin - nb).into(), color });
    verticies.push(Vertex { pos: (origin + nc).into(), color });
    
    verticies.push(Vertex { pos: (origin - nb).into(), color });
    verticies.push(Vertex { pos: (origin + na).into(), color });
    verticies.push(Vertex { pos: (origin + nc).into(), color });
}

pub struct Renderer {
    pub vertex_buffer: wgpu::Buffer,
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
            label: Some("entity"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: 100000 * VERTEX_SIZE as u64,
            mapped_at_creation: false,
        });

        let color_target = wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("entity"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/entity.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("entity"),
            bind_group_layouts: &[camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("entity"),
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
