use super::types::{Uniforms, Vertex};

/// Renders crosshair lines (vertical + horizontal) at the cursor position.
///
/// Uses the same line.wgsl shader as the grid/divider renderers.
pub struct CrosshairRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_count: u32,
}

impl CrosshairRenderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        shader: &wgpu::ShaderModule,
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("crosshair pipeline layout"),
            bind_group_layouts: &[
                &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("crosshair uniform bgl"),
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
            label: Some("crosshair pipeline"),
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

        // 4 vertices: 2 per line (vertical + horizontal)
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("crosshair vertex buffer"),
            size: (4 * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("crosshair uniform buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("crosshair bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            vertex_buffer,
            uniform_buffer,
            bind_group,
            vertex_count: 0,
        }
    }

    /// Update crosshair lines at the given pixel position.
    /// `active` controls visibility — when false, no lines are drawn.
    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        cursor_x: f32,
        cursor_y: f32,
        canvas_width: f32,
        canvas_height: f32,
        active: bool,
    ) {
        if !active || canvas_width < 1.0 || canvas_height < 1.0 {
            self.vertex_count = 0;
            return;
        }

        // Convert pixel coords to NDC: x ∈ [-1, 1], y ∈ [-1, 1] (Y flipped)
        let x_ndc = -1.0 + 2.0 * (cursor_x / canvas_width);
        let y_ndc = 1.0 - 2.0 * (cursor_y / canvas_height);

        let color = [1.0, 1.0, 1.0, 0.4]; // dim white

        let vertices = [
            // Vertical line (full height at cursor X)
            Vertex::new([x_ndc, -1.0], color),
            Vertex::new([x_ndc, 1.0], color),
            // Horizontal line (full width at cursor Y)
            Vertex::new([-1.0, y_ndc], color),
            Vertex::new([1.0, y_ndc], color),
        ];

        self.vertex_count = 4;
        queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&vertices),
        );

        let uniforms = Uniforms::new(canvas_width, canvas_height);
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&uniforms),
        );
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
