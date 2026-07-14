# Design: fast-chart — Native GPU-Accelerated Trading Chart

## Technical Approach

3-crate hexagonal Rust workspace enforcing architecture at compile time. `fast-chart-domain` (zero deps) defines pure types. `fast-chart-core` defines port traits + `ChartController` orchestration. `fast-chart-app` implements wgpu/winit/glyphon adapters. Data flows async (tokio mpsc) from providers into ring buffers; render loop is synchronous. Zoom/pan updates a projection uniform only — no vertex buffer rebuild. SIMD via `core::simd` for indicator math. See [proposal](../proposal.md) for scope, [exploration](../../design/exploration.md) for library decisions, and [specs](../../specs/) for requirements.

## Architecture Decisions

### 1. 3-crate workspace

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Single crate | Faster LTO, simpler deps | ❌ |
| **Workspace (3 crates)** | Compile-time boundary enforcement, parallel compilation, domain CI without GPU | ✅ |

**Rationale**: Domain crate compiles in < 1s; core crate builds without GPU deps for CI on headless runners; app crate isolates wgpu's heavy compilation. Merging is trivial (move `src/` up) — splitting after coupling is not.

### 2. Ring buffer design

| Option | Tradeoff | Decision |
|--------|----------|----------|
| `VecDeque` | Allocation overhead, runtime capacity | ❌ |
| **`MaybeUninit<[T; N]>`** | Zero-init-free, const generic, O(1) push/drain | ✅ |

**Rationale**: Const generic capacity avoids allocation at runtime. `MaybeUninit` avoids initializing 100k slots. Slice access via `index + modulo` + contiguous window copy.

### 3. GPU rendering architecture

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Multiple render passes | FBO switch overhead | ❌ |
| **Single pass, draw-order** | No FBO switching, layers composited by draw order | ✅ |

**Rationale**: Lines, candles, fills share one `RenderPass`. Order: grid → series → indicators → crosshair → HUD. Projection uniform `mat4x4<f32>` in a single uniform buffer updated via `queue.write_buffer()` on zoom/pan — vertex buffers never change.

### 4. winit event loop + wgpu integration

| Option | Tradeoff | Decision |
|--------|----------|----------|
| tokio on render thread | Render loop becomes async, complicates GPU sync | ❌ |
| **`ControlFlow::Poll` + mpsc receiver** | Sync render loop, data thread pushes via mpsc | ✅ |

**Rationale**: wgpu is sync — `get_current_texture()`, command encoding, `submit()` are CPU work. Data provider runs on a separate tokio runtime; pushes `DataEvent` through `mpsc::Receiver` polled in `about_to_wait()`.

### 5. Indicator SIMD

| Option | Tradeoff | Decision |
|--------|----------|----------|
| `wide` crate | Extra dep, wrapper around same intrinsics | ❌ |
| **`core::simd` (portable_simd)** | Zero-dependency, stable at 1.96, portable | ✅ |

**Rationale**: `f64x4` for 256-bit AVX2 rolling windows, `f64x2` on NEON. Every indicator has a scalar fallback proven bit-identical within 1e-12 via test.

### 6. Multi-resolution cascade

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Single-ring, re-sample on demand | CPU cost per zoom level change | ❌ |
| **Cascade: Tick→1m→5m→15m→1H→1D** | Memory per tier, O(1) resolution switch | ✅ |

**Rationale**: Each aggregator reads from the tier below and pushes OHLC to its own ring when its interval closes. Zoom/pan reads from the active tier ring — no recomputation.

### 7. Data push model

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Polling (render loop pulls) | Busy-wait, high latency | ❌ |
| **tokio mpsc channel** | Async push, `send().await` backpressure | ✅ |

**Rationale**: `tokio::sync::mpsc` with capacity 1024. Render loop calls `try_recv()` in `about_to_wait()` to drain without blocking. Backpressure via bounded channel.

### 8. Text rendering

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Raw swash + custom SDF | Full control, high dev cost | ❌ |
| **glyphon (cosmic-text + swash wgpu)** | Integrates into single render pass, atlas caching | ✅ |

**Rationale**: `TextRenderer::prepare()` + `::render(&mut rpass)` slots into existing pipeline. `FontSystem` + `SwashCache` shared at init. Fallback to raw swash if glyphon breaks.

### 9. Configuration system

