# fast-chart Refactor & Development Roadmap

## Vision

**fast-chart** will be Rust's definitive answer to TradingView Lightweight Charts — not a web wrapper, not a binding, but a native, GPU-accelerated financial charting **library** that applications embed. The target is visual quality indistinguishable from TradingView, with an architecture that surpasses it through Rust's safety, performance, and extensibility guarantees.

Every decision must pass one test: _"Does this bring us closer to being the best charting library in any language?"_

---

## Fundamental Shift: Application → Library

### The Problem

fast-chart currently ships as an **application**, not a library. The `fast-chart-app` crate owns the window, event loop, GPU surface, and rendering pipeline — consumers cannot embed it. The public API exposes `GpuRenderer`, `winit::Window`, `ChartController`, and `ChartState` as concrete types, meaning anyone wanting a chart must adopt the entire winit + wgpu stack as-is.

This is backwards. A charting **library** should let users bring their own window, event loop, and GPU context. The library owns the **chart logic** and produces **render commands** — the host application owns the GPU surface and executes them.

### What Changes

| Current (Application) | Target (Library) |
|---|---|
| `fast-chart-app` = binary with `main()` | `fast-chart` = library crate (no binary) |
| Library owns `winit::Window` | Host owns the window |
| Library owns `wgpu::Device` | Host owns the device |
| `GpuRenderer` has `render()` with hardcoded passes | `SeriesRenderer` trait + `DrawCommand` queue |
| `DrawLayer` is a 2-variant stub | `DrawLayer` = 15-layer enum with ZIndex ordering |
| Rendering logic scattered across 12 sub-renderers | Renderers produce `DrawCommand`s, host batches & submits |
| No coordinate transform pipeline | World → Logical → Chart → Pane → Screen → GPU |
| `ChartState` is a flat struct | Per-pane state with independent viewport/scale/layers |
| wgpu/winit deps locked into core | Core is dep-free (only domain); renderer is a separate crate |

### The Goal After Refactor

```
User's application
    │
    ├── fast-chart (domain + core logic, zero GPU deps)
    │     ChartBuilder::new()
    │       .add_pane(|p| p.series(CandleSeries::new(data)))
    │       .add_pane(|p| p.indicator(Rsi::new(14)))
    │       .build()
    │       .handle_event(event)  → Option<DrawCommands>
    │       .render()            → Vec<DrawCommand>
    │
    └── User's renderer (wgpu, glow, software — whatever they want)
          for cmd in draw_commands { execute(cmd); }
```

---

## Architecture After Refactor

```
fast-chart-domain          Pure types, zero dependencies
                           Bar, Tick, Viewport, LinearScale, TimeScale,
                           Crosshair, PriceScale, Marker, Drawing tools,
                           Indicator trait + implementations, Error types

fast-chart                 Library crate — chart logic, rendering abstractions
                           ChartBuilder, PaneEngine, LayoutEngine,
                           SeriesRenderer trait, DrawCommand queue,
                           CoordinatePipeline, InteractionEngine,
                           CacheManager, AnimationEngine

fast-chart-renderer-wgpu   Optional GPU renderer (depends on wgpu)
                           WgpuBackend: implements RendererBackend trait
                           Sub-renderers for each series type
                           WGSL shaders, text rendering (glyphon)

fast-chart-examples        Example applications (winit + wgpu)
                           Simple chart, multi-pane, custom series,
                           drawing tools demo, real-time data
```

**Dependency direction**: `fast-chart-domain` ← `fast-chart` ← `fast-chart-renderer-wgpu` → `fast-chart-examples`

---

## Phase 0: Foundation Refactor — Library Identity

> **Goal**: Stop being an application. Establish the library crate as the single public surface.

### Objectives

1. `fast-chart` (root crate) becomes the **library** — no binary, no winit, no wgpu dependency
2. `fast-chart-app` is renamed to `fast-chart-examples` (or removed)
3. Domain crate is stable and complete
4. Core crate is renamed to be the library itself (or merged into root)

### PRs

