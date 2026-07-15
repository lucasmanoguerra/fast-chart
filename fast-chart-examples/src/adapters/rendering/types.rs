/// Shared vertex type for all rendering pipelines (grid, line, candle, etc.).
///
/// Coordinates must be in normalized device coordinates (NDC):
/// - x ∈ [-1, 1] (left to right)
/// - y ∈ [-1, 1] (bottom to top: -1 = low price, +1 = high price)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub const fn new(position: [f32; 2], color: [f32; 4]) -> Self {
        Self { position, color }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position: vec2<f32> @location(0)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // color: vec4<f32> @location(1)
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Uniform buffer shared by all rendering pipelines.
///
/// The line/grid shaders read `resolution` (CPU-computed NDC).
/// The candle shader reads the viewport fields (GPU world→NDC transform).
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub resolution: [f32; 2],
    pub viewport_x_min: f32,
    pub viewport_x_max: f32,
    pub viewport_y_min: f32,
    pub viewport_y_max: f32,
    pub padding: [f32; 2],
}

impl Uniforms {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            resolution: [width, height],
            viewport_x_min: 0.0,
            viewport_x_max: 1.0,
            viewport_y_min: 0.0,
            viewport_y_max: 1.0,
            padding: [0.0; 2],
        }
    }

    /// Uniforms with viewport values for the candle shader's world→NDC transform.
    pub fn with_viewport(
        width: f32,
        height: f32,
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
    ) -> Self {
        Self {
            resolution: [width, height],
            viewport_x_min: x_min,
            viewport_x_max: x_max,
            viewport_y_min: y_min,
            viewport_y_max: y_max,
            padding: [0.0; 2],
        }
    }
}

/// RGB color constants for chart elements.
pub mod colors {
    pub const GRID: [f32; 4] = [0.2, 0.2, 0.25, 0.5];
    pub const LINE_CLOSE: [f32; 4] = [0.0, 0.8, 1.0, 1.0];
    pub const BULLISH: [f32; 4] = [0.1, 0.8, 0.3, 1.0];
    pub const BEARISH: [f32; 4] = [0.9, 0.2, 0.2, 1.0];
    pub const WICK_BULLISH: [f32; 4] = [0.1, 0.8, 0.3, 0.7];
    pub const WICK_BEARISH: [f32; 4] = [0.9, 0.2, 0.2, 0.7];
    pub const BACKGROUND: wgpu::Color = wgpu::Color {
        r: 0.05,
        g: 0.05,
        b: 0.08,
        a: 1.0,
    };
}
