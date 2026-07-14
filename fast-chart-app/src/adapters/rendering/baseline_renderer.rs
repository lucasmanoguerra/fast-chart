use fast_chart_core::Bar;

use super::pipeline_utils;
use super::types::{Uniforms, Vertex, colors};

/// GPU renderer for baseline charts.
///
/// Similar to area, but fills above baseline with one color
/// and below baseline with another color. Each pair of consecutive bars
/// forms a quad where the color depends on whether the close is above
/// or below the baseline price.
#[allow(dead_code)]
pub struct BaselineRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    index_capacity: usize,
    vertex_count: u32,
    index_count: u32,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    above_color: [f32; 4],
    below_color: [f32; 4],
    baseline_price: f64,
}

#[allow(dead_code)]
impl BaselineRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let fill = pipeline_utils::create_fill_pipeline(device, format, "baseline");

        // Pre-allocate for up to 100k bars.
        let vertex_capacity = 200_000;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("baseline vertex buffer"),
            size: (vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_capacity = 100_000 * 6;
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("baseline index buffer"),
            size: (index_capacity * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline: fill.pipeline,
            vertex_buffer,
            index_buffer,
            vertex_capacity,
            index_capacity,
            vertex_count: 0,
            index_count: 0,
            uniform_buffer: fill.uniform_buffer,
            bind_group: fill.bind_group,
            above_color: colors::BULLISH,
            below_color: colors::BEARISH,
            baseline_price: 0.0,
        }
    }

    pub fn set_colors(&mut self, above: [f32; 4], below: [f32; 4]) {
        self.above_color = above;
        self.below_color = below;
    }

    pub fn set_baseline(&mut self, price: f64) {
        self.baseline_price = price;
    }

    /// Generate triangle vertices for baseline fill.
    ///
    /// Each bar generates two vertices: one at close, one at baseline.
    /// Color is determined by whether close is above or below baseline.
    /// Consecutive bar pairs form quads via index buffers.
    pub fn update_baseline(
        &mut self,
        queue: &wgpu::Queue,
        bars: &[Bar],
        canvas_width: f32,
        canvas_height: f32,
        time_start: f64,
        time_end: f64,
        value_min: f64,
        value_max: f64,
    ) {
        if bars.len() < 2 {
            self.vertex_count = 0;
            self.index_count = 0;
            return;
        }

        let time_range = time_end - time_start;
        let value_range = value_max - value_min;

        if time_range < f64::EPSILON || value_range < f64::EPSILON {
            self.vertex_count = 0;
            self.index_count = 0;
            return;
        }

        let mut vertices: Vec<Vertex> = Vec::with_capacity(bars.len() * 2);
        let mut indices: Vec<u32> = Vec::with_capacity((bars.len() - 1) * 6);

        let cw = canvas_width as f64;
        let ch = canvas_height as f64;

        for bar in bars {
            let x = ((bar.timestamp as f64 - time_start) / time_range * cw) as f32;
            let y_close = ((1.0 - (bar.close - value_min) / value_range) * ch) as f32;
            let y_baseline =
                ((1.0 - (self.baseline_price - value_min) / value_range) * ch) as f32;

            let color = if bar.close >= self.baseline_price {
                self.above_color
            } else {
                self.below_color
            };

            // Close-price vertex (top of area)
            vertices.push(Vertex::new([x, y_close], color));
            // Baseline vertex (bottom of area, transparent)
            vertices.push(Vertex::new(
                [x, y_baseline],
                [0.0, 0.0, 0.0, 0.0],
            ));
        }

        // Build quads from consecutive bar pairs.
        for i in 0..(bars.len() - 1) {
            let base = (i as u32) * 2;
            indices.extend_from_slice(&[base, base + 1, base + 2]);
            indices.extend_from_slice(&[base + 1, base + 3, base + 2]);
        }

        let vert_count = vertices.len();
        let idx_count = indices.len();

        if vert_count > self.vertex_capacity || idx_count > self.index_capacity {
            log::warn!(
                "Baseline renderer: vertex count {} or index count {} exceeds capacity",
                vert_count,
                idx_count
            );
            self.vertex_count = 0;
            self.index_count = 0;
            return;
        }

        self.vertex_count = vert_count as u32;
        self.index_count = idx_count as u32;

        if vert_count > 0 {
            queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&vertices),
            );
        }
        if idx_count > 0 {
            queue.write_buffer(
                &self.index_buffer,
                0,
                bytemuck::cast_slice(&indices),
            );
        }

        let uniforms = Uniforms::new(canvas_width, canvas_height);
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&uniforms),
        );
    }

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
