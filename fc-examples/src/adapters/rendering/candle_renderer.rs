use fc_core::Bar;

use super::types::{Uniforms, Vertex, colors};

/// GPU renderer for candlestick charts.
///
/// Each candle is a world-space quad (body + wick) rendered via index buffers.
/// The vertex shader maps (timestamp, price) → NDC using the viewport uniform,
/// so zoom/pan only updates the uniform buffer — no vertex rebuild.
pub struct CandleRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_capacity: usize,
    vertex_count: u32,
    index_count: u32,
}

impl CandleRenderer {
    /// Create the candle renderer with the candle.wgsl shader.
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("candle shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/candle.wgsl").into(),
            ),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("candle pipeline layout"),
            bind_group_layouts: &[
                &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("candle uniform bgl"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                }),
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("candle pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Pre-allocate for 100k candles: 8 verts + 12 indices each.
        let vertex_capacity = 100_000 * 8;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("candle vertex buffer"),
            size: (vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_capacity = 100_000 * 12;
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("candle index buffer"),
            size: (index_capacity * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("candle uniform buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("candle bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group,
            vertex_capacity,
            vertex_count: 0,
            index_count: 0,
        }
    }

    /// Upload candle vertices and indices for the visible bars.
    ///
    /// Each bar becomes 2 quads (body + wick) = 8 vertices, 12 indices.
    /// Coordinates are world-space (timestamp, price); the shader does the
    /// world → NDC conversion via the viewport uniform.
    pub fn update_candles(
        &mut self,
        queue: &wgpu::Queue,
        bars: &[Bar],
        bar_width: f32,
    ) {
        let mut vertices: Vec<Vertex> = Vec::with_capacity(bars.len() * 8);
        let mut indices: Vec<u32> = Vec::with_capacity(bars.len() * 12);

        for bar in bars {
            let ts = bar.timestamp as f32;
            let half_body = bar_width * 0.5;
            let wick_half = bar_width * 0.1;

            let (body_color, wick_color) = if bar.is_bullish() {
                (colors::BULLISH, colors::WICK_BULLISH)
            } else {
                (colors::BEARISH, colors::WICK_BEARISH)
            };

            // Body quad: 4 vertices
            let v0 = vertices.len() as u32;
            vertices.push(Vertex::new([ts - half_body, bar.open as f32], body_color));
            vertices.push(Vertex::new([ts + half_body, bar.open as f32], body_color));
            vertices.push(Vertex::new([ts + half_body, bar.close as f32], body_color));
            vertices.push(Vertex::new([ts - half_body, bar.close as f32], body_color));
            indices.extend_from_slice(&[v0, v0 + 1, v0 + 2, v0, v0 + 2, v0 + 3]);

            // Wick quad: 4 vertices
            let v4 = vertices.len() as u32;
            vertices.push(Vertex::new([ts - wick_half, bar.low as f32], wick_color));
            vertices.push(Vertex::new([ts + wick_half, bar.low as f32], wick_color));
            vertices.push(Vertex::new([ts + wick_half, bar.high as f32], wick_color));
            vertices.push(Vertex::new([ts - wick_half, bar.high as f32], wick_color));
            indices.extend_from_slice(&[v4, v4 + 1, v4 + 2, v4, v4 + 2, v4 + 3]);
        }

        self.vertex_count = vertices.len().min(self.vertex_capacity) as u32;
        self.index_count = indices.len() as u32;

        if self.vertex_count > 0 {
            queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&vertices[..self.vertex_count as usize]),
            );
            queue.write_buffer(
                &self.index_buffer,
                0,
                bytemuck::cast_slice(&indices[..self.index_count as usize]),
            );
        } else {
            self.vertex_count = 0;
            self.index_count = 0;
        }
    }

    /// Update the viewport uniform so the shader maps world coords → NDC.
    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        width: f32,
        height: f32,
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
    ) {
        let uniforms = Uniforms::with_viewport(width, height, x_min, x_max, y_min, y_max);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Issue the indexed draw call. Must be called inside an active render pass.
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.index_count == 0 {
            return;
        }
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
