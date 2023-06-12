use std::sync::Arc;

use eframe::{
    egui,
    egui_wgpu::{self, wgpu},
};

macro_rules! pipeline {
    ($device:tt, $name:tt, $target:expr, $bind:expr, $buffers:expr) => {{
        let shader = $device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some($name),
            source: wgpu::ShaderSource::Wgsl(
                include_str!(concat!("./shaders/", $name, ".wgsl")).into(),
            ),
        });

        let pipeline_layout = $device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some($name),
            bind_group_layouts: $bind,
            push_constant_ranges: &[],
        });

        $device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some($name),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: $buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some($target)],
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
        })
    }};
}

#[allow(unused_imports)]
use eframe::wgpu::util::DeviceExt;

mod camera;
use camera::{Camera, CameraUniform};
use nalgebra::{Matrix4, Vector3};

use crate::object::{self, Object};

const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;

pub struct Viewer {
    camera: Camera,

    pub objects: Vec<Object>,
    object_verticies: u32,

    pub object_changed: bool,
}

impl Viewer {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        let device = &wgpu_render_state.device;

        let camera = Camera::default();
        let camera_uniform = CameraUniform::new(device, camera.uniform());

        let blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                operation: wgpu::BlendOperation::Add,
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            },
            alpha: wgpu::BlendComponent {
                operation: wgpu::BlendOperation::Add,
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::Zero,
            },
        };

        let color_target = wgpu::ColorTargetState {
            format: wgpu_render_state.target_format,
            blend: Some(blend),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let object_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("object"),
            size: 600000,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let object_pipeline = pipeline!(
            device,
            "object",
            color_target.clone(),
            &[&camera_uniform.bind_group_layout],
            &[object::VERTEX_BUFFER_LAYOUT]
        );

        let grid_pipeline = pipeline!(
            device,
            "grid",
            color_target,
            &[&camera_uniform.bind_group_layout],
            &[]
        );

        wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(ViewerRenderResources {
                object_pipeline,
                grid_pipeline,
                camera_uniform,

                object_buffer,
            });

        Some(Self {
            camera,
            objects: vec![],
            object_verticies: 0,
            object_changed: false,
        })
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        /*egui::Frame::canvas(ui.style()).show(ui, |ui| {
            self.custom_painting(ui);
        });*/
        self.custom_painting(ui);
    }

    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();

        self.camera.resize(available_size.x, available_size.y);

        let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::drag());

        // Rotation
        if response.dragged_by(egui::PointerButton::Secondary) {
            self.camera.yaw += response.drag_delta().x * 0.005;
            self.camera.pitch += response.drag_delta().y * -0.005;

            if self.camera.pitch < -SAFE_FRAC_PI_2 {
                self.camera.pitch = -SAFE_FRAC_PI_2;
            } else if self.camera.pitch > SAFE_FRAC_PI_2 {
                self.camera.pitch = SAFE_FRAC_PI_2;
            }
        }

        // Translation
        if response.dragged_by(egui::PointerButton::Middle) {
            let delta = Matrix4::from_euler_angles(self.camera.pitch, self.camera.yaw, 0.0)
                .transform_vector(&Vector3::new(
                    response.drag_delta().x * 0.001 / self.camera.zoom,
                    response.drag_delta().y * 0.001 / self.camera.zoom,
                    0.0,
                ));

            self.camera.position += delta;
        }

        // Zoom
        if ui.rect_contains_pointer(rect) {
            ui.ctx().input(|i| {
                for event in &i.events {
                    if let egui::Event::Scroll(v) = event {
                        if v[0] == 0.0 {
                            if v[1] > 0.0 {
                                self.camera.zoom *= 1.0 + 0.001 * v[1];
                            } else if v[1] < 0.0 {
                                self.camera.zoom /= 1.0 + 0.001 * -v[1];
                            }
                        }
                    }
                }
            });
        }

        let uniform = self.camera.uniform();

        let object_buffer = if self.object_changed {
            let mut object_buffer: Vec<object::Vertex> = Vec::new();

            for object in &self.objects {
                object_buffer.append(&mut object.verticies());
            }

            self.object_verticies = object_buffer.len() as u32;
            self.object_changed = false;

            Some(object_buffer)
        } else {
            None
        };

        let object_verticies = self.object_verticies;

        let cb = egui_wgpu::CallbackFn::new()
            .prepare(move |_device, queue, _encoder, paint_callback_resources| {
                let resources: &ViewerRenderResources = paint_callback_resources.get().unwrap();

                resources.camera_uniform.update(queue, uniform);

                if let Some(ref object_buffer) = object_buffer {
                    queue.write_buffer(
                        &resources.object_buffer,
                        0,
                        bytemuck::cast_slice(object_buffer.as_slice()),
                    );
                }

                Vec::new()
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let resources: &ViewerRenderResources = paint_callback_resources.get().unwrap();
                resources.draw_grid(render_pass);

                if object_verticies > 0 {
                    resources.draw_object(render_pass, object_verticies);
                }
            });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };

        ui.painter().add(callback);
    }
}

pub struct ViewerRenderResources {
    grid_pipeline: wgpu::RenderPipeline,
    object_pipeline: wgpu::RenderPipeline,

    camera_uniform: CameraUniform,
    object_buffer: wgpu::Buffer,
}

impl ViewerRenderResources {
    fn draw_grid<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        render_pass.set_pipeline(&self.grid_pipeline);
        render_pass.set_bind_group(0, &self.camera_uniform.bind_group, &[]);
        render_pass.draw(0..6, 0..2);
    }

    fn draw_object<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>, object_verticies: u32) {
        render_pass.set_pipeline(&self.object_pipeline);
        render_pass.set_bind_group(0, &self.camera_uniform.bind_group, &[]);
        render_pass.set_vertex_buffer(
            0,
            self.object_buffer
                .slice(0..(object::VERTEX_SIZE as u64 * object_verticies as u64)),
        );
        render_pass.draw(0..object_verticies, 0..object_verticies / 3);
    }
}