| # | Task | Complexity | Files |
|---|---|---|---|
| 0.1 | Rename `fast-chart-core` → `fast-chart` (the library crate). Update root `Cargo.toml` workspace members. | M | `Cargo.toml`, `fast-chart/Cargo.toml` |
| 0.2 | Create `fast-chart-examples/` crate. Move `fast-chart-app/src/main.rs` + adapters into it. Update workspace members. | M | `fast-chart-examples/Cargo.toml`, `fast-chart-examples/src/main.rs` |
| 0.3 | Remove wgpu/winit deps from the library crate. `GpuRenderer`, all sub-renderers, WGSL shaders, `winit::Window` references move to `fast-chart-examples` (temporarily) until `fast-chart-renderer-wgpu` is created. | S | `fast-chart/Cargo.toml`, adapter files |
| 0.4 | Strip the `ChartRenderer` port trait of wgpu specifics. It should reference abstract rendering concepts only. | S | `fast-chart/src/ports/render.rs` |
| 0.5 | Add `#[cfg(doctest)]` documentation examples to all public types in `fast-chart-domain`. Ensure `cargo test --doc` passes. | M | `fast-chart-domain/src/*.rs` |
| 0.6 | Add `docs/` directory with `architecture/overview.md` explaining the library's hexagonal boundaries. | S | `docs/architecture/overview.md` |
| 0.7 | Establish CI gate: library crate must compile with `--no-default-features` and zero wgpu/winit deps. | S | `.github/workflows/ci.yml` |

### Dependencies

None. This is the foundation.

### Migration Notes

- The `fast-chart-core/src/app/` directory (chart_controller, pane, layout_manager, viewport_management, frame_counter, indicator_service) moves into `fast-chart/src/app/`.
- The `fast-chart-core/src/ports/` directory moves into `fast-chart/src/ports/`.
- All re-exports from `fast-chart-domain` remain in `fast-chart/src/lib.rs` (gateway pattern preserved).
- `fast-chart-app` adapter code moves to `fast-chart-examples/` and is no longer part of the library's public surface.

### Success Criteria

- [ ] `cargo build -p fast-chart` compiles with zero wgpu/winit deps
- [ ] `cargo build -p fast-chart-examples` compiles (uses wgpu + winit)
- [ ] `cargo test --workspace` passes (554+ tests)
- [ ] No `unsafe` outside of `fast-chart-renderer-wgpu` (future crate)
- [ ] `fast-chart/src/lib.rs` has zero rendering-specific imports

---

## Phase 1: Core Abstractions — The Rendering Contract

> **Goal**: Define the traits and types that make the library renderer-agnostic. Any backend (wgpu, glow, Skia, software) can implement them.

### Objectives

1. `SeriesRenderer` trait — every series type implements this
2. `DrawCommand` enum — the universal render queue primitive
3. `DrawLayer` system — proper ZIndex ordering with 15+ layers
4. `CoordinatePipeline` — World → Screen transform abstraction
5. `RendererBackend` trait — what the host must provide

### PRs

| # | Task | Complexity | Files |
|---|---|---|---|
| 1.1 | Define `DrawCommand` enum in library: `DrawLine`, `DrawRect`, `DrawCircle`, `DrawPath`, `DrawText`, `DrawImage`, `DrawTriangle`, with position, color, width, style, z_index, clip_rect fields. | L | `fast-chart/src/render/commands.rs` |
| 1.2 | Define `DrawLayer` enum with 15 layers (Background, Watermark, Grid, PriceScale, TimeScale, Indicators, Candles, Volume, CustomSeries, Drawings, Crosshair, Selection, FloatingLabels, Tooltip, Cursor). Each variant carries a ZIndex. | S | `fast-chart/src/render/layers.rs` |
| 1.3 | Define `SeriesRenderer` trait: `fn update(&mut self, data, viewport) -> Vec<DrawCommand>`, `fn hit_test(&self, point) -> Option<SeriesHit>`, `fn bounds(&self) -> Rect`. | L | `fast-chart/src/render/series_renderer.rs` |
| 1.4 | Define `RendererBackend` trait: `fn execute(&mut self, commands: &[DrawCommand])`, `fn resize(&mut self, w, h)`, `fn set_clip(&mut self, rect)`, `fn clear_clip(&mut self)`. | M | `fast-chart/src/render/backend.rs` |
| 1.5 | Implement `CoordinatePipeline` struct: `World → Logical → Chart → Pane → Screen → GPU`. Methods: `world_to_screen(point, viewport, pane_layout) -> ScreenPoint`, `screen_to_world(point, ...) -> WorldPoint`. All transforms use `floor(x) + 0.5` for pixel-perfect alignment. | L | `fast-chart/src/render/coordinates.rs` |
| 1.6 | Define `RenderContext` struct passed to all renderers: contains coordinate pipeline, clip rect, pane bounds, time/value ranges, DPI scale factor, timestamp. | M | `fast-chart/src/render/context.rs` |
| 1.7 | Add `mod render` to library's `lib.rs`. Re-export key types: `DrawCommand`, `DrawLayer`, `SeriesRenderer`, `RendererBackend`, `CoordinatePipeline`. | S | `fast-chart/src/lib.rs` |
| 1.8 | Write unit tests for `CoordinatePipeline`: world→screen, screen→world roundtrip, pixel-perfect alignment, multi-pane transforms. | M | `fast-chart/src/render/coordinates.rs` (tests) |

