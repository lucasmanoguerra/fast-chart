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

/// Minimal uniform buffer — currently unused by the shader (CPU computes NDC).
/// Exists to satisfy the `@group(0) @binding(0) var<uniform>` declaration.
/// Phase 6 will add a `transform: mat4x4<f32>` field here for zoom/pan.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub resolution: [f32; 2],
    pub padding: [f32; 2],
}

impl Uniforms {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            resolution: [width, height],
            padding: [0.0; 2],
        }
    }
}

/// RGB color constants for chart elements.
pub mod colors {
    pub const GRID: [f32; 4] = [0.2, 0.2, 0.25, 0.5];
    pub const LINE_CLOSE: [f32; 4] = [0.0, 0.8, 1.0, 1.0];
    pub const BACKGROUND: wgpu::Color = wgpu::Color {
        r: 0.05,
        g: 0.05,
        b: 0.08,
        a: 1.0,
    };
}
