# Renderer Crate (`fc-renderer-wgpu`)

## Purpose

`fc-renderer-wgpu` provides the wgpu-based GPU rendering backend. It
implements the `RendererBackend` trait from the library crate and translates
`DrawCommand`s into GPU draw calls.

---

## WgpuBackend

The main entry point. Holds wgpu state and implements `RendererBackend`:

```rust
pub struct WgpuBackend {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    uniform_buffer: Option<wgpu::Buffer>,
    // ...
}
```

### Lifecycle

```rust
let mut backend = WgpuBackend::new(device, queue);
backend.set_surface(surface, config);

// Per frame
backend.begin_frame();
backend.clear(color);
backend.execute(&draw_commands);
backend.end_frame();
```

---

## GPU Pipeline

### Vertex Generation

`vertex_gen` converts `DrawCommand`s into GPU-ready vertices and indices.
Each primitive type (line, rect, circle, triangle, path, text, image) has
its own vertex generation logic.

### Sub-Renderers

| Renderer | File | Handles |
|----------|------|---------|
| `candle` | `renderers/candle.rs` | Candlestick bars |
| `line` | `renderers/line.rs` | Line series |
| `area` | `renderers/area.rs` | Area fill series |
| `bar` | `renderers/bar.rs` | OHLC bar series |
| `histogram` | `renderers/histogram.rs` | Histogram series |
| `baseline` | `renderers/baseline.rs` | Baseline series |
| `text` | `renderers/text.rs` | Text labels |

### Scissor Clipping

`scissor.rs` manages pane-level clipping via wgpu scissor tests. Each pane's
render area is clipped to prevent bleed between panes.

### GPU Cache

`cache.rs` manages GPU buffer allocation and reuse. Vertex and index buffers
are grown as needed and reused across frames to minimize allocations.

### Types

`types.rs` defines GPU-specific types: `Vertex` (position + color + UV),
`Uniforms` (transform matrix, viewport dimensions).

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `wgpu` | GPU abstraction (Vulkan/Metal/DX12/WebGPU) |
| `bytemuck` | Zero-copy type casting for GPU buffers |
| `log` | Logging |

---

## WgpuRenderer

Higher-level renderer that wraps `WgpuBackend` with series-specific rendering
logic. Orchestrates the render passes and manages the draw command queue.
