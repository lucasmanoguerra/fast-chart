// OHLC bar vertex/fragment shader.
//
// Vertex positions arrive in world space: x = timestamp (u64 as f32),
// y = price (f64 as f32). The uniform viewport maps world → NDC,
// enabling zoom/pan by updating the uniform buffer alone.
//
// This shader is structurally identical to candle.wgsl — both render
// world-space primitives that need the same viewport transform.

struct Uniforms {
    resolution: vec2<f32>,
    viewport_x_min: f32,
    viewport_x_max: f32,
    viewport_y_min: f32,
    viewport_y_max: f32,
    padding: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

fn world_to_ndc(world: vec2<f32>) -> vec2<f32> {
    let x = (world.x - uniforms.viewport_x_min) / (uniforms.viewport_x_max - uniforms.viewport_x_min) * 2.0 - 1.0;
    let y = 1.0 - (world.y - uniforms.viewport_y_min) / (uniforms.viewport_y_max - uniforms.viewport_y_min) * 2.0;
    return vec2<f32>(x, y);
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(world_to_ndc(in.position), 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
