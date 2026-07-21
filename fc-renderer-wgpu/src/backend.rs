use fc_render::backend::RendererBackend;
use fc_render::commands::DrawCommand;
use fc_primitives::Rect;

use crate::types::{Uniforms, Vertex};
use crate::vertex_gen;

pub struct WgpuBackend {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    uniform_buffer: Option<wgpu::Buffer>,
    uniform_bind_group: Option<wgpu::BindGroup>,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vertex_buffer_capacity: u32,
    index_buffer_capacity: u32,
    width: u32,
    height: u32,
    clip_rect: Option<Rect>,
    frame_started: bool,
    surface_texture: Option<wgpu::SurfaceTexture>,
}

impl WgpuBackend {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self {
            device,
            queue,
            surface: None,
            surface_config: None,
            pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            uniform_buffer: None,
            uniform_bind_group: None,
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_buffer_capacity: 0,
            index_buffer_capacity: 0,
            width: 0,
            height: 0,
            clip_rect: None,
            frame_started: false,
            surface_texture: None,
        }
    }

    pub fn set_surface(
        &mut self,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
    ) {
        self.width = config.width;
        self.height = config.height;
        self.surface_config = Some(config);
        self.surface = Some(surface);
        self.pipeline = None;
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn surface(&self) -> Option<&wgpu::Surface<'static>> {
        self.surface.as_ref()
    }

    pub fn surface_config(&self) -> Option<&wgpu::SurfaceConfiguration> {
        self.surface_config.as_ref()
    }

    fn ensure_pipeline(&mut self) {
        if self.pipeline.is_some() {
            return;
        }

        let Some(config) = &self.surface_config else {
            log::warn!("ensure_pipeline: no surface config — skipping");
            return;
        };

        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("basic.wgsl"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("shaders/basic.wgsl").into(),
                ),
            });

        let bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("uniforms_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("pipeline_layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        let pipeline =
            self.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("render_pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[Vertex::LAYOUT],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                });

        let uniform_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("uniform_buffer"),
                    contents: bytemuck::bytes_of(&Uniforms {
                        resolution: [self.width as f32, self.height as f32],
                        time: 0.0,
                        _padding: 0.0,
                    }),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let uniform_bind_group =
            self.device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("uniform_bind_group"),
                    layout: &bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    }],
                });

        self.pipeline = Some(pipeline);
        self.uniform_buffer = Some(uniform_buffer);
        self.uniform_bind_group = Some(uniform_bind_group);
    }

    fn ensure_buffers(&mut self, min_vertices: u32, min_indices: u32) {
        let mut needed_verts = min_vertices;
        let mut needed_inds = min_indices;

        if self.vertex_buffer.is_none() || self.vertex_buffer_capacity < needed_verts {
            needed_verts = needed_verts.max(1024).max(self.vertex_buffer_capacity * 2);
            self.vertex_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("vertex_buffer"),
                size: (needed_verts as usize * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.vertex_buffer_capacity = needed_verts;
        }

        if self.index_buffer.is_none() || self.index_buffer_capacity < needed_inds {
            needed_inds = needed_inds.max(2048).max(self.index_buffer_capacity * 2);
            self.index_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("index_buffer"),
                size: (needed_inds as usize * std::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.index_buffer_capacity = needed_inds;
        }
    }
}

use wgpu::util::DeviceExt;

impl RendererBackend for WgpuBackend {
    fn execute(&mut self, commands: &[DrawCommand]) {
        if !self.frame_started {
            log::trace!("execute called without begin_frame — skipping");
            return;
        }

        if commands.is_empty() {
            return;
        }

        self.ensure_pipeline();

        let surface_w = self.width as f32;
        let surface_h = self.height as f32;

        self.vertices.clear();
        self.indices.clear();
        vertex_gen::generate_sorted_vertices(commands, surface_w, surface_h, &mut self.vertices, &mut self.indices);

        if self.vertices.is_empty() {
            return;
        }

        self.ensure_buffers(self.vertices.len() as u32, self.indices.len() as u32);

        if let Some(vb) = &self.vertex_buffer {
            self.queue.write_buffer(
                vb,
                0,
                bytemuck::cast_slice(&self.vertices),
            );
        }
        if let Some(ib) = &self.index_buffer {
            self.queue.write_buffer(
                ib,
                0,
                bytemuck::cast_slice(&self.indices),
            );
        }

        let (pipeline, uniform_bind_group) =
            match (&self.pipeline, &self.uniform_bind_group) {
                (Some(p), Some(bg)) => (p, bg),
                _ => {
                    log::warn!("execute: pipeline or bind group not ready");
                    return;
                }
            };

        let surface_texture = match self.surface_texture.take() {
            Some(t) => t,
            None => {
                log::warn!("execute: no surface texture — was begin_frame called?");
                return;
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("frame_encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, uniform_bind_group, &[]);

            if let (Some(vb), Some(ib)) = (&self.vertex_buffer, &self.index_buffer) {
                render_pass.set_vertex_buffer(0, vb.slice(..));
                render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);

                if let Some(clip) = &self.clip_rect {
                    render_pass.set_scissor_rect(
                        clip.x as u32,
                        clip.y as u32,
                        clip.width as u32,
                        clip.height as u32,
                    );
                }

                render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        if let (Some(surface), Some(config)) = (&self.surface, &self.surface_config) {
            let mut new_config = config.clone();
            new_config.width = width;
            new_config.height = height;
            surface.configure(&self.device, &new_config);
            self.surface_config = Some(new_config);
        }
    }

    fn set_clip(&mut self, rect: Rect) {
        self.clip_rect = Some(rect);
    }

    fn clear_clip(&mut self) {
        self.clip_rect = None;
    }

    fn clear(&mut self, color: [f32; 4]) {
        let Some(surface) = &self.surface else {
            return;
        };

        let texture = match surface.get_current_texture() {
            Ok(t) => t,
            Err(e) => {
                log::warn!("clear: failed to get surface texture: {e}");
                return;
            }
        };

        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("clear_encoder"),
                });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: color[0] as f64,
                            g: color[1] as f64,
                            b: color[2] as f64,
                            a: color[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        texture.present();
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn begin_frame(&mut self) {
        if self.frame_started {
            log::trace!("begin_frame called twice — ignoring");
            return;
        }

        let Some(surface) = &self.surface else {
            log::trace!("begin_frame: no surface — skipping");
            return;
        };

        match surface.get_current_texture() {
            Ok(texture) => {
                self.surface_texture = Some(texture);
                self.frame_started = true;
            }
            Err(e) => {
                log::warn!("begin_frame: failed to acquire surface texture: {e}");
            }
        }
    }

    fn end_frame(&mut self) {
        if !self.frame_started {
            return;
        }

        if let Some(texture) = self.surface_texture.take() {
            texture.present();
        }
        self.frame_started = false;
    }
}
