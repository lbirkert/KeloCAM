use eframe::wgpu;
use nalgebra::{UnitVector3, Vector3};
use std::sync::Arc;

pub enum Tool {
    Move { origin: Vector3<f32> },
    Scale { origin: Vector3<f32> },
    Rotate { origin: Vector3<f32> },
}

impl Tool {
    pub fn verticies(&self, scale: f32) -> Vec<Vertex> {
        match self {
            Tool::Move { origin } => {
                let mut verticies = Vec::new();
                Self::move_tool(&mut verticies, origin, scale);
                verticies
            }
            _ => panic!(),
        }
    }

    #[rustfmt::skip]
    fn triangle(
        verticies: &mut Vec<Vertex>,
        offset: &Vector3<f32>,
        scale: f32,
        a: Vector3<f32>,
        b: Vector3<f32>,
        c: Vector3<f32>,
        color: [f32; 3],
    ) {
        verticies.push(Vertex { pos: (offset + a.scale(scale)).into(), color });
        verticies.push(Vertex { pos: (offset + b.scale(scale)).into(), color });
        verticies.push(Vertex { pos: (offset + c.scale(scale)).into(), color });
    }

    #[rustfmt::skip]
    fn arrow(verticies: &mut Vec<Vertex>, offset: &Vector3<f32>, normal: UnitVector3<f32>, scale: f32, color: [f32; 3]) {
        let a = normal.cross(&normal.zxy()).normalize();
        let b = normal.cross(&a);
        let c = normal.scale(2.0);

        Self::triangle(verticies, offset, scale / 2.0, a, -a, -b, color);
        Self::triangle(verticies, offset, scale / 2.0, a, b, -a, color);
        Self::triangle(verticies, offset, scale / 2.0, a, b, c, color);
        Self::triangle(verticies, offset, scale / 2.0, b, -a, c, color);
        Self::triangle(verticies, offset, scale / 2.0, -a, -b, c, color);
        Self::triangle(verticies, offset, scale / 2.0, -b, a, c, color);
    }

    #[rustfmt::skip]
    fn move_tool(verticies: &mut Vec<Vertex>, offset: &Vector3<f32>, scale: f32) {
        // X axis
        Self::triangle(verticies, offset, scale, Vector3::new(0.0, -0.1,  0.0), Vector3::new(5.0, -0.1,  0.0), Vector3::new(5.0, 0.1, 0.0), [1.0, 0.0, 0.0]);
        Self::triangle(verticies, offset, scale, Vector3::new(0.0, -0.1,  0.0), Vector3::new(5.0,  0.1,  0.0), Vector3::new(0.0, 0.1, 0.0), [1.0, 0.0, 0.0]);
        Self::triangle(verticies, offset, scale, Vector3::new(0.0,  0.0, -0.1), Vector3::new(5.0,  0.0, -0.1), Vector3::new(5.0, 0.0, 0.1), [1.0, 0.0, 0.0]);
        Self::triangle(verticies, offset, scale, Vector3::new(0.0,  0.0, -0.1), Vector3::new(5.0,  0.0,  0.1), Vector3::new(0.0, 0.0, 0.1), [1.0, 0.0, 0.0]);
        // Y axis                         
        Self::triangle(verticies, offset, scale, Vector3::new(-0.1, 0.0,  0.0), Vector3::new(-0.1, 5.0,  0.0), Vector3::new(0.1, 5.0, 0.0), [0.0, 1.0, 0.0]);
        Self::triangle(verticies, offset, scale, Vector3::new(-0.1, 0.0,  0.0), Vector3::new( 0.1, 5.0,  0.0), Vector3::new(0.1, 0.0, 0.0), [0.0, 1.0, 0.0]);
        Self::triangle(verticies, offset, scale, Vector3::new( 0.0, 0.0, -0.1), Vector3::new( 0.0, 5.0, -0.1), Vector3::new(0.0, 5.0, 0.1), [0.0, 1.0, 0.0]);
        Self::triangle(verticies, offset, scale, Vector3::new( 0.0, 0.0, -0.1), Vector3::new( 0.0, 5.0,  0.1), Vector3::new(0.0, 0.0, 0.1), [0.0, 1.0, 0.0]);
        // Z axis                         
        Self::triangle(verticies, offset, scale, Vector3::new(-0.1,  0.0, 0.0), Vector3::new(-0.1,  0.0, 5.0), Vector3::new(0.1, 0.0, 5.0), [0.0, 0.0, 1.0]);
        Self::triangle(verticies, offset, scale, Vector3::new(-0.1,  0.0, 0.0), Vector3::new( 0.1,  0.0, 5.0), Vector3::new(0.1, 0.0, 0.0), [0.0, 0.0, 1.0]);
        Self::triangle(verticies, offset, scale, Vector3::new( 0.0, -0.1, 0.0), Vector3::new( 0.0, -0.1, 5.0), Vector3::new(0.0, 0.1, 5.0), [0.0, 0.0, 1.0]);
        Self::triangle(verticies, offset, scale, Vector3::new( 0.0, -0.1, 0.0), Vector3::new( 0.0,  0.1, 5.0), Vector3::new(0.0, 0.1, 0.0), [0.0, 0.0, 1.0]);
        // Arrows added at last so they get drawn over everything else
        Self::arrow(verticies, &(offset + Vector3::new(scale * 5.0, 0.0, 0.0)), Vector3::x_axis(), 0.5 * scale, [1.0, 0.0, 0.0]);
        Self::arrow(verticies, &(offset + Vector3::new(0.0, scale * 5.0, 0.0)), Vector3::y_axis(), 0.5 * scale, [0.0, 1.0, 0.0]);
        Self::arrow(verticies, &(offset + Vector3::new(0.0, 0.0, scale * 5.0)), Vector3::z_axis(), 0.5 * scale, [0.0, 0.0, 1.0]);
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
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

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
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
            size: 1000 * VERTEX_SIZE as u64,
            mapped_at_creation: false,
        });

        let color_target = wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("tool"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/tool.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("tool"),
            bind_group_layouts: &[camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("tool"),
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
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            vertex_buffer,
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
