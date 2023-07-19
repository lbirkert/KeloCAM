use super::path;
use eframe::wgpu;
use nalgebra::{Unit, Vector3};
use std::sync::Arc;

use super::{
    object::{self, Transformation},
    ray::Ray,
};

#[derive(Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn vector(&self) -> Unit<Vector3<f32>> {
        match self {
            Axis::X => Vector3::x_axis(),
            Axis::Y => Vector3::y_axis(),
            Axis::Z => Vector3::z_axis(),
        }
    }
}

#[derive(Debug)]
pub enum Action {
    Transform { axis: Axis },
    Hover { axis: Axis },
}

pub enum Tool {
    Move,
    Scale,
    Rotate,
}

impl Tool {
    pub fn selection_transformation(&self) -> Transformation {
        match self {
            Tool::Move => object::Transformation::Translate {
                translate: Vector3::zeros(),
            },
            Tool::Scale => object::Transformation::Scale {
                scale: Vector3::from_element(1.0),
            },
            Tool::Rotate => object::Transformation::Rotate {
                rotate: Vector3::zeros(),
            },
        }
    }

    pub fn verticies(
        &self,
        verticies: &mut Vec<Vertex>,
        path_indicies: &mut Vec<path::Index>,
        path_verticies: &mut Vec<path::Vertex>,

        offset: &Vector3<f32>,
        scale: f32,
        action: &Option<Action>,
    ) {
        match self {
            Tool::Move => {
                Self::move_tool(
                    verticies,
                    path_indicies,
                    path_verticies,
                    offset,
                    scale,
                    action,
                );
            }
            Tool::Scale => {
                Self::scale_tool(
                    verticies,
                    path_indicies,
                    path_verticies,
                    offset,
                    scale,
                    action,
                );
            }
            _ => panic!(),
        }
    }

    pub fn triangle(
        verticies: &mut Vec<Vertex>,
        origin: &Vector3<f32>,
        scale: f32,
        a: Vector3<f32>,
        b: Vector3<f32>,
        c: Vector3<f32>,
        color: [f32; 3],
    ) {
        verticies.push(Vertex {
            pos: (origin + a.scale(scale)).into(),
            color,
        });
        verticies.push(Vertex {
            pos: (origin + b.scale(scale)).into(),
            color,
        });
        verticies.push(Vertex {
            pos: (origin + c.scale(scale)).into(),
            color,
        });
    }

    pub fn cube(verticies: &mut Vec<Vertex>, origin: &Vector3<f32>, scale: f32, color: [f32; 3]) {
        let a = Vector3::new(1.0, 1.0, 1.0);
        let b = Vector3::new(-1.0, 1.0, 1.0);
        let c = Vector3::new(-1.0, -1.0, 1.0);
        let d = Vector3::new(1.0, -1.0, 1.0);

        Self::triangle(verticies, origin, scale / 2.0, a, c, b, color);
        Self::triangle(verticies, origin, scale / 2.0, a, d, c, color);
        Self::triangle(verticies, origin, scale / 2.0, -a, -c, -b, color);
        Self::triangle(verticies, origin, scale / 2.0, -a, -d, -c, color);

        Self::triangle(verticies, origin, scale / 2.0, -c, d, a, color);
        Self::triangle(verticies, origin, scale / 2.0, -c, -b, d, color);
        Self::triangle(verticies, origin, scale / 2.0, c, -d, -a, color);
        Self::triangle(verticies, origin, scale / 2.0, c, b, -d, color);

        Self::triangle(verticies, origin, scale / 2.0, -c, b, -d, color);
        Self::triangle(verticies, origin, scale / 2.0, -c, a, b, color);
        Self::triangle(verticies, origin, scale / 2.0, c, -b, d, color);
        Self::triangle(verticies, origin, scale / 2.0, c, -a, -b, color);
    }

    pub fn arrow(
        verticies: &mut Vec<Vertex>,
        origin: &Vector3<f32>,
        normal: &Vector3<f32>,
        scale: f32,
        color: [f32; 3],
    ) {
        let a = normal.cross(&normal.zxy()).normalize();
        let b = normal.cross(&a);
        let c = normal.scale(2.0);

        Self::triangle(verticies, origin, scale, a, -a, -b, color);
        Self::triangle(verticies, origin, scale, a, b, -a, color);
        Self::triangle(verticies, origin, scale, a, b, c, color);
        Self::triangle(verticies, origin, scale, b, -a, c, color);
        Self::triangle(verticies, origin, scale, -a, -b, c, color);
        Self::triangle(verticies, origin, scale, -b, a, c, color);
    }