| Option | Tradeoff | Decision |
|--------|----------|----------|
| JSON | No comments, no inline docs | ❌ |
| **TOML + serde + `ChartConfig::default()`** | Comments, partial merge, hot-reload via notify | ✅ |

**Rationale**: `serde::Deserialize` with `#[serde(default)]` so partial configs merge cleanly. `notify` crate file watcher posts `ConfigReloadEvent` on change. Invalid files log error and keep previous.

### 10. Pane layout

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Absolute positioning per pane | Complex resize math | ❌ |
| **LayoutManager, vertical stack** | Shared x-axis, independent y-scales, draggable dividers | ✅ |

**Rationale**: `Pane { viewport, series, indicators }` — x-axis tied to shared `Viewport.time_range`, y-axis per pane. Dividers enforce `min_height` configurable per pane.

## Data Flow

### Startup
```
main → winit::EventLoop → wgpu::Instance::create_surface()
  → device/queue init → create render pipelines (line, candle, fill)
  → load ChartConfig from TOML → create LayoutManager
  → SimulatedDataProvider::subscribe() → mpsc channel
  → winit event loop: ControlFlow::Poll
```

### Frame render
```
winit::Event::RedrawRequested
  → ChartController::handle_frame()
    → for each pane (ordered top→bottom):
        update projection uniform (queue.write_buffer)
        rpass.set_pipeline(grid) → draw grid lines
        rpass.set_pipeline(line/candle) → draw price series
        rpass.set_pipeline(line) → draw indicator overlays
        draw crosshair lines
        glyphon TextRenderer::render → HUD text
    → encoder.finish() → queue.submit() → surface.present()
```

### Data ingestion
```
DataProvider (tokio task)
  → mpsc::Sender::send(DataEvent::BarClosed(bar))
  → mpsc::Receiver (polled in about_to_wait)
  → TimeSeries::push(bar)
  → cascade: if bar closes, push to next tier
  → mark dirty_rect → window.request_redraw()
```

### Interaction
```
winit::WindowEvent
  → WinitInputAdapter::on_mouse_move/on_scroll/on_click
  → InteractionCommand (UpdateCrosshair | ZoomAtCursor | PanBy)
  → ChartController::handle_input
  → updates Viewport.time_range or Crosshair.position
  → if viewport changed: queue.write_buffer(projection_uniform)
  → window.request_redraw()
```

