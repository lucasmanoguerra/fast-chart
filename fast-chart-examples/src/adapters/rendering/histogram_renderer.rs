use fast_chart::Bar;

use super::pipeline_utils;
use super::types::{Uniforms, Vertex, colors};

/// GPU renderer for histogram charts.
///
/// Renders vertical bars from a baseline to each bar's value.
/// Typically used for volume or oscillator displays. Each bar
/// generates a quad (4 vertices + 6 indices) rendered with the
/// `fill.wgsl` shader.
#[allow(dead_code)]
pub struct HistogramRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    index_capacity: usize,
    vertex_count: u32,
    index_count: u32,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    positive_color: [f32; 4],
    negative_color: [f32; 4],
    baseline_price: f64,
}

#[allow(dead_code)]
impl HistogramRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let fill = pipeline_utils::create_fill_pipeline(device, format, "histogram");

        // Pre-allocate for 100k bars: 4 verts + 6 indices each.
        let vertex_capacity = 100_000 * 4;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("histogram vertex buffer"),
            size: (vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_capacity = 100_000 * 6;
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("histogram index buffer"),
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
            positive_color: colors::BULLISH,
            negative_color: colors::BEARISH,
            baseline_price: 0.0,
        }
    }

    pub fn set_colors(&mut self, positive: [f32; 4], negative: [f32; 4]) {
        self.positive_color = positive;
        self.negative_color = negative;
    }

    pub fn set_baseline(&mut self, price: f64) {
        self.baseline_price = price;
    }

    /// Generate quad vertices for histogram bars.
    ///
    /// Each bar generates 4 vertices (quad) + 6 indices (2 triangles).
    /// The bar width is 80% of the available slot width per bar.
    pub fn update_histogram(
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
        if bars.is_empty() {
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

        let cw = canvas_width as f64;
        let ch = canvas_height as f64;
        let bar_width = (cw / bars.len() as f64) * 0.8;
        let mut vertices: Vec<Vertex> = Vec::with_capacity(bars.len() * 4);
        let mut indices: Vec<u32> = Vec::with_capacity(bars.len() * 6);

        for bar in bars {
            let x_center = ((bar.timestamp as f64 - time_start) / time_range * cw) as f32;
            let half = (bar_width / 2.0) as f32;
            let x_left = x_center - half;
            let x_right = x_center + half;

            let y_baseline =
                ((1.0 - (self.baseline_price - value_min) / value_range) * ch) as f32;
            let y_value =
                ((1.0 - (bar.close - value_min) / value_range) * ch) as f32;

            let color = if bar.close >= self.baseline_price {
                self.positive_color
            } else {
                self.negative_color
            };

            let base_idx = vertices.len() as u32;

            // 4 vertices for the quad (baseline-left, baseline-right,
            // value-right, value-left).
            vertices.push(Vertex::new([x_left, y_baseline], color));
            vertices.push(Vertex::new([x_right, y_baseline], color));
            vertices.push(Vertex::new([x_right, y_value], color));
            vertices.push(Vertex::new([x_left, y_value], color));

            // 6 indices (2 triangles).
            indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
            indices.extend_from_slice(&[base_idx, base_idx + 2, base_idx + 3]);
        }

        let vert_count = vertices.len();
        let idx_count = indices.len();

        if vert_count > self.vertex_capacity || idx_count > self.index_capacity {
            log::warn!(
                "Histogram renderer: vertex count {} or index count {} exceeds capacity",
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