### Dependencies

Phase 0 complete (library crate exists, no GPU deps).

### Migration Notes

- The `fast-chart-app/src/adapters/rendering/layers.rs` 16-line stub is replaced by the new `DrawLayer` enum in the library.
- Existing sub-renderers in `fast-chart-app` (or `fast-chart-examples`) will eventually implement `SeriesRenderer`, but that happens in Phase 6 when the wgpu renderer is refactored.
- The current `ChartRenderer` port trait (1 method: `resize`) is superseded by `RendererBackend`.

### Success Criteria

- [ ] `DrawCommand` can express every visual primitive the spec requires
- [ ] `SeriesRenderer` trait is object-safe (can be `Box<dyn SeriesRenderer>`)
- [ ] `CoordinatePipeline` roundtrip test passes (world → screen → world within epsilon)
- [ ] All types are `Send + Sync` (required for multi-threaded rendering)
- [ ] Zero GPU dependencies in the library crate

---

## Phase 2: Pane & Layout Engine

> **Goal**: Independent panes with their own viewport, scale, layers, and series. A layout engine that supports horizontal, vertical, and grid nesting.

### Objectives

1. `Pane` struct with independent viewport, price scale, and layer stack
2. `LayoutEngine` supporting Horizontal, Vertical, and Grid layouts with nesting
3. `TimeScale` improvements (infinite scroll, bar spacing, right offset)
4. `PriceScale` improvements (auto, manual, log, percentage, inverted)

### PRs

| # | Task | Complexity | Files |
|---|---|---|---|
| 2.1 | Refactor `Pane` to own a `PaneState` with: `viewport: Viewport`, `price_scales: Vec<PriceScale>`, `layers: Vec<Box<dyn SeriesRenderer>>`, `drawings: DrawingSet`, `markers: MarkerSet`, `price_lines: PriceLineSet`. | L | `fast-chart/src/pane/mod.rs` |
| 2.2 | Implement `LayoutEngine` trait with variants: `VerticalStack`, `HorizontalSplit`, `GridLayout { rows, cols }`. Each layout computes pane rects from parent rect. | L | `fast-chart/src/layout/mod.rs` |
| 2.3 | Add `PaneDivider` — draggable separator between panes with hit testing, cursor changes, and resize events. | M | `fast-chart/src/pane/divider.rs` |
| 2.4 | Implement `TimeScale` improvements: `bar_spacing: f64`, `right_offset: f64`, `scroll_to_end()`, `visible_range() -> Range<usize>`, business days awareness (placeholder for Phase 7). | L | `fast-chart-domain/src/scale.rs` |
| 2.5 | Implement `PriceScale` improvements: `mode: PriceScaleMode { Auto, Manual, Locked, Logarithmic, Percentage, Indexed, Inverted }`, `margin_top: f64`, `margin_bottom: f64`, auto-fit with padding. | L | `fast-chart-domain/src/price_scale.rs` |
| 2.6 | Add `PaneEvent` enum: `DividerDragged { index, delta }`, `PaneResized { id, new_height }`, `PaneAdded { id }`, `PaneRemoved { id }`. Route through `InteractionEngine`. | M | `fast-chart/src/pane/events.rs` |
| 2.7 | Write tests: multi-pane layout, divider drag, viewport sync across panes, time scale infinite scroll, price scale auto-fit. | M | test modules in pane/ and layout/ |

### Dependencies

Phase 1 complete (coordinate pipeline and draw commands exist).

### Migration Notes

- The existing `Pane` in `fast-chart-core/src/app/pane.rs` (393 lines, well-tested) is the starting point. The refactor extends it with `layers`, `drawings`, and the `PaneState` composition.
- `LayoutManager` in `fast-chart-core/src/app/layout_manager.rs` (365 lines) is the starting point for `LayoutEngine`. The vertical stack logic is preserved; horizontal and grid are new.
- The current `Pane::pixel_y_offset()` and `Pane::pixel_height()` methods are replaced by the `LayoutEngine`'s rect computation.

### Success Criteria

- [ ] 3 panes with independent viewports render correctly
- [ ] Divider drag works with min-height enforcement
- [ ] `TimeScale::visible_range()` returns correct bar indices
- [ ] `PriceScale::auto_fit()` handles empty data, single bar, and 10M bars
- [ ] Nested layouts (vertical inside horizontal) produce correct rects
- [ ] All existing pane/layout tests still pass

---

## Phase 3: Series & Indicators

> **Goal**: Complete the series type coverage. Improve indicator architecture for custom indicators.

### Objectives

1. Missing series types: Step Line, Volume, Point & Figure, Line Break, Range
2. Improve `Indicator` trait for composable, multi-panel indicators
3. Add indicator overlay vs. separate pane support

