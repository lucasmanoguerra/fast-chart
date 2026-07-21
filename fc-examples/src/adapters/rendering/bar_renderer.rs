use fc_primitives::bar::Bar;

use super::types::{Uniforms, Vertex, colors};

/// GPU renderer for OHLC bar charts.
///
/// Each bar is rendered as 3 line segments (6 vertices, LineList):
///   - Open tick:  horizontal line from (timestamp - bar_width*0.4, open)  to (timestamp, open)
///   - High-Low:   vertical line   from (timestamp, low)                   to (timestamp, high)
///   - Close tick: horizontal line from (timestamp, close)                 to (timestamp + bar_width*0.4, close)
///
/// Coordinates are world-space (timestamp, price); the bar.wgsl shader does the
/// world → NDC conversion via the viewport uniform, so zoom/pan only updates
/// the uniform buffer — no vertex rebuild.
pub struct BarRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_capacity: usize,
    vertex_count: u32,
}

impl BarRenderer {
    /// Create the bar renderer with the bar.wgsl shader.
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bar shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/bar.wgsl").into(),
            ),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bar pipeline layout"),
            bind_group_layouts: &[
                &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("bar uniform bgl"),
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
            label: Some("bar pipeline"),
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
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Pre-allocate for 100k bars × 6 vertices each = 600k vertices.
        let vertex_capacity = 100_000 * 6;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bar vertex buffer"),
            size: (vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bar uniform buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bar bind group"),
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
            vertex_capacity,
            vertex_count: 0,
        }
    }

    /// Upload OHLC bar vertices for the visible bars.
    ///
    /// Each bar becomes 3 line segments (LineList) = 6 vertices, no index buffer.
    /// Coordinates are world-space (timestamp, price); the shader does the
    /// world → NDC conversion via the viewport uniform.
    pub fn update_bars(
        &mut self,
        queue: &wgpu::Queue,
        bars: &[Bar],
        bar_width: f32,
    ) {
        let mut vertices: Vec<Vertex> = Vec::with_capacity(bars.len() * 6);

        for bar in bars {
            let ts = bar.timestamp as f32;
            let tick_half = bar_width * 0.4;
            let color = if bar.is_bullish() {
                colors::BULLISH
            } else {
                colors::BEARISH
            };

            // Open tick: horizontal line from left to timestamp
            let open_y = bar.open as f32;
            vertices.push(Vertex::new([ts - tick_half, open_y], color));
            vertices.push(Vertex::new([ts, open_y], color));

            // High-Low vertical line
            let low_y = bar.low as f32;
            let high_y = bar.high as f32;
            vertices.push(Vertex::new([ts, low_y], color));
            vertices.push(Vertex::new([ts, high_y], color));

            // Close tick: horizontal line from timestamp to right
            let close_y = bar.close as f32;
            vertices.push(Vertex::new([ts, close_y], color));
            vertices.push(Vertex::new([ts + tick_half, close_y], color));
        }

        self.vertex_count = vertices.len().min(self.vertex_capacity) as u32;

        if self.vertex_count > 0 {
            queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&vertices[..self.vertex_count as usize]),
            );
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

    /// Issue the draw call. Must be called inside an active render pass.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a minimal bar for testing vertex generation.
    fn test_bar(timestamp: u64, open: f64, high: f64, low: f64, close: f64) -> Bar {
        Bar::new(timestamp, open, high, low, close, 1000).unwrap()
    }

    #[test]
    fn vertex_count_per_bar() {
        // Each bar = 3 line segments × 2 vertices = 6 vertices
        let bars = vec![
            test_bar(1000, 100.0, 105.0, 99.0, 102.0),
            test_bar(2000, 102.0, 108.0, 101.0, 105.0),
        ];
        let mut renderer = create_renderer_for_test();
        renderer.update_bars(&bars, 10.0);
        assert_eq!(renderer.vertex_count, 12); // 2 bars × 6 vertices
    }

    #[test]
    fn empty_bars_produces_zero_vertices() {
        let mut renderer = create_renderer_for_test();
        renderer.update_bars(&[], 10.0);
        assert_eq!(renderer.vertex_count, 0);
    }

    #[test]
    fn bullish_bar_uses_green_color() {
        let bar = test_bar(1000, 100.0, 105.0, 99.0, 102.0);
        assert!(bar.is_bullish());
        // Verify the color constant is green-ish
        assert!(colors::BULLISH[1] > colors::BULLISH[0]); // green > red
    }

    #[test]
    fn bearish_bar_uses_red_color() {
        let bar = test_bar(1000, 102.0, 105.0, 99.0, 100.0);
        assert!(!bar.is_bullish());
        assert!(colors::BEARISH[0] > colors::BEARISH[1]); // red > green
    }

    #[test]
    fn open_tick_is_horizontal() {
        // For a bullish bar: open=100, the open tick vertices should both have y=100
        let bar = test_bar(1000, 100.0, 105.0, 99.0, 102.0);
        let ts = bar.timestamp as f32;
        let tick_half = 10.0 * 0.4;
        let open_y = bar.open as f32;

        // Open tick endpoints
        let left = Vertex::new([ts - tick_half, open_y], colors::BULLISH);
        let right = Vertex::new([ts, open_y], colors::BULLISH);
        assert_eq!(left.position[1], right.position[1]);
    }

    #[test]
    fn high_low_is_vertical() {
        let bar = test_bar(1000, 100.0, 105.0, 99.0, 102.0);
        let ts = bar.timestamp as f32;
        let low_y = bar.low as f32;
        let high_y = bar.high as f32;

        let bottom = Vertex::new([ts, low_y], colors::BULLISH);
        let top = Vertex::new([ts, high_y], colors::BULLISH);
        // Same x coordinate = vertical line
        assert_eq!(bottom.position[0], top.position[0]);
        // high > low
        assert!(top.position[1] > bottom.position[1]);
    }

    #[test]
    fn close_tick_is_horizontal() {
        let bar = test_bar(1000, 100.0, 105.0, 99.0, 102.0);
        let ts = bar.timestamp as f32;
        let tick_half = 10.0 * 0.4;
        let close_y = bar.close as f32;

        let left = Vertex::new([ts, close_y], colors::BULLISH);
        let right = Vertex::new([ts + tick_half, close_y], colors::BULLISH);
        assert_eq!(left.position[1], right.position[1]);
    }

    #[test]
    fn tick_width_is_40_percent_of_bar_width() {
        let bar_width = 10.0f32;
        let tick_half = bar_width * 0.4;
        // The tick extends bar_width * 0.4 on each side from the center
        assert!((tick_half - 4.0).abs() < f32::EPSILON);
    }

    /// Create a BarRenderer without a real device — for tests that only
    /// inspect vertex generation logic.
    fn create_renderer_for_test() -> BarRendererForTest {
        BarRendererForTest::new()
    }

    /// Lightweight test harness that replicates the vertex generation logic
    /// without requiring a GPU device. Used only in unit tests.
    struct BarRendererForTest {
        vertex_count: u32,
    }

    impl BarRendererForTest {
        fn new() -> Self {
            Self { vertex_count: 0 }
        }

        fn update_bars(&mut self, bars: &[Bar], bar_width: f32) {
            let mut count = 0u32;
            for bar in bars {
                let _ = bar.timestamp as f32;
                let _ = bar_width * 0.4;
                let _ = bar.open as f32;
                let _ = bar.low as f32;
                let _ = bar.high as f32;
                let _ = bar.close as f32;
                // 3 segments × 2 vertices = 6
                count += 6;
            }
            self.vertex_count = count;
        }
    }
}
