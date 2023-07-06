use eframe::wgpu;
use nalgebra::Vector3;
use std::sync::Arc;

pub struct Arrow {
    pub origin: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub color: [f32; 3],
    pub scale: f32,
}

impl Arrow {
    #[rustfmt::skip]
    pub fn verticies(&self) -> Vec<Vertex> {
        let normal = self.normal.normalize();
        let a = normal.cross(&normal.zxy()).normalize();
        let b = normal.cross(&a);

        let a = a.scale(self.scale / 2.0);
        let b = b.scale(self.scale / 2.0);
        let c = normal.scale(self.scale);

        vec![
            // Tri 1
            Vertex { pos: (self.origin + a).into(), color: self.color },
            Vertex { pos: (self.origin - a).into(), color: self.color },
            Vertex { pos: (self.origin - b).into(), color: self.color },
            // Tri 2
            Vertex { pos: (self.origin + a).into(), color: self.color },
            Vertex { pos: (self.origin + b).into(), color: self.color },
            Vertex { pos: (self.origin - a).into(), color: self.color },
            // Tri 3
            Vertex { pos: (self.origin + a).into(), color: self.color },
            Vertex { pos: (self.origin + b).into(), color: self.color },
            Vertex { pos: (self.origin + c).into(), color: self.color },
            // Tri 4
            Vertex { pos: (self.origin + b).into(), color: self.color },
            Vertex { pos: (self.origin - a).into(), color: self.color },
            Vertex { pos: (self.origin + c).into(), color: self.color },
            // Tri 5
            Vertex { pos: (self.origin - a).into(), color: self.color },
            Vertex { pos: (self.origin - b).into(), color: self.color },
            Vertex { pos: (self.origin + c).into(), color: self.color },
            // Tri 6
            Vertex { pos: (self.origin - b).into(), color: self.color },
            Vertex { pos: (self.origin + a).into(), color: self.color },
            Vertex { pos: (self.origin + c).into(), color: self.color },
        ]
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
            size: 100000 * VERTEX_SIZE as u64,
            mapped_at_creation: false,
        });

        let color_target = wgpu::ColorTargetState {
            format,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("arrow"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/arrow.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("arrow"),
            bind_group_layouts: &[camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("arrow"),
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