### PRs

| # | Task | Complexity | Files |
|---|---|---|---|
| 3.1 | Add `StepLineSeries` — line chart where value holds until next data point (horizontal then vertical segments). | M | `fast-chart/src/series/step_line.rs` |
| 3.2 | Add `VolumeSeries` — histogram bars colored by price direction (close >= open = bullish). Supports overlay on main pane or separate pane. | M | `fast-chart/src/series/volume.rs` |
| 3.3 | Add `PointFigureSeries` — X/O chart with configurable reversal amount. No time axis (column-based). | L | `fast-chart/src/series/point_figure.rs` |
| 3.4 | Add `LineBreakSeries` — Renko-like but uses N-line break logic. | M | `fast-chart/src/series/line_break.rs` |
| 3.5 | Add `RangeSeries` — price range bars (high-low range fixed). | M | `fast-chart/src/series/range.rs` |
| 3.6 | Refactor `Indicator` trait: add `fn overlay_mode(&self) -> OverlayMode { OverlayOnPane(pane_id) | SeparatePane }` and `fn preferred_scale(&self) -> PriceScaleMode`. | M | `fast-chart-domain/src/indicator.rs` |
| 3.7 | Add `IndicatorRenderer` trait (extends `SeriesRenderer`): `fn render_overlay(&self, ctx, pane) -> Vec<DrawCommand>` and `fn render_separate(&self, ctx) -> Vec<DrawCommand>`. | M | `fast-chart/src/render/indicator_renderer.rs` |
| 3.8 | Implement `SeriesType::All` enum with all 15+ variants. Each variant maps to its `SeriesRenderer` implementation. | S | `fast-chart-domain/src/series_type.rs` |
| 3.9 | Add tests for each new series type: data generation, rendering commands, edge cases (empty data, single point, gaps). | L | test modules per series |

### Dependencies

Phase 1 complete (SeriesRenderer trait exists). Phase 2 helps but is not strictly required.

### Migration Notes

- The existing 16 indicators in `fast-chart-domain/src/indicators/` remain as pure computation (they already are). The new `IndicatorRenderer` wraps them with rendering logic.
- `HeikinAshi`, `Renko`, and `Kagi` in the indicators module are actually series transformations — they should be promoted to series types in `fast-chart/src/series/` with their computation staying in domain.
- The current `SeriesType` enum (Candle, Bar, Line, Area, Baseline) is extended, not replaced.

### Success Criteria

- [ ] All 15 series types from the spec have a domain type and renderer
- [ ] `Indicator::overlay_mode()` allows RSI to render in a separate pane while SMA overlays the main chart
- [ ] Custom series can be added by implementing `SeriesRenderer` — no changes to library code
- [ ] `PointFigureSeries` handles no-time-axis rendering correctly
- [ ] All series types have at least 5 unit tests each

---

## Phase 4: Drawing Tools

> **Goal**: Complete the drawing tool set. Make every drawing selectable, draggable, and deletable.

### Objectives

1. Missing drawing tools: Arrow, Ray, Segment, Box, Circle, Polygon, Text, Image, Label
2. Unified `Drawing` trait with selection, hit testing, and serialization
3. Drawing interaction: create, select, move, resize, delete

### PRs

| # | Task | Complexity | Files |
|---|---|---|---|
| 4.1 | Define `Drawing` trait: `fn id(&self) -> DrawingId`, `fn hit_test(&self, point) -> HitResult`, `fn move_to(&mut self, delta)`, `fn bounds(&self) -> Rect`, `fn to_commands(&self, ctx) -> Vec<DrawCommand>`. | M | `fast-chart-domain/src/drawing/trait.rs` |
| 4.2 | Add `Arrow` drawing — line with arrowhead at one or both ends. Configurable head size, fill, and color. | S | `fast-chart-domain/src/drawing/arrow.rs` |
| 4.3 | Add `Ray` drawing — starts at a point, extends infinitely in one direction. | S | `fast-chart-domain/src/drawing/ray.rs` |
| 4.4 | Add `Segment` drawing — line between two points with configurable endpoints (one or both extend to edge). | S | `fast-chart-domain/src/drawing/segment.rs` |
| 4.5 | Add `Box` drawing — 3D-style box with depth perspective (optional), fill, and border. | M | `fast-chart-domain/src/drawing/box_shape.rs` |
| 4.6 | Add `Circle` drawing — center + radius (pixel or price-based). | S | `fast-chart-domain/src/drawing/circle.rs` |
| 4.7 | Add `Polygon` drawing — arbitrary N-vertex shape, closed or open, with fill. | M | `fast-chart-domain/src/drawing/polygon.rs` |
| 4.8 | Add `TextDrawing` — anchored text with font size, color, background, alignment. | M | `fast-chart-domain/src/drawing/text_drawing.rs` |
| 4.9 | Add `ImageDrawing` — image at chart position with width/height. (Placeholder for Phase 7 image loading.) | S | `fast-chart-domain/src/drawing/image_drawing.rs` |
| 4.10 | Add `LabelDrawing` — floating label attached to a price level or bar, with configurable style. | M | `fast-chart-domain/src/drawing/label.rs` |
| 4.11 | Implement `DrawingManager` — manages all drawings in a pane: add, remove, select, deselect, hit-test cycle, bring-to-front, send-to-back. | L | `fast-chart/src/drawing/manager.rs` |
| 4.12 | Implement `DrawingInteraction` — state machine for create/select/move/resize/delete flows. Driven by `InteractionEngine` events. | L | `fast-chart/src/drawing/interaction.rs` |
| 4.13 | Refactor existing drawing types (TrendLine, Rectangle, etc.) to implement the `Drawing` trait. | M | `fast-chart-domain/src/drawing/*.rs` |
| 4.14 | Add serialization support (`serde`) for all drawing types (optional feature flag). | M | `fast-chart-domain/Cargo.toml`, `fast-chart-domain/src/drawing/*.rs` |
| 4.15 | Tests: hit testing for each drawing type, move/resize, drawing manager operations, serialization roundtrip. | L | test modules |

