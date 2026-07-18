# Migration Guide

This guide covers breaking changes across development phases. Use it when
upgrading from an earlier version of fast-chart.

## Phase 0: Library Identity

### Application → Library

The crate `fast-chart-app` (binary) was replaced by `fast-chart` (library).

| Before | After |
|--------|-------|
| `fast-chart-app` crate with `main()` | `fast-chart` crate, no binary |
| Library owns `winit::Window` | Host owns the window |
| Library owns `wgpu::Device` | Host owns the device |

**Action**: Remove `fast-chart-app` dependency. Add `fast-chart` as a library
dependency. Your application owns the window, event loop, and GPU surface.

### Crate Restructuring

| Before | After |
|--------|-------|
| Single `fast-chart-app` crate | `fc-types` + `fast-chart` |
| Domain types mixed with app logic | Domain types isolated in `fc-types` |

**Action**: Import domain types from `fast-chart` (re-exports `fc-types`).
Do not depend on `fc-types` directly unless building a custom renderer.

## Phase 1: Core Abstractions

### RendererBackend Trait

Rendering backends must implement the new `RendererBackend` trait:

```rust
pub trait RendererBackend: Send + Sync {
    fn execute(&mut self, commands: &[DrawCommand]);
    fn resize(&mut self, width: u32, height: u32);
    fn set_clip(&mut self, rect: Rect);
    fn clear_clip(&mut self);
    fn clear(&mut self, color: [f32; 4]);
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn begin_frame(&mut self);
    fn end_frame(&mut self);
}
```

**Action**: Replace direct wgpu calls with `RendererBackend` implementations.
The `WgpuBackend` in `fc-renderer-wgpu` is the reference implementation.

### DrawCommand Enum

Ad-hoc rendering calls are replaced by the `DrawCommand` enum:

```rust
pub enum DrawCommand {
    DrawLine { x0, y0, x1, y1, color, width, style, z_index },
    DrawRect { x, y, width, height, fill, stroke, stroke_width, z_index },
    DrawCircle { cx, cy, radius, fill, stroke, stroke_width, z_index },
    DrawTriangle { x0, y0, x1, y1, x2, y2, fill, stroke, stroke_width, z_index },
    DrawPath { points, color, width, style, closed, fill, z_index },
    DrawText { x, y, text, color, font_size, align_x, align_y, z_index },
    DrawImage { x, y, src, width, height, opacity, z_index },
}
```

**Action**: Replace imperative draw calls with `DrawCommand` construction.
Use convenience constructors: `DrawCommand::line()`, `filled_rect()`, `text()`, etc.

### SeriesRenderer Trait

Series renderers implement the `SeriesRenderer` trait:

```rust
pub trait SeriesRenderer: Send + Sync {
    fn update(&mut self, data: &[DrawCommand], bounds: Rect) -> Vec<DrawCommand>;
    fn hit_test(&self, x: f32, y: f32) -> Option<SeriesHit>;
    fn bounds(&self) -> Rect;
    fn layer_z_index(&self) -> i32;
}
```

**Action**: Migrate series rendering logic to produce `DrawCommand`s instead of
direct GPU calls.

## Phase 2: Pane System

### Independent Panes

Each pane now has its own viewport, price scales, series layers, drawings,
markers, and price lines.

| Before | After |
|--------|-------|
| Single global viewport | Per-pane viewport (shared x-axis, independent y-axis) |
| Flat chart state | `Pane` struct with isolated state |

**Action**: Access pane-specific state through the `Pane` struct. Time ranges are
shared across all panes; value ranges are pane-local.

### Pane Dividers

Panes are separated by draggable dividers (`PaneDivider`). Divider positions
are managed by `LayoutManager`.

**Action**: Use `LayoutManager` for multi-pane layouts instead of manual height
calculations.

## Phase 3: Series Types

### Series Implementations

Built-in series types in `fast-chart/src/series/`:

| Series | Description |
|--------|-------------|
| `VolumeSeries` | Volume bars |
| `StepLineSeries` | Step line chart |
| `LineBreakSeries` | Line break chart |
| `RangeSeries` | Range bar chart |
| `PointFigureSeries` | Point & figure chart |

Domain layer provides: Candlestick (Bar), OHLC, Line, Area, Baseline, Histogram,
Heikin Ashi, Renko, Kagi.

**Action**: Use the built-in series types or implement `SeriesRenderer` for custom
series.

## Phase 4: Drawing Tools

### Drawing System

Drawing tools are managed per-pane through `DrawingSet`:

```rust
use fast_chart::{
    TrendLine, Rectangle, FibonacciRetracement, FibonacciExtension,
    Pitchfork, Ellipse, HorizontalLine, VerticalLine, Path,
};
```

Drawings are selectable, movable, and deletable. Each has a unique `DrawingId`.

**Action**: Replace manual drawing state management with `DrawingSet` and the
`DrawingManager`.

## Phase 5: Input System

### Platform-Agnostic Events

Input events are now platform-agnostic via `InputEvent`:

```rust
pub enum InputEvent {
    MouseMove(MouseMoveEvent),
    MouseDown(MouseButtonEvent),
    MouseUp(MouseButtonEvent),
    Wheel(WheelEvent),
    TouchStart(TouchEvent),
    TouchMove(TouchEvent),
    TouchEnd(TouchEvent),
    Pinch(PinchEvent),
    KeyDown(KeyEvent),
    KeyUp(KeyEvent),
    StylusMove(StylusEvent),
    // ...
}
```

### InteractionEngine

The `InteractionEngine` consumes `InputEvent`s and produces `ChartCommand`s.
Sub-modules handle specific interactions:

- `zoom` — wheel zoom, pinch zoom, axis zoom
- `pan` — drag pan, momentum, inertia
- `crosshair` — magnetic snapping, sync
- `gesture` — multi-touch gestures
- `keyboard` — keyboard shortcuts

**Action**: Translate platform events (winit, web) to `InputEvent` and feed them
to the interaction engine.

## Phase 6: Render Pipeline

### Pass System

The render pipeline is divided into 12 ordered passes:

```
Background → Watermark → Grid → Session → Indicator → Series
→ Drawing → Overlay → Labels → Crosshair → Tooltip → Debug
```

Each pass has a 1000-unit z-index range. Passes can be individually enabled,
disabled, and marked dirty for selective re-rendering.

### Dirty Rendering

The `DirtyRegionTracker` tracks which screen regions need re-rendering per pass.
Only dirty regions are re-rendered, reducing GPU work.

**Action**: Mark regions dirty when state changes. The pipeline handles the rest.

### DrawLayer System

15 z-index layers ensure correct visual ordering:

```
Background(0) → Watermark(100) → Grid(200) → PriceScale(300)
→ TimeScale(400) → Indicators(500) → Candles(600) → Volume(700)
→ CustomSeries(800) → Drawings(900) → Crosshair(1000)
→ Selection(1100) → FloatingLabels(1200) → Tooltip(1300) → Cursor(1400)
```

## Phase 7: Polish

### Coordinate Pipeline

`CoordinatePipeline` handles world ↔ screen transforms with pixel-perfect
alignment (`floor(x) + 0.5`).

### Cache System

5 sub-caches for performance: `GeometryCache`, `TextCache`, `AxisCache`,
`GridCache`, `IndicatorCache`.

### Theme System

Design token-based theming with hot-swap support via `ThemeHandle`.

### Animation

Interpolation support for zoom, scroll, and price transitions.

### Localization

`Localizer` trait with `EnglishLocalizer` and `SpanishLocalizer` implementations.
