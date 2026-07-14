use fast_chart_core::{PriceLine, Viewport};

use super::types::{Uniforms, Vertex};

const MAX_PRICE_LINES: usize = 100;
const MAX_VERTICES: usize = MAX_PRICE_LINES * 4; // 4 vertices per quad (2 triangles)
const MAX_INDICES: usize = MAX_PRICE_LINES * 6; // 6 indices per quad (2 triangles)

/// Renders horizontal price lines at specific price levels.
///
/// Price lines appear above markers but below text labels.
/// Uses indexed TriangleList rendering with the line.wgsl shader.
pub struct PriceLineRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_count: u32,
    index_count: u32,
}

impl PriceLineRenderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        shader: &wgpu::ShaderModule,
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("price line pipeline layout"),
            bind_group_layouts: &[
                &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("price line uniform bgl"),
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
            label: Some("price line pipeline"),
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
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("price line vertex buffer"),
            size: (MAX_VERTICES * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("price line index buffer"),
            size: (MAX_INDICES * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("price line uniform buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("price line bind group"),
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
            vertex_count: 0,
            index_count: 0,
        }
    }

    /// Update price lines for the current viewport.
    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        price_lines: &[PriceLine],
        viewport: &Viewport,
        canvas_width: f32,
        canvas_height: f32,
    ) {
        if price_lines.is_empty() {
            self.vertex_count = 0;
            self.index_count = 0;
            return;
        }

        let value_range = viewport.value_max - viewport.value_min;
        if value_range < f64::EPSILON {
            self.vertex_count = 0;
            self.index_count = 0;
            return;
        }

        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for line in price_lines {
            // Skip lines outside viewport
            if line.price < viewport.value_min || line.price > viewport.value_max {
                continue;
            }

            // Calculate Y position in NDC
            let value_ratio = (line.price - viewport.value_min) / value_range;
            let y_ndc = (-1.0 + 2.0 * value_ratio) as f32;

            // Line extends from left to right
            let x_left = -1.0;
            let x_right = 1.0;

            // Line thickness in NDC (convert from pixels)
            let thickness = line.width / canvas_height;

            let base_index = vertices.len() as u32;
            let color = line.color;

            // Create a thin quad for the line
            vertices.push(Vertex::new([x_left, y_ndc - thickness], color));
            vertices.push(Vertex::new([x_right, y_ndc - thickness], color));
            vertices.push(Vertex::new([x_right, y_ndc + thickness], color));
            vertices.push(Vertex::new([x_left, y_ndc + thickness], color));

            // Two triangles forming the quad
            indices.extend_from_slice(&[
                base_index,
                base_index + 1,
                base_index + 2,
                base_index,
                base_index + 2,
                base_index + 3,
            ]);

            // Stop if we hit the limit
            if vertices.len() >= MAX_VERTICES || indices.len() >= MAX_INDICES {
                break;
            }
        }

        self.vertex_count = vertices.len() as u32;
        self.index_count = indices.len() as u32;

        if self.vertex_count > 0 {
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
            queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));
        }

        // Upload uniforms
        let uniforms = Uniforms::new(canvas_width, canvas_height);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.vertex_count == 0 || self.index_count == 0 {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