### Dependencies

Phase 1 complete (DrawCommand and CoordinatePipeline exist). Phase 2 helps (pane context for drawing placement).

### Migration Notes

- The existing 9 drawing types in `fast-chart-domain/src/drawing.rs` (983 lines) are refactored to implement the `Drawing` trait. The file is split into one file per type.
- The existing `DrawingSet` (typed vectors per drawing kind) is replaced by `DrawingManager` which uses `Box<dyn Drawing>` for polymorphic handling.
- The `DrawingId` type and `ChartPoint` type are preserved.

### Success Criteria

- [ ] All drawing types from the spec exist and implement `Drawing` trait
- [ ] Hit testing returns correct results for each drawing type (within 5px tolerance)
- [ ] `DrawingManager` correctly handles add/remove/select/z-order
- [ ] Drawing interaction state machine handles full create→select→move→delete lifecycle
- [ ] Serialization roundtrip passes for all drawing types
- [ ] 200+ tests for drawing tools

---

## Phase 5: Interaction System

> **Goal**: Complete input handling for all device types and interaction modes.

### Objectives

1. Unified input event system (mouse, touch, trackpad, keyboard, stylus)
2. Zoom system: wheel, pinch, box, axis, animated, programmatic
3. Pan system: drag, momentum, inertia, auto-scroll, follow-price
4. Crosshair modes: normal, magnetic, hidden, sync, global

### PRs

| # | Task | Complexity | Files |
|---|---|---|---|
| 5.1 | Define `InputEvent` enum: `MouseMove`, `MouseDown/Up`, `Wheel`, `Pinch`, `KeyDown/Up`, `TouchStart/Move/End`, `Stylus`. Platform-agnostic. | M | `fast-chart/src/input/mod.rs` |
| 5.2 | Define `InteractionEngine` — processes `InputEvent`s and produces `ChartCommand`s (zoom, pan, select, draw, etc.). State machine pattern. | L | `fast-chart/src/input/engine.rs` |
| 5.3 | Implement zoom modes: `WheelZoom` (current), `PinchZoom` (two-finger), `BoxZoom` (drag rectangle), `AxisZoom` (x or y only), `AnimatedZoom` (smooth interpolation), `ProgrammaticZoom` (API). | L | `fast-chart/src/input/zoom.rs` |
| 5.4 | Implement pan modes: `DragPan` (current), `MomentumPan` (fling), `InertiaPan` (decelerate), `AutoScroll` (follow latest bar), `FollowPrice` (lock crosshair to last price). | L | `fast-chart/src/input/pan.rs` |
| 5.5 | Refactor `KineticScroll` to support momentum + inertia with configurable friction, velocity decay, and snap-to-bar. | M | `fast-chart-domain/src/kinetic.rs` |
| 5.6 | Implement crosshair modes: `Normal`, `Magnetic` (snap to OHLC), `Hidden`, `Sync` (same position across panes), `Global` (shared across charts). | M | `fast-chart/src/input/crosshair.rs` |
| 5.7 | Implement `KeyboardShortcuts` — configurable key bindings for timeframe switch, drawing tool selection, undo/redo, zoom reset. | M | `fast-chart/src/input/keyboard.rs` |
| 5.8 | Add `GestureDetector` — unifies mouse, touch, and trackpad gestures into a common gesture vocabulary (tap, double-tap, long-press, drag, pinch, rotate). | L | `fast-chart/src/input/gesture.rs` |
| 5.9 | Tests: zoom center preservation, pan bounds, momentum deceleration, crosshair modes, keyboard shortcuts, gesture detection. | L | test modules |