    // TODO: rework this
    fn move_tool(
        verticies: &mut Vec<Vertex>,
        path_indicies: &mut Vec<path::Index>,
        path_verticies: &mut Vec<path::Vertex>,
        origin: &Vector3<f32>,
        scale: f32,
        action: &Option<Action>,
    ) {
        let mut xcolor = [1.0, 0.0, 0.0];
        let mut ycolor = [0.0, 1.0, 0.0];
        let mut zcolor = [0.0, 0.0, 1.0];

        match action {
            Some(Action::Hover { axis }) | Some(Action::Transform { axis }) => match axis {
                Axis::X => xcolor = [1.0, 0.7, 0.7],
                Axis::Y => ycolor = [0.7, 1.0, 0.7],
                Axis::Z => zcolor = [0.7, 0.7, 1.0],
            },
            _ => {}
        };

        // X axis
        Self::arrow(
            verticies,
            &(origin + Vector3::new(scale * 5.0, 0.0, 0.0)),
            &Vector3::new(1.0, 0.0, 0.0),
            0.5 * scale,
            xcolor,
        );
        path::Path {
            points: vec![*origin, origin + Vector3::new(5.0 * scale, 0.0, 0.0)],
            closed: false,
            color: xcolor,
            thickness: 5.0,
        }
        .append(path_verticies, path_indicies);

        // Y axis
        Self::arrow(
            verticies,
            &(origin + Vector3::new(0.0, scale * 5.0, 0.0)),
            &Vector3::new(0.0, 1.0, 0.0),
            0.5 * scale,
            ycolor,
        );
        path::Path {
            points: vec![*origin, origin + Vector3::new(0.0, 5.0 * scale, 0.0)],
            closed: false,
            color: ycolor,
            thickness: 5.0,
        }
        .append(path_verticies, path_indicies);

        // Z axis
        Self::arrow(
            verticies,
            &(origin + Vector3::new(0.0, 0.0, scale * 5.0)),
            &Vector3::new(0.0, 0.0, 1.0),
            0.5 * scale,
            zcolor,
        );
        path::Path {
            points: vec![*origin, origin + Vector3::new(0.0, 0.0, 5.0 * scale)],
            closed: false,
            color: zcolor,
            thickness: 5.0,
        }
        .append(path_verticies, path_indicies);
    }

    fn scale_tool(
        verticies: &mut Vec<Vertex>,
        path_indicies: &mut Vec<path::Index>,
        path_verticies: &mut Vec<path::Vertex>,
        origin: &Vector3<f32>,
        scale: f32,
        action: &Option<Action>,
    ) {
        let mut xcolor = [1.0, 0.0, 0.0];
        let mut ycolor = [0.0, 1.0, 0.0];
        let mut zcolor = [0.0, 0.0, 1.0];

        match action {
            Some(Action::Hover { axis }) | Some(Action::Transform { axis }) => match axis {
                Axis::X => xcolor = [1.0, 0.7, 0.7],
                Axis::Y => ycolor = [0.7, 1.0, 0.7],
                Axis::Z => zcolor = [0.7, 0.7, 1.0],
            },
            _ => {}
        };

        Self::cube(verticies, origin, scale, [1.0, 1.0, 1.0]);

        // X axis
        Self::cube(
            verticies,
            &(origin + Vector3::new(scale * 5.0, 0.0, 0.0)),
            scale,
            xcolor,
        );
        path::Path {
            points: vec![*origin, origin + Vector3::new(5.0 * scale, 0.0, 0.0)],
            closed: false,
            color: xcolor,
            thickness: 5.0,
        }
        .append(path_verticies, path_indicies);

        // Y axis
        Self::cube(
            verticies,
            &(origin + Vector3::new(0.0, scale * 5.0, 0.0)),
            scale,
            ycolor,
        );
        path::Path {
            points: vec![*origin, origin + Vector3::new(0.0, 5.0 * scale, 0.0)],
            closed: false,
            color: ycolor,
            thickness: 5.0,
        }
        .append(path_verticies, path_indicies);

        // Z axis
        Self::cube(
            verticies,
            &(origin + Vector3::new(0.0, 0.0, scale * 5.0)),
            scale,
            zcolor,
        );
        path::Path {
            points: vec![*origin, origin + Vector3::new(0.0, 0.0, 5.0 * scale)],
            closed: false,
            color: zcolor,
            thickness: 5.0,
        }
        .append(path_verticies, path_indicies);
    }

    pub fn msaxis(origin: &Vector3<f32>, camera_ray: &Ray, scale: f32) -> Option<Axis> {
        Self::intersect_axis(origin, camera_ray, scale, &Vector3::x_axis()).map_or_else(
            || {
                Self::intersect_axis(origin, camera_ray, scale, &Vector3::y_axis()).map_or_else(
                    || {
                        Self::intersect_axis(origin, camera_ray, scale, &Vector3::z_axis())
                            .map(|_| Axis::Z)
                    },
                    |_| Some(Axis::Y),
                )
            },
            |_| Some(Axis::X),
        )
    }

    fn intersect_axis(
        origin: &Vector3<f32>,
        camera_ray: &Ray,
        scale: f32,
        axis: &Vector3<f32>,
    ) -> Option<Vector3<f32>> {
        let eye_normal = camera_ray.origin - origin;
        let ortho = eye_normal.cross(axis).normalize().scale(scale);
        camera_ray.square_intersect(
            &(origin - ortho.scale(0.5)),
            &ortho,
            &axis.scale(scale * 6.0),
        )
    }

    pub fn intersect(&self, origin: &Vector3<f32>, camera_ray: &Ray, scale: f32) -> Option<Axis> {
        match self {
            Tool::Move | Tool::Scale => Self::msaxis(origin, camera_ray, scale),
            _ => panic!(),
        }
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
