# Architecture Exploration — fast-chart

> Native desktop trading chart application (GPU-accelerated, hexagonal architecture)
> Rust 1.96.1 | Target: x86_64-unknown-linux-gnu | Hybrid persistence (openspec + engram)

---

## Table of Contents

1. [Library Ecosystem Comparison](#1-library-ecosystem-comparison)
2. [Hexagonal Architecture](#2-hexagonal-architecture)
3. [GPU Rendering Pipeline](#3-gpu-rendering-pipeline)
4. [Data Pipeline Performance](#4-data-pipeline-performance)
5. [User Interaction Model](#5-user-interaction-model)
6. [Project Structure](#6-project-structure)
7. [Recommendation Summary](#7-recommendation-summary)
8. [Risks & Mitigations](#8-risks--mitigations)
9. [Next Steps](#9-next-steps)

---

## 1. Library Ecosystem Comparison

### 1.1 Windowing: winit

| Library | Pros | Cons |
|---------|------|------|
| **winit** | Pure Rust, cross-platform (X11/Wayland/macOS/Windows/Web), `ApplicationHandler` trait is clean, EventLoop 2 API is well-designed, largest ecosystem adoption | API churn (breaking between versions), Wayland quirks on some compositors |
| **sdl2** | Battle-tested, game controller support, stable API | C bindings (not pure Rust), heavier, more game-oriented than desktop-app |
| **glfw** | Mature, explicit OpenGL context management | C bindings, OpenGL-centric, losing momentum |

**Recommendation: winit** — clear winner for wgpu integration. The `create_surface()` API takes a raw window handle from winit directly. The `ApplicationHandler` trait provides clean lifecycle. We target winit 0.30+ (latest stable).

### 1.2 GPU API: wgpu

| Library | Pros | Cons |
|---------|------|------|
| **wgpu** | Cross-platform (Vulkan/Metal/D3D12/GL), WebGPU standard, pure Rust, safety guarantees, built-in shader compilation from WGSL, large ecosystem | Slightly higher API verbosity, learning curve for graphics novices |
| **glow** | Simpler API, works everywhere OpenGL works | OpenGL state machine complexity, no Vulkan/Metal native, driver inconsistency |
| **glium** | High-level, easy to use | Unmaintained, depends on `glutin`, OpenGL-only, no async |

**Recommendation: wgpu v29** — the only serious option for a modern GPU-accelerated desktop app. WGSL shaders compile to all backends. The `wgpu::util` module provides `BufferInitDescriptor` and device helpers. Frame lifecycle is clean: `get_current_texture()` → `create_command_encoder()` → `begin_render_pass()` → draw calls → `encoder.finish()` → `queue.submit()` → `texture.present()`.

### 1.3 GPU Text Rendering: glyphon

| Library | Pros | Cons |
|---------|------|------|
| **glyphon** | Purpose-built for wgpu, cosmic-text integration, SwashCache for glyph rasterization, TextRenderer API is straightforward, atlas caching | Small ecosystem, limited layout control, docs are sparse |
| **cosmic-text** | Full text layout engine (bidi, shaping, line breaking) | Only a layout engine — needs a separate renderer |
| **swash** | High-quality font rasterization, subpixel rendering | Low-level, requires building a renderer around it |
| **egui** | Immediate mode GUI, built-in text, easy to overlay | Opinionated about event handling, heavy for just text labels, doesn't integrate into a layered render pass cleanly |

**Recommendation: glyphon on top of cosmic-text + swash.** glyphon already wraps cosmic-text for layout and swash for rasterization, and provides a wgpu `TextRenderer` that integrates into a render pass. The pattern is:
1. Create shared `Cache` and `FontSystem` at init
2. Create `TextAtlas` and `TextRenderer` per surface
3. Each frame: `text_renderer.prepare(device, queue, &mut font_system, &mut atlas, &viewport, text_areas, &mut swash_cache)`
4. In the render pass: `text_renderer.render(&atlas, &viewport, &mut rpass)`

This solves the text-as-vertices approach without writing custom SDF or distance-field shaders.

### 1.4 SIMD: portable_simd (stable since Rust 1.78)

| Library | Pros | Cons |
|---------|------|------|
| **`core::simd`** (portable_simd) | Stable in 1.96, zero deps, vendor-agnostic, auto-vectorization hinting, works on all architectures | Not always optimal for a specific CPU (no AVX-512 tuning without `is_x86_feature_detected!`) |
| **`wide`** | Simple API, `f32x8`, `f32x4` etc., proven in audio/ML | Extra dependency, mostly a wrapper around the same LLVM intrinsics |
| **`faster`** | High-level iterator-based SIMD | Less maintained, doesn't leverage stabilized `core::simd` |

**Recommendation: `core::simd` (portable_simd)** for indicator calculations. At Rust 1.96 this is stable and zero-dependency. Use `std::simd::f32x8` for SMA/EMA rolling calculations and covariance matrices. For hot paths that need architecture-specific tuning, use `cfg!(target_feature = "avx2")` with runtime fallback.

### 1.5 Time Handling: `time` crate

| Library | Pros | Cons |
|---------|------|------|
| **`time`** | Well-designed API, `TimeDelta`, `OffsetDateTime`, serde support, no panicking defaults, actively maintained | A bit verbose for simple use cases |
| **`chrono`** | Most popular, huge ecosystem, `NaiveDateTime` for non-tz | Legacy API warts, `Local::now()` panics without feature flags, timezone DB complexity |
| **hifitime** | Nanosecond precision, epoch handling, designed for scientific/aerospace | Overkill for OHLC time series, niche, fewer ecosystem integrations |

**Recommendation: `time` v0.3** for time-series timestamps. Use `time::OffsetDateTime` for bar/candle timestamps with nanosecond precision. `time::Duration` for timeframe definitions. `chrono` only if integration with an existing crate forces it.

### 1.6 Serialization: serde + rkyv (complementary)

| Library | Pros | Cons |
|---------|------|------|
| **serde** | Universal, `#[derive(Serialize, Deserialize)]`, all formats supported | Allocation-heavy for hot path, no zero-copy |
| **rkyv** | Zero-copy deserialization, memory-mappable archives, ARCHIVE trait, no allocations at read time | Schema evolution requires care, `bytecheck` for safety, fewer format backends |
| **bincode** | Compact binary format | No zero-copy, just a format for serde |

**Recommendation: both, for different purposes.**
- **serde** (with `serde_json`) for config files, external API communication, debugging
- **rkyv** (with `bytecheck`) for **hot-path time series data**: archive bars/ticks into a memory-mapped ring buffer, read directly from the archive without deserialization. This is critical for real-time data ingestion.

Pattern: `Bar` derives both `Serialize`/`Deserialize` AND `Archive`/`Serialize`/`Deserialize` (rkyv). Config uses serde JSON. Historical data files use rkyv archives. Live ticks use the rkyv archived form directly in the ring buffer.

### 1.7 Async Runtime: tokio (data layer only)

| Library | Pros | Cons |
|---------|------|------|
| **tokio** | The standard async runtime, TCP/UDP/WebSocket support for live data, channels for push-based updates | Does NOT belong in the render loop — rendering stays sync |
| **async-std** | Simpler API | Smaller ecosystem, losing momentum |

**Recommendation: tokio (async data layer) + sync rendering.** The architecture is:
- Dedicated tokio runtime for data fetching (WebSocket, file I/O, REST)
- Data pushed through `tokio::sync::mpsc` channels to the render thread
- The winit event loop polls the channel receiver as a `winit::event::UserEvent`
- **winit's event loop is NOT async** — it runs on its own sync loop. wgpu operations are sync (command encoding is CPU work, only `queue.submit` touches the GPU).

This hybrid model is proven: data pipeline is async, rendering stays deterministic and sync.

### 1.8 UI Overlays: Custom wgpu layers (NOT egui)

| Library | Pros | Cons |
|---------|------|------|
| **Custom wgpu layers** | Full control over compositing, no event loop conflicts, layering precisions | More code upfront |
| **egui** (wgpu backend) | Rapid UI development, rich widget library | Event loop contention with winit, opinionated input model, conflicts with custom render passes, awkward to embed inside a layered compositing pipeline |
| **druid** | Declarative | Unmaintained / replaced by Xilem, not production-ready |

**Recommendation: Custom wgpu for HUD + context panels.** The hex architecture already defines `ChartRenderer` as a port. The adapter renders all chart layers — including HUD, crosshair info, and indicator overlays — in the same wgpu render pass pipeline. egui is explicitly **not recommended** because its immediate-mode input model conflicts with the event-driven interaction handling and adds complexity to the layer compositing order.

---

## 2. Hexagonal Architecture

### 2.1 Domain Layer (innermost — zero external dependencies)

The domain crate/lib contains pure data structures and traits. It imports ONLY `core`, `alloc`, and potentially `std::simd` (which is part of std).

```
┌──────────────────────────────────────────────┐
│                 ADAPTERS                      │
│  (wgpu, winit, glyphon, tokio, data files)   │
│  ┌────────────────────────────────────────┐   │
│  │           APPLICATION                   │   │
│  │  (ChartController, UseCases)           │   │
│  │  ┌──────────────────────────────────┐  │   │
│  │  │           PORTS                  │  │   │
│  │  │  (traits: ChartRenderer,         │  │   │
│  │  │   DataProvider, Interaction)      │  │   │
│  │  │  ┌────────────────────────────┐  │  │   │
│  │  │  │         DOMAIN             │  │  │   │
│  │  │  │  Bar, Tick, Series,        │  │  │   │
│  │  │  │  Indicator trait,          │  │  │   │
│  │  │  │  Viewport, Scale,          │  │  │   │
│  │  │  │  Crosshair, SeriesType     │  │  │   │
│  │  │  └────────────────────────────┘  │  │   │
│  │  └──────────────────────────────────┘  │   │
│  └────────────────────────────────────────┘   │
└──────────────────────────────────────────────┘
```

**Domain types:**

```rust
// domain/bar.rs
#[derive(Clone, Debug, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug, PartialEq))]
pub struct Bar {
    pub timestamp: Timestamp,  // use time::OffsetDateTime
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

// domain/tick.rs
#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct Tick {
    pub timestamp: Timestamp,
    pub price: f64,
    pub volume: u64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
}

// domain/indicator.rs
pub trait Indicator {
    type Input;
    type Output;
    fn name(&self) -> &'static str;
    fn next(&mut self, input: &Self::Input) -> Self::Output;
    fn reset(&mut self);
}

// domain/viewport.rs
pub struct Viewport {
    pub time_range: Range<Timestamp>,
    pub price_range: Range<f64>,
}

// domain/scale.rs
pub enum ScaleType { Linear, Logarithmic }

pub struct Scale {
    pub scale_type: ScaleType,
    pub range: Range<f64>,   // input domain
    pub screen_range: Range<f32>,  // output pixel range
}
impl Scale {
    pub fn map(&self, value: f64) -> f32 { /* linear/log transform */ }
}

// domain/crosshair.rs
pub struct Crosshair {
    pub position: ScreenPoint,
    pub snap_to_data: bool,
}

// domain/series.rs
pub enum SeriesType { Bar, Candle, Line, Area, Baseline, Histogram }

// domain/error.rs
pub enum ChartError {
    InsufficientData,
    InvalidParameter(&'static str),
    DataGap,
    CalculationError,
}
```

**Key constraint:** These types NEVER import `wgpu`, `winit`, `tokio`, or any I/O crate. They are testable in isolation with `cargo test --lib` and zero dependencies beyond std.

### 2.2 Ports Layer (application interfaces)

Traits that define the boundaries. These live in a `ports/` module and import from domain.

```rust
// ports/render.rs
pub trait ChartRenderer {
    fn begin_frame(&mut self, viewport: &Viewport);
    fn draw_bar(&mut self, bar: &Bar, color: Rgba, width: f32);
    fn draw_line(&mut self, points: &[ScreenPoint], color: Rgba, width: f32);
    fn draw_text(&mut self, text: &str, position: ScreenPoint, style: &TextStyle);
    fn draw_rect(&mut self, rect: ScreenRect, color: Rgba, fill: bool);
    fn end_frame(&mut self) -> Result<(), RenderError>;
    fn resize(&mut self, size: ScreenSize);
}

// ports/data_provider.rs
#[async_trait]
pub trait DataProvider: Send + 'static {
    fn symbol(&self) -> &str;
    async fn load_historical(&self, timeframe: Timeframe, range: Range<Timestamp>)
        -> Result<Vec<Bar>, DataError>;
    async fn subscribe(&self, sender: tokio::sync::mpsc::Sender<MarketEvent>);
}

// ports/interaction.rs
pub trait InteractionHandler {
    fn on_mouse_move(&mut self, pos: ScreenPoint);
    fn on_scroll(&mut self, delta: ScrollDelta, pos: ScreenPoint);
    fn on_click(&mut self, button: MouseButton, pos: ScreenPoint);
    fn on_key(&mut self, key: KeyEvent);
    fn on_resize(&mut self, size: ScreenSize);
}
```

### 2.3 Application Layer (use cases)

The orchestration logic lives here. `ChartController` wires ports together:

```rust
// application/chart_controller.rs
pub struct ChartController {
    renderer: Box<dyn ChartRenderer>,
    data_provider: Box<dyn DataProvider>,
    indicator_services: Vec<Box<dyn IndicatorService>>,
    viewport: Viewport,
    series: SeriesStore,
    crosshair: Crosshair,
}

impl ChartController {
    pub fn handle_input(&mut self, event: InputEvent) { /* dispatches to viewport, crosshair, or indicator mgmt */ }
    pub fn render_frame(&mut self) { /* asks renderer to redraw */ }
    pub fn on_data(&mut self, bars: &[Bar]) { /* updates series, recalculates indicators, requests redraw */ }
}
```

### 2.4 Adapters Layer (outer ring)

Each port trait gets one or more adapter implementations:

| Port | Adapter(s) |
|------|-----------|
| `DataProvider` | `SimulatedDataProvider` (dev/testing), `FileDataProvider` (CSV/Parquet/rkyv archive), `LiveDataProvider` (WebSocket/FIX) |
| `ChartRenderer` | `WgpuRenderer` (glyphon + wgpu pipelines) |
| `InteractionHandler` | `WinitInputAdapter` (translates winit events → domain interaction commands) |
| `WindowLifecycle` | `WinitWindowAdapter` (surface creation, resize, DPI) |

### 2.5 Boundary Enforcement

- `domain/` has **`[lib.rs](http://lib.rs)` with no dev-dependencies beyond std**
- `ports/` depends on `domain/` only
- `application/` depends on `domain/` + `ports/`
- `adapters/` depends on everything (the outer ring that violates DIP)
- In the Cargo.toml: domain as a path dependency if using workspace, or careful `pub use` re-exports in a single crate

**CI enforcement:** `cargo check --package fc-types` must compile with zero GPU/async deps.

---

## 3. GPU Rendering Pipeline

### 3.1 Frame Lifecycle

```
winit requests redraw
  │
  ▼
ChartController::render_frame()
  │
  ├─► WgpuRenderer::begin_frame()
  │     ├─ surface.get_current_texture() → TextureView
  │     ├─ device.create_command_encoder()
  │     └─ encoder.begin_render_pass() for each layer
  │
  ├─► Layer 1: Grid (background grid lines)
  │     └─ draw_lines with viewport projection
  │
  ├─► Layer 2: Price Series
  │     └─ draw_line (for line type), draw_rect (for candles/bars)
  │
  ├─► Layer 3: Indicators
  │     └─ draw_line (overlays like SMA/EMA)
  │
  ├─► Layer 4: Drawings / Annotations (future)
  │
  ├─► Layer 5: Crosshair
  │     └─ draw_line (cross lines), draw_text (OHLC values)
  │
  ├─► Layer 6: HUD / Overlays
  │     └─ draw_text (symbol, timeframe, indicator values)
  │
  ├─► WgpuRenderer::end_frame()
  │     ├─ RenderPass::end()
  │     ├─ queue.submit(encoder.finish())
  │     └─ surface.present()
  │
  ▼
 GPU presents frame
```

### 3.2 Wgpu Pipeline Layout

There are multiple wgpu render pipelines, one per primitive type:

| Pipeline | Shader(s) | Topology | Use |
|----------|-----------|----------|-----|
| `line_pipeline` | `line.wgsl` (vertex + fragment) | `LineStrip` | Price lines, indicator lines, crosshair, grid |
| `candle_pipeline` | `candle.wgsl` (vertex + fragment) | `TriangleList` | OHLC candles (rect + wick) |
| `bar_pipeline` | `bar.wgsl` | `TriangleList` | OHLC bars (open-tick, close-tick, high-low line) |
| `fill_pipeline` | `fill.wgsl` | `TriangleList` | Area fill below line series |

**WGSL shader for lines (simplified):**
```wgsl
struct Uniforms {
    projection: mat4x4<f32>,  // viewport transform
}
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = uniforms.projection * vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
```

The projection matrix in the uniform buffer is the **viewport transform**: it maps data coordinates (price × time) to clip space. When the user zooms or pans, only this uniform is updated — the vertex buffers stay unchanged. This is the key GPU-accelerated transformation: **no vertex buffer rebuild on pan/zoom**.

### 3.3 Glyphon Integration

glyphon renders into the SAME render pass as the chart layers. The render order is:

```
Begin render pass (clear background)
  → Grid layer (glyphon not needed — just lines)
  → Price series layer
  → Indicator layer
  → Crosshair layer (glyphon for OHLC label)
  → HUD layer (glyphon for text overlays)
End render pass
```

glyphon's `TextRenderer::render()` takes `&mut RenderPass`, so it integrates as one more draw call in the sequence. The key is that glyphon text rendering happens AFTER geometry so text always draws on top.

For each text element (axis label, crosshair value, HUD text), we create a `TextArea` with `Buffer` (cosmic-text layout), position it in screen coordinates, and pass all areas to `prepare()` then `render()`.

### 3.4 Dirty Region Tracking

Full frame redraws are wasteful for:
- Crosshair movement (cursor follows mouse)
- Scrolling price labels
- Ticker/status updates

**Strategy:** Maintain a `dirty_rect: Option<ScreenRect>` in the renderer.
- On crosshair move: set `dirty_rect = Some(crosshair_old_union_new)`
- On data update (new bar): set `dirty_rect = Some(right_edge_rect)`
- On pan/zoom: set to `None` (full redraw required)

The render pass uses `LoadOp::Load` (not `Clear`) for partial redraws, and only clears the dirty region via a scissor rect. However, wgpu doesn't natively support per-region load — the simplest approach is:
1. **Full clear for pan/zoom/resize** (these are infrequent)
2. **Load-op without clear for cursor/crosshair** (overdraw the old crosshair with the clear color first, then draw the new one)

This is a performance optimization for later — start with full clears, optimize to load-op when profiling shows a bottleneck.

### 3.5 Layer Compositing

All layers render into the same texture (no separate framebuffers per layer — that would waste GPU memory and bandwidth). The layering is achieved by **draw order within a single render pass**, with depth testing disabled (2D orthographic).

Each layer rendering method takes `&mut RenderPass` and issues its own draw calls. The `WgpuRenderer::render_frame()` orchestrates the order.

---

## 4. Data Pipeline Performance

### 4.1 Ring Buffer for Time Series

```rust
// adapters/data/ring_buffer.rs
pub struct RingBuffer<T, const N: usize> {
    buffer: [MaybeUninit<T>; N],
    head: usize,   // write position
    count: usize,  // number of items
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub fn push(&mut self, item: T) { /* O(1) append, overwrites oldest */ }
    pub fn get(&self, index: usize) -> Option<&T>;
    pub fn iter(&self) -> RingIter<T, N>;
    pub fn slice(&self, range: Range<usize>) -> &[T];  // contiguous window
}
```

- Fixed capacity at compile time via const generics (e.g., `RingBuffer<Bar, 100_000>` for 100k 1m bars ≈ 69 days of 1m data in memory)
- `MaybeUninit<T>` avoids zero-initialization overhead
- For the hot tick buffer: `RingBuffer<Tick, 1_000_000>` (~64MB for 64-byte ticks)
- When the buffer wraps, stale data is silently overwritten (oldest data first)

### 4.2 Multi-Resolution Aggregation

| Resolution | Capacity | Retention | Source |
|-----------|----------|-----------|--------|
| Tick | 1,000,000 | ~1 hour (active market) | Live feed |
| 1m bar | 100,000 | ~69 days | Aggregated from ticks |
| 5m bar | 100,000 | ~347 days | Aggregated from 1m |
| 15m bar | 100,000 | ~3.9 years | Aggregated from 5m |
| 1H bar | 100,000 | ~11.4 years | Aggregated from 15m |
| 1D bar | 100,000 | ~273 years | Aggregated from 1H |

Aggregation pipeline: `TickAggregator` accumulates ticks into the current 1m bar; when the 1m bar closes, it's pushed to the 1m ring and cascades upward. This happens at the adapter level (data layer), not in domain.

### 4.3 SIMD Indicator Calculations

Indicators operate on the ring buffer's contiguous slice. Example — SMA with `core::simd`:

```rust
use std::simd::{f64x4, StdFloat};

fn sma_simd(prices: &[f64], period: usize) -> Vec<f64> {
    let mut result = Vec::with_capacity(prices.len());
    let mut sum = 0.0;
    
    // Warmup (first period-1 values)
    for &p in prices.iter().take(period - 1) {
        sum += p;
        result.push(f64::NAN);
    }
    
    // SIMD-accelerated sliding window
    // For each chunk of 4, compute partial sums
    let chunks = prices.windows(period).collect::<Vec<_>>();
    // In practice: use chunk_exact(4) on the window starts
    for window in prices.windows(period) {
        sum += window[period - 1] - window[0]; // O(1) update
        result.push(sum / period as f64);
    }
    
    result
}
```

For EMA (exponential), the recursive nature limits SIMD speedup, but `core::simd` still helps for parallel processing of independent indicator series.

SIMD targets: `f64x4` (256-bit AVX2) for price arrays. Runtime detection via `is_x86_feature_detected!("avx2")`.

### 4.4 Event-Driven Push Model

```
Live Data Source (WebSocket)
  │
  ▼
LiveDataProvider (tokio task)
  │  parses → Tick
  │
  ▼
tokio::sync::mpsc::Sender<MarketEvent>
  │  MarketEvent::Tick(Tick), MarketEvent::BarClosed(Bar)
  │
  ▼
ChartController::on_data()
  │  updates ring buffer, recalculates indicators, marks dirty rect
  │
  ▼
request_redraw() → winit render loop
```

The pull model (polling) is used only for historical data loading on startup.

### 4.5 Data Normalization

At ingestion boundary (inside the `DataProvider` adapter), ticks are normalized:
- Timestamp → `time::OffsetDateTime` in UTC
- Prices → `f64` (no decimal normalization needed for display)
- Volume → `u64`
- Invalid/missing data → filtered, gaps tracked in `DataGap` error variant

This ensures the domain layer never sees raw "string price" or malformed data.

---

## 5. User Interaction Model

### 5.1 Interaction Flow

```
winit WindowEvent
  │
  ▼
WinitInputAdapter (translates winit → domain)
  │
  ├─ CursorMoved(pos) → InteractionEvent::MouseMove(ScreenPoint { x, y })
  ├─ MouseWheel { delta, phase } → InteractionEvent::Scroll(ScrollDelta { lines_x, lines_y })
  ├─ MouseInput { button, state } → InteractionEvent::Click(MouseButton, ScreenPoint)
  └─ KeyboardInput { key, state } → InteractionEvent::Key(KeyEvent)
  │
  ▼
ChartController::handle_input(event)
  │
  ├─ MouseMove → Crosshair::update_position(pos)
  │              → if snap_to_data: find nearest bar, show OHLC
  │              → request_redraw() (partial, crosshair only)
  │
  ├─ Scroll → Viewport::zoom_at_cursor(factor, cursor_pos)
  │           → Scale::update_range(new_domain)
  │           → request_redraw() (full)
  │
  ├─ Click+drag → Viewport::pan(delta_pixels / scale_factor)
  │               → request_redraw() (full)
  │
  └─ Key(period) → Viewport::set_timeframe(Timeframe::OneMinute ... OneDay)
                   → reload data from DataProvider
                   → request_redraw() (full)
```

### 5.2 Viewport Operations

| Operation | Frequency | GPU Impact | CPU Impact |
|-----------|-----------|------------|------------|
| Crosshair move | Very high (per mouse move) | Partial redraw (dirty rect) | None (just uniform update or overlay) |
| Scroll zoom | Medium | Update projection uniform | Recalculate visible range |
| Pan/drag | Medium | Update projection uniform | None |
| Timeframe change | Low | Full redraw | New data load, indicator recalculation |
| New bar/tick | Medium (per incoming data) | Partial (right edge) | Append to buffer, update indicator |

### 5.3 Zoom/Pan GPU Acceleration

The critical insight: **vertex buffers for price data do NOT change on zoom or pan.**

The projection matrix in the uniform buffer encodes the viewport transform:
```
projection = ortho(
    viewport.time_range.start, viewport.time_range.end,
    viewport.price_range.start, viewport.price_range.end
)
```

When the user scroll-zooms: only the uniform buffer is updated via `queue.write_buffer()`. The vertex buffer with all historical data points remains untouched. This is the defining performance advantage of GPU-accelerated charting over Canvas2D/SVG approaches.

---

## 6. Project Structure

### 6.1 Single Crate vs Workspace

| Aspect | Single Crate | Workspace |
|--------|-------------|-----------|
| Compile parallelism | Limited (one crate `lib.rs`) | High (each crate compiles in parallel) |
| LTO effectiveness | Full crate LTO, better optimization | Less aggressive cross-crate inlining |
| Dependency management | One `Cargo.toml` | Each crate has its own, some duplication |
| Boundary enforcement | Module visibility only (can be bypassed) | Crate-level `pub use` gating |
| Test isolation | `#[cfg(test)]` modules | `cargo test -p fc-types` |
| Build on change | Rebuilds entire project | Only changed crates |
| First-build time | ~same | ~same (all crates built) |
| Incremental | Rebuilds more | Rebuilds less |

**Recommendation: Start as a workspace with 3 crates, merge if needed.**

```
fast-chart/
├── Cargo.toml                  # workspace root
├── openspec/
├── fc-types/          # Zero-dependency pure domain
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── bar.rs
│       ├── tick.rs
│       ├── series.rs
│       ├── indicator.rs
│       ├── viewport.rs
│       ├── scale.rs
│       ├── crosshair.rs
│       └── error.rs
├── fast-chart-core/            # Ports + Application
│   ├── Cargo.toml              # depends on domain
│   └── src/
│       ├── lib.rs
│       ├── ports/              # traits
│       │   ├── mod.rs
│       │   ├── render.rs
│       │   ├── data_provider.rs
│       │   └── interaction.rs
│       └── app/                # use cases
│           ├── mod.rs
│           └── chart_controller.rs
└── fast-chart-app/             # Adapters + main
    ├── Cargo.toml              # depends on core + wgpu, winit, glyphon, tokio, etc.
    └── src/
        ├── main.rs
        ├── adapters/
        │   ├── mod.rs
        │   ├── rendering/
        │   ├── input/
        │   ├── data/
        │   └── config/
        └── shaders/            # WGSL files (included via include_wgsl!)
```

**Rationale:**
- `fc-types` compiles in < 1s, zero deps — devs iterate on data structures instantly
- `fast-chart-core` compiles without GPU deps — continuous integration in CI without GPU
- `fast-chart-app` is the heavy compilation target (wgpu, glyphon, winit, tokio) — only rebuilt when adapters change
- Cross-crate boundary enforcement means `domain` types cannot accidentally import `wgpu`

**Future merge path:** If incremental builds are fast enough (sub-5s), workspace overhead may not justify the split. The split is safer upfront; merging is easy (move `src/` up), splitting is harder.

### 6.2 WGSL Shader Organization

Shaders live in `fast-chart-app/src/shaders/` and are included at compile time:

```rust
let shader = device.create_shader_module(wgpu::include_wgsl!("../../shaders/line.wgsl"));
```

Shader modules:
```
shaders/
├── line.wgsl         # LineStrip pipeline
├── candle.wgsl       # Candle OHLC pipeline
├── fill.wgsl         # Area fill pipeline
└── text.wgsl         # (glyphon provides its own)
```

### 6.3 Module Dependency Graph

```
fc-types
    ↑ (no deps)
    │
fast-chart-core
    ↑ (depends on domain + port types)
    │
fast-chart-app
    ↑ (depends on core + all external crates)
```

---

## 7. Recommendation Summary

### 7.1 Libraries (Final Table)

| Concern | Choice | Version | Rationale |
|---------|--------|---------|-----------|
| Windowing | **winit** | 0.30+ | Pure Rust, wgpu integration, mature event loop |
| GPU API | **wgpu** | 29.x | Cross-platform, WGSL, safety, ecosystem |
| Text GPU | **glyphon** | 0.5+ | wgpu-native TextRenderer, cosmic-text + swash |
| SIMD | `core::simd` (std) | stable | Zero deps, portable, stable at 1.96 |
| Time | **time** | 0.3 | Clean API, serde, no panicking defaults |
| Serialization | **serde** + **rkyv** | serde 1.x, rkyv 0.8 | serde for config, rkyv for hot data path |
| Async | **tokio** | 1.x | Standard, WebSocket support, mpsc channels |
| HUD/Overlay | **Custom wgpu** | — | Avoid egui event-loop conflicts |

### 7.2 Hex Architecture Decision

**HEXAGONAL — Yes, full ports & adapters.** The domain layer is genuinely framework-independent (trading data structures don't change with rendering technology). This is a textbook case where hexagonal architecture pays off:
- Testing: domain logic tested without a GPU
- Replaceability: swap `wgpu` for `vello` or raw Vulkan later
- Clarity: `ChartRenderer` trait defines exactly what drawing primitives the chart needs

### 7.3 Performance Strategy

1. **Ring buffers with rkyv** for zero-copy time series access
2. **GPU projection transform** for free zoom/pan (no vertex rebuild)
3. **SIMD via `core::simd`** for rolling indicator calculations
4. **Multi-resolution aggregation** to bound memory usage
5. **Event-driven push model** for real-time data (no polling)
6. **Draw-order layering** in a single render pass (no FBO switching)

### 7.4 Project Bootstrap Sequence

```
Phase 1: Domain crate + tests
  - Bar, Tick, Indicator trait, Viewport, Scale, Crosshair
  - 100% test coverage
  - No GPU needed

Phase 2: Core crate (ports + controller)
  - ChartRenderer trait, DataProvider trait
  - ChartController orchestrator
  - SimulatedDataProvider for testing

Phase 3: Winit + wgpu window
  - Window creation, surface config
  - Basic triangle on screen
  - Event loop integration

Phase 4: Line pipeline + bar data
  - WGSL line shader
  - Simulated price data rendered as line
  - Viewport projection uniform → pan/zoom works

Phase 5: Candle pipeline
  - OHLC candle geometry
  - Volume histogram

Phase 6: Text (glyphon)
  - Axis labels, price labels
  - Crosshair OHLC tooltip

Phase 7: Indicators
  - SMA, EMA, RSI indicator implementations
  - SIMD acceleration

Phase 8: Real data sources
  - FileDataProvider (CSV/rkyv archive)
  - LiveDataProvider (WebSocket)
  - Multi-resolution aggregation
```

---

## 8. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| wgpu API churn between versions | Medium | Medium | Pin exact version in Cargo.lock, isolate behind `ChartRenderer` trait so swap is adapter-only |
| glyphon breaking changes | Medium | Medium | glyphon is a thin wrapper — could fall back to custom text rendering with swash if needed |
| Performance of f64 on GPU (vs f32) | Low | Medium | Use f32 for rendering (screen space), keep f64 for domain calculations. Shader projection matrix is f32 (screen coords don't need f64) |
| wgpu `present_mode` vsync latency | Low | High | Start with `AutoVsync`, allow switching to `AutoNoVsync` for low-latency mode. Profile to see if it matters for chart rendering |
| Window compositor tearing (Wayland) | Medium | Low | wgpu handles buffer swapping. Use `Fifo` vsync on Wayland for smooth rendering |
| Workspace complexity overhead | Low | Low | If workspace becomes friction, merge into single crate. This is a one-line change |
| rkyv schema evolution for persisted data | Medium | Medium | Use `#[rkyv(with = ...)]` for forward-compatibility, version-tag archives |
| SIMD portability on ARM (Apple Silicon) | Low | Low | `core::simd` handles this — `f64x2` on NEON, `f64x4` on AVX2. Runtime fallback for non-SIMD paths |

---

## 9. Next Steps

### Ready for Proposal Phase?

**Yes** — the architecture is sufficiently explored to move to the proposal phase. The decisions with the highest uncertainty have been resolved:
- ✅ Library choices aligned to wgpu ecosystem
- ✅ Hexagonal boundary definitions
- ✅ Workspace vs single crate: workspace (3 crates)
- ✅ Rendering approach: single-pass, multi-pipeline, uniform-driven zoom
- ✅ Data pipeline: ring buffers + rkyv + tokio push model

### Recommended Phase Order

```
1. sdd-propose   → formal change proposal (architecture-foundation)
2. sdd-spec      → requirement specs per domain
3. sdd-design    → detailed design for Phase 1 (domain crate)
4. sdd-tasks     → task breakdown for Phase 1
5. sdd-apply     → implement Phase 1
```

### Open Questions for Proposal Phase

- Should `Scale` support `f64` or `f32` for the mapping function? (Performance vs precision tradeoff — recommend using f64 in domain, f32 only in renderer)
- Font choice for glyphon? (System fonts vs bundled font file)
- Default color scheme? (Light vs dark, configurable)
- What's the MVP timeframe definition that unblocks work? (1m bars from simulated data is sufficient)

---

*End of exploration document.*