### Dependencies

Phase 1 (DrawCommand, CoordinatePipeline) and Phase 2 (Pane, LayoutEngine) must be complete.

### Migration Notes

- The existing `InteractionCommand` and `InteractionHandler` in `fast-chart-core/src/ports/interaction.rs` (23 lines) are replaced by `InputEvent` + `InteractionEngine`.
- The existing `ViewportCommand` enum is absorbed into `ChartCommand`.
- `KineticScroll` in domain (already well-tested) is extended with momentum parameters.
- The winit-specific event handling in `fast-chart-app/src/adapters/input/handler.rs` becomes an adapter that converts `winit::Event` → `InputEvent`.

### Success Criteria

- [ ] Wheel zoom preserves center point (existing test still passes)
- [ ] Pinch zoom works on touch devices (at least the event processing path)
- [ ] Box zoom produces correct viewport from drag rectangle
- [ ] Momentum pan decelerates smoothly over 500ms
- [ ] Magnetic crosshair snaps to nearest OHLC values
- [ ] Auto-scroll follows the latest bar when new data arrives
- [ ] All keyboard shortcuts are configurable

---

## Phase 6: Render Pipeline

> **Goal**: Move rendering from hardcoded passes to a composable, efficient pipeline with dirty rendering and caching.

### Objectives

1. Render pass system (Background → Grid → Session → Indicator → Series → Drawing → Overlay → Labels → Crosshair → Tooltip → Debug)
2. Dirty rendering — only redraw changed regions
3. Multi-level caching (geometry, text, glyph, series, axis, grid, indicator, label)
4. Pixel-perfect rendering throughout
5. Create `fast-chart-renderer-wgpu` crate

### PRs

| # | Task | Complexity | Files |
|---|---|---|---|
| 6.1 | Create `fast-chart-renderer-wgpu/` crate. Move wgpu code from `fast-chart-examples` into it. Depends on `fast-chart` (library). | L | `fast-chart-renderer-wgpu/Cargo.toml`, all rendering files |
| 6.2 | Implement `WgpuBackend: RendererBackend` — execute `DrawCommand`s via wgpu. Batches commands by type for efficient GPU submission. | XL | `fast-chart-renderer-wgpu/src/backend.rs` |
| 6.3 | Define `RenderPass` enum: `Background`, `Watermark`, `Grid`, `Session`, `Indicator`, `Series`, `Drawing`, `Overlay`, `Labels`, `Crosshair`, `Tooltip`, `Debug`. Each pass has a priority and can be skipped. | M | `fast-chart/src/render/passes.rs` |
| 6.4 | Implement `RenderPipeline` — orchestrates pass execution. Collects `DrawCommand`s from all panes, sorts by layer, batches by pass, clips to pane rects. | L | `fast-chart/src/render/pipeline.rs` |
| 6.5 | Implement `DirtyRegion` tracking — `InvalidationMask` is extended with per-region granularity (viewport rect, not just level flags). `needs_redraw(region) -> bool`. | L | `fast-chart/src/render/dirty.rs` |
| 6.6 | Implement `CacheManager` with sub-caches: `GeometryCache` (vertex buffers), `TextCache` (formatted strings), `AxisCache` (tick positions), `GridCache` (grid lines), `IndicatorCache` (indicator geometry). Invalidation via content hash. | XL | `fast-chart/src/cache/mod.rs` |
| 6.7 | Pixel-perfect audit — ensure all line rendering uses `floor(x) + 0.5` alignment. Add `PixelPerfect` helper trait. | M | `fast-chart/src/render/pixel_perfect.rs` |
| 6.8 | Implement all existing sub-renderers (Candle, Line, Bar, Area, Histogram, Baseline, Crosshair, Grid, Markers, PriceLines, Text) as `SeriesRenderer` implementations in the wgpu crate. | XL | `fast-chart-renderer-wgpu/src/renderers/*.rs` |
| 6.9 | Add scissor rect management for multi-pane rendering (existing per-pane scissor logic in gpu_renderer.rs is the starting point). | M | `fast-chart-renderer-wgpu/src/backend.rs` |
| 6.10 | Performance benchmarks: 10K, 100K, 1M bars. Target: 60fps at 100K visible bars, <16ms frame time. | L | `fast-chart-renderer-wgpu/benches/` |

### Dependencies

Phases 1-5 complete. This is the integration phase.

### Migration Notes