### Indicator calculation
```
DataProvider push → IndicatorService::calculate(indicators, series)
  → for each indicator in registry:
      scalar or SIMD path (cfg!(target_feature = "avx2"))
      push result to overlay ring
  → mark indicator pane dirty → redraw
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `fast-chart-domain/src/lib.rs` | Create | Module exports |
| `fast-chart-domain/src/bar.rs` | Create | `Bar` struct with validation |
| `fast-chart-domain/src/tick.rs` | Create | `Tick` struct |
| `fast-chart-domain/src/series.rs` | Create | `TimeSeries<T, N>` ring buffer |
| `fast-chart-domain/src/indicator.rs` | Create | `Indicator` trait |
| `fast-chart-domain/src/viewport.rs` | Create | Visible range + zoom |
| `fast-chart-domain/src/scale.rs` | Create | Domain↔screen mapping |
| `fast-chart-domain/src/crosshair.rs` | Create | Cursor position + snap |
| `fast-chart-domain/src/series_type.rs` | Create | `SeriesType` enum |
| `fast-chart-domain/src/error.rs` | Create | `ChartError` enum |
| `fast-chart-domain/Cargo.toml` | Create | Zero external deps |
| `fast-chart-core/src/lib.rs` | Create | Module exports |
| `fast-chart-core/src/ports/mod.rs` | Create | Port re-exports |
| `fast-chart-core/src/ports/render.rs` | Create | `ChartRenderer` trait |
| `fast-chart-core/src/ports/data_provider.rs` | Create | `DataProvider` trait |
| `fast-chart-core/src/ports/interaction.rs` | Create | `InteractionHandler` trait |
| `fast-chart-core/src/app/mod.rs` | Create | App module |
| `fast-chart-core/src/app/chart_controller.rs` | Create | Orchestration |
| `fast-chart-core/src/app/indicator_service.rs` | Create | Indicator registry + calc |
| `fast-chart-core/src/app/viewport_management.rs` | Create | Zoom/pan logic |
| `fast-chart-core/Cargo.toml` | Create | Depends on domain only |
| `fast-chart-app/src/main.rs` | Create | Entry point + event loop |
| `fast-chart-app/src/adapters/mod.rs` | Create | Adapter re-exports |
| `fast-chart-app/src/adapters/rendering/mod.rs` | Create | Renderer module |
| `fast-chart-app/src/adapters/rendering/wgpu_renderer.rs` | Create | `WgpuRenderer` impl |
| `fast-chart-app/src/adapters/rendering/pipelines.rs` | Create | Pipeline creation |
| `fast-chart-app/src/adapters/rendering/text_renderer.rs` | Create | glyphon integration |
| `fast-chart-app/src/adapters/rendering/layers.rs` | Create | Draw order |
| `fast-chart-app/src/adapters/rendering/shaders/line.wgsl` | Create | Line shader |
| `fast-chart-app/src/adapters/rendering/shaders/candle.wgsl` | Create | Candle shader |
| `fast-chart-app/src/adapters/rendering/shaders/fill.wgsl` | Create | Area fill shader |
| `fast-chart-app/src/adapters/input/mod.rs` | Create | Input module |
| `fast-chart-app/src/adapters/input/winit_input.rs` | Create | `WinitInputAdapter` |
| `fast-chart-app/src/adapters/data/mod.rs` | Create | Data module |
| `fast-chart-app/src/adapters/data/simulated.rs` | Create | `SimulatedDataProvider` |
| `fast-chart-app/src/adapters/data/rkyv_archive.rs` | Create | Zero-copy persist |
| `fast-chart-app/src/config/mod.rs` | Create | Config module |
| `fast-chart-app/src/config/chart_config.rs` | Create | `ChartConfig` + serde |
| `fast-chart-app/Cargo.toml` | Create | wgpu/winit/glyphon/tokio deps |
| `Cargo.toml` (root) | Create | Workspace definition |

## Interfaces / Contracts

```rust
// DataProvider port
#[async_trait]
pub trait DataProvider: Send + 'static {
    async fn subscribe(&mut self) -> mpsc::Receiver<DataEvent>;
    fn name(&self) -> &str;
}

// ChartRenderer port
pub trait ChartRenderer: Send {
    fn render(&mut self, frame: &mut Frame, state: &ChartState) -> Result<()>;
    fn resize(&mut self, width: u32, height: u32);
}

// InteractionHandler port
pub trait InteractionHandler: Send {
    fn handle_event(&mut self, event: WindowEvent, viewport: &mut Viewport)
        -> InteractionResult;
}

// ChartController — single orchestrator
pub struct ChartController {
    renderer: Box<dyn ChartRenderer>,
    data_provider: Box<dyn DataProvider>,
    interaction: Box<dyn InteractionHandler>,
    layout: LayoutManager,
    indicators: IndicatorRegistry,
    crosshair: Crosshair,
    config: ChartConfig,
}
```

## Testing Strategy

| Layer | What | How |
|-------|------|-----|
| Domain | Bar/Tick validation, TimeSeries push/pop/drain, Scale mapping, Viewport zoom/pan, Crosshair snap, Indicator trait dispatch | Unit tests, `cargo test -p fast-chart-domain` |
| Core | ChartController orchestration with mocked ports, IndicatorService registration & calc dispatch, viewport management | Mock trait impls, `cargo test -p fast-chart-core` |
| App | wgpu pipeline creation (smoke), glyphon text atlas init, SimulatedDataProvider generation correctness, config TOML round-trip | Integration tests, headless wgpu instance |
| E2E | Full render pipeline, interaction sequences, multi-pane layout rendering | Manual window verification, screenshot comparison (future) |

## Threat Matrix

N/A — no routing, shell, subprocess, VCS/PR automation, executable-file classification, or process-integration boundary. This is a greenfield desktop GUI application.

## Migration / Rollout

No migration required — greenfield project. All crates are new. Implementation follows the 15-phase plan defined in the task brief, beginning with domain crate (Phase 1) and progressing through app integration (Phase 15).

## Open Questions

- [ ] Font choice for glyphon — system fonts vs bundled font file?
- [ ] Default color scheme — dark theme with specific hex values?
- [ ] MVP timeframe definition — 1m bars from simulated data sufficient?
