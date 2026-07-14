use super::types::{Uniforms, Vertex, colors};
use fast_chart_core::{Bar, TimeSeries, Viewport};

/// GPU renderer for line series (e.g., close price line).
///
/// Generates line-list vertices from a `TimeSeries<Bar>` and renders them
/// using the shared line.wgsl shader. Vertex data is regenerated on each
/// `update_line()` call based on the current viewport.
pub struct LineRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    vertex_count: u32,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl LineRenderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        shader: &wgpu::ShaderModule,
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("line pipeline layout"),
            bind_group_layouts: &[
                &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("line uniform bgl"),
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
            label: Some("line pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    // Alpha blending for smooth line edges in the future.
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Pre-allocate for up to 100_000 bars → ~200_000 vertices (2 per segment).
        let vertex_capacity = 200_000;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("line vertex buffer"),
            size: (vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("line uniform buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("line bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            vertex_buffer,
            vertex_capacity,
            vertex_count: 0,
            uniform_buffer,
            bind_group,
        }
    }

    /// Generate line-strip vertices from the visible bars in the time series.
    ///
    /// For each pair of consecutive visible bars, two vertices are emitted
    /// (LineList topology). The close price is mapped to NDC using the
    /// viewport's time/value ranges and canvas dimensions.
    #[allow(dead_code)]
    pub fn update_line<const N: usize>(
        &mut self,
        queue: &wgpu::Queue,
        series: &TimeSeries<Bar, N>,
        viewport: &Viewport,
        canvas_width: f32,
        canvas_height: f32,
    ) {
        if series.len() < 2 {
            self.vertex_count = 0;
            return;
        }

        let time_range = viewport.time_end as f64 - viewport.time_start as f64;
        let value_range = viewport.value_max - viewport.value_min;

        // Guard against degenerate ranges.
        if time_range < f64::EPSILON || value_range < f64::EPSILON {
            self.vertex_count = 0;
            return;
        }

        let line_color = colors::LINE_CLOSE;
        let mut vertices: Vec<Vertex> = Vec::new();

        let mut prev_x_ndc: f32 = 0.0;
        let mut prev_y_ndc: f32 = 0.0;
        let mut has_prev = false;

        for i in 0..series.len() {
            let bar = match series.get(i) {
                Some(b) => b,
                None => continue,
            };

            // Skip bars outside the viewport time range.
            if bar.timestamp < viewport.time_start || bar.timestamp > viewport.time_end {
                has_prev = false;
                continue;
            }

            // Map timestamp → x NDC: [-1, 1]
            let time_ratio = (bar.timestamp as f64 - viewport.time_start as f64) / time_range;
            let x_ndc = (-1.0 + 2.0 * time_ratio) as f32;

            // Map close price → y NDC: [-1, 1] (bottom = low, top = high)
            let value_ratio = (bar.close - viewport.value_min) / value_range;
            let y_ndc = (-1.0 + 2.0 * value_ratio) as f32;

            if has_prev {
                vertices.push(Vertex::new([prev_x_ndc, prev_y_ndc], line_color));
                vertices.push(Vertex::new([x_ndc, y_ndc], line_color));
            }

            prev_x_ndc = x_ndc;
            prev_y_ndc = y_ndc;
            has_prev = true;
        }

        // Cap to pre-allocated buffer size.
        let count = vertices.len().min(self.vertex_capacity);
        vertices.truncate(count);
        self.vertex_count = count as u32;

        if count > 0 {
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        }

        let uniforms = Uniforms::new(canvas_width, canvas_height);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Clear the vertex count (no data to render).
    pub fn update_line_from_empty(
        &mut self,
        queue: &wgpu::Queue,
        canvas_width: f32,
        canvas_height: f32,
    ) {
        self.vertex_count = 0;
        let uniforms = Uniforms::new(canvas_width, canvas_height);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Upload pre-computed vertices directly (used by GpuRenderer's Vec<Bar> bridge).
    pub fn upload_vertices(
        &mut self,
        queue: &wgpu::Queue,
        vertices: &[Vertex],
        canvas_width: f32,
        canvas_height: f32,
    ) {
        let count = vertices.len().min(self.vertex_capacity);
        self.vertex_count = count as u32;

        if count > 0 {
            queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&vertices[..count]),
            );
        }

        let uniforms = Uniforms::new(canvas_width, canvas_height);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.vertex_count == 0 {
            return;
        }
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
    }
}