- `GpuRenderer` (1122 lines) in `fast-chart-app/src/adapters/gpu_renderer.rs` is the starting point for `WgpuBackend`. It is decomposed: the orchestration logic goes to `RenderPipeline`, the wgpu-specific code goes to `WgpuBackend`, and the per-series vertex generation goes to individual `SeriesRenderer` implementations.
- The 12 sub-renderers (CandleRenderer, LineRenderer, etc.) are refactored to implement `SeriesRenderer` and produce `DrawCommand`s instead of directly writing GPU buffers.
- `ChartState` is split: domain state stays in `Pane`/`ChartController`, GPU state (vertex buffers, bind groups) lives in the wgpu crate's caches.

### Success Criteria

- [ ] `WgpuBackend` renders all series types via `DrawCommand` execution
- [ ] Dirty rendering skips unchanged regions (verified via render pass count reduction)
- [ ] Cache hit rate >90% for pan/zoom on static data
- [ ] Pixel-perfect: no anti-aliasing artifacts on 1px lines
- [ ] 60fps at 100K visible bars on integrated GPU
- [ ] All existing visual tests pass (manual comparison with current rendering)

---

## Phase 7: Polish, Performance & Documentation

> **Goal**: Production-ready library with complete documentation, animations, theming, and a stable public API.

### Objectives

1. Animation system (price, scale, zoom, scroll, opacity, selection, hover)
2. Theming system (dark, light, custom via design tokens)
3. Complete documentation (rustdoc + architecture docs + examples)
4. Public API stabilization (semver, feature flags)
5. Performance optimization pass

### PRs

| # | Task | Complexity | Files |
|---|---|---|---|
| 7.1 | Implement `AnimationEngine` — interpolates values over time with easing functions (linear, ease-in, ease-out, ease-in-out, spring). Animations: price tick, scale transition, zoom, scroll, opacity fade, selection highlight, hover effect. | L | `fast-chart/src/animation/mod.rs` |
| 7.2 | Implement `Theme` system — design token structure: `ChartTheme { background, grid, text, bullish, bearish, crosshair, ... }`. Built-in: `DarkTheme`, `LightTheme`. Custom themes via builder. | M | `fast-chart/src/theme/mod.rs` |
| 7.3 | Add `SessionLines` rendering — vertical lines marking trading sessions (market open/close). Configurable per exchange. | M | `fast-chart/src/render/session.rs` |
| 7.4 | Documentation: write `## Examples` doc-tests for every public API item. Target: `cargo doc --open` shows a usable chart in 5 minutes. | L | `fast-chart/src/**/*.rs` |
| 7.5 | Documentation: `docs/getting-started.md`, `docs/architecture/*.md` (8-10 files), `docs/migration-guide.md`. | L | `docs/` |
| 7.6 | Public API audit: review all `pub` items. Ensure consistency, naming conventions, and completeness. Mark unstable APIs with `#[unstable]` or feature flags. | M | all public surfaces |
| 7.7 | Add feature flags: `feature = "wgpu"` (renderer-wgpu), `feature = "serde"` (serialization), `feature = "animation"`, `feature = "sessions"`. | M | all `Cargo.toml` |
| 7.8 | Performance: profile with 10M bars. Optimize hot paths: vertex generation, cache invalidation, draw command sorting. Target: <1ms frame time for pan/zoom. | XL | benchmark-driven |
| 7.9 | Add `#[inline]` and `#[cold]` annotations to hot/cold paths identified by profiling. | M | hot path functions |
| 7.10 | Add `ChartBuilder` fluent API: `Chart::builder().pane(\|p\| p.series(Candle::new(data))).theme(DarkTheme).build()`. | L | `fast-chart/src/builder.rs` |
| 7.11 | Create `fast-chart-examples/` with 5+ examples: simple candle, multi-pane, custom series, drawing tools, real-time data. | L | `fast-chart-examples/src/` |
| 7.12 | Publish `fast-chart` and `fast-chart-domain` to crates.io (pre-release: `0.1.0-alpha`). | S | `Cargo.toml` version bump |

### Dependencies

Phase 6 complete (render pipeline works end-to-end).

### Migration Notes

- Animations are layered on top of the existing `InvalidationMask` system — animated values produce incremental viewport updates.
- The existing `Localization` types (EnglishLocalizer, SpanishLocalizer) in domain are preserved and extended with theme-aware formatting.
- Feature flags ensure the library can be used without wgpu, without serde, without animation — keeping dependencies minimal for embedded use cases.

### Success Criteria

