use super::types::{Uniforms, Vertex, colors};

/// GPU renderer for chart grid lines (horizontal price levels + vertical time intervals).
///
/// Grid lines are drawn as `LineList` primitives with dim gray color and 50% alpha.
/// Vertex data is regenerated on each `update_grid()` call and uploaded to the GPU.
pub struct GridRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl GridRenderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        shader: &wgpu::ShaderModule,
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("grid pipeline layout"),
            bind_group_layouts: &[
                &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("grid uniform bgl"),
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
            label: Some("grid pipeline"),
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

        // Pre-allocate for ~10 horizontal + ~20 vertical = ~60 vertices (2 per line).
        let max_vertices = 120;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("grid vertex buffer"),
            size: (max_vertices * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("grid uniform buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("grid bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            vertex_buffer,
            vertex_count: 0,
            uniform_buffer,
            bind_group,
        }
    }

    /// Regenerate grid line vertices for the current canvas dimensions.
    ///
    /// Generates ~10 horizontal lines (price levels) and ~20 vertical lines (time intervals),
    /// all spanning the full canvas in NDC space.
    pub fn update_grid(
        &mut self,
        queue: &wgpu::Queue,
        width: f32,
        height: f32,
    ) {
        let mut vertices = Vec::with_capacity(120);

        let grid_color = colors::GRID;

        // --- Horizontal lines (price levels) ---
        let h_count = 10;
        for i in 0..=h_count {
            let y = -1.0 + 2.0 * (i as f32 / h_count as f32);
            vertices.push(Vertex::new([-1.0, y], grid_color));
            vertices.push(Vertex::new([1.0, y], grid_color));
        }

        // --- Vertical lines (time intervals) ---
        let v_count = 20;
        for i in 0..=v_count {
            let x = -1.0 + 2.0 * (i as f32 / v_count as f32);
            vertices.push(Vertex::new([x, -1.0], grid_color));
            vertices.push(Vertex::new([x, 1.0], grid_color));
        }

        self.vertex_count = vertices.len() as u32;

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        let uniforms = Uniforms::new(width, height);
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