- [ ] `Chart::builder().pane(|p| p.series(Candle::new(data))).build()` compiles and renders
- [ ] Dark and Light themes render correctly
- [ ] Price animation interpolates smoothly over 300ms
- [ ] All public items have rustdoc with examples
- [ ] `cargo doc` produces no warnings
- [ ] 10M bars: pan at <1ms, zoom at <5ms
- [ ] At least 3 examples in `fast-chart-examples/`
- [ ] Pre-release published to crates.io

---

## Migration Strategy

### Principle: Never Break Working Code

Every phase produces a **working library**. The migration is incremental:

1. **Phase 0** is a pure structural refactor. No logic changes. All 554 tests must pass at the end of every PR.
2. **Phase 1** adds new traits and types alongside existing code. Nothing is removed — old code is marked `#[deprecated]` and new code is added in parallel.
3. **Phases 2-5** progressively replace deprecated code. Each replacement is behind a feature flag or migration path.
4. **Phase 6** is the big integration — the wgpu renderer is rebuilt on the new traits. During this phase, the old `GpuRenderer` is kept as `GpuRendererLegacy` and the new one as `WgpuBackend`. Both must render identically (visual regression testing).
5. **Phase 7** removes legacy code and stabilizes the API.

### Branch Strategy

```
main
├── refactor/phase-0-library-identity
├── refactor/phase-1-core-abstractions
├── refactor/phase-2-pane-layout
├── refactor/phase-3-series-indicators
├── refactor/phase-4-drawing-tools
├── refactor/phase-5-interaction
├── refactor/phase-6-render-pipeline
└── refactor/phase-7-polish
```

Each phase branch merges to main after all tests pass and the phase is verified. Phases can be worked on in parallel where dependencies allow (e.g., Phase 3 and Phase 4 can overlap since they touch different parts of the codebase).

### Testing Strategy During Migration

- **Before each phase**: run `cargo test --workspace` and record baseline (554+ tests)
- **During each phase**: every PR must maintain or increase the test count
- **After each phase**: run full test suite + visual comparison test (if wgpu renderer changed)
- **Regression gate**: no PR merges if any existing test fails

---

## Summary: Phase Dependency Graph

```
Phase 0: Foundation ─────────────────────────────────────────────────┐
    │                                                               │
Phase 1: Core Abstractions ────────────────────────┐               │
    │                                               │               │
    ├─── Phase 2: Pane & Layout ────────────────────┤               │
    │       │                                       │               │
    │       ├─── Phase 3: Series & Indicators ──────┤               │
    │       │                                       │               │
    │       └─── Phase 4: Drawing Tools ────────────┤               │
    │                                               │               │
    └─── Phase 5: Interaction System ───────────────┤               │
                                                    │               │
                                        Phase 6: Render Pipeline ◄─┘
                                                    │
                                        Phase 7: Polish & Docs
```

**Parallelizable**: Phases 3, 4, and 5 can be worked on simultaneously after Phase 2 starts. Phase 6 requires all of 1-5 to be complete.

---

## Total Estimated Effort

| Phase | PRs | Complexity | Estimated Time |
|---|---|---|---|
| Phase 0: Foundation | 7 | M | 1-2 weeks |
| Phase 1: Core Abstractions | 8 | L | 2-3 weeks |
| Phase 2: Pane & Layout | 7 | L | 2-3 weeks |
| Phase 3: Series & Indicators | 9 | M-L | 2-3 weeks |
| Phase 4: Drawing Tools | 15 | M-L | 3-4 weeks |
| Phase 5: Interaction System | 9 | M-L | 2-3 weeks |
| Phase 6: Render Pipeline | 10 | XL | 4-6 weeks |
| Phase 7: Polish & Docs | 12 | M-L | 3-4 weeks |
| **Total** | **77** | | **19-28 weeks** |

---

## Critical Risks

1. **Phase 6 is the highest-risk phase** — rebuilding the wgpu renderer on new abstractions while maintaining visual parity. Mitigation: keep `GpuRendererLegacy` until visual regression tests pass.

2. **DrawCommand performance** — the intermediate representation adds an allocation layer. Mitigation: arena-allocate `DrawCommand`s, batch by type, pre-size vectors.

3. **API stability** — designing the public API before Phase 6 integration may require changes. Mitigation: mark everything `#[unstable]` until Phase 7.

4. **Test coverage during refactor** — migrating code risks losing test coverage. Mitigation: no code deletion without equivalent new tests.

---

## What NOT to Do

- **Do not add wgpu/winit dependencies to the library crate.** Ever. The library is renderer-agnostic.
- **Do not skip Phase 0.** Everything else builds on having a clean library crate.
- **Do not implement Phase 6 renderers before Phase 1 traits exist.** The traits are the contract.
- **Do not stabilize the API before Phase 7.** Premature stabilization blocks architectural improvements.
- **Do not optimize before Phase 6.** Profile first, optimize second.
