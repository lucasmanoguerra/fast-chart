# Library Crate (`fast-chart`)

## Purpose

`fast-chart` is the application layer that bridges domain types with rendering
and data-provider adapters through well-defined port traits. It is the single
gateway to all chart logic — applications depend only on this crate.

---

## Modules

| Module | Purpose |
|--------|---------|
| `app/` | ChartController, Pane, LayoutManager, ViewportManager |
| `ports/` | Port traits (RendererBackend, DataProvider, InteractionHandler) |
| `render/` | DrawCommand, RendererBackend, SeriesRenderer, Pipeline, Layers |
| `cache/` | GeometryCache, TextCache, AxisCache, GridCache, IndicatorCache |
| `input/` | InputEvent, InteractionEngine, Zoom, Pan, Crosshair, Keyboard |
| `series/` | Built-in series (Volume, StepLine, LineBreak, Range, PointFigure) |
| `theme/` | ChartTheme, ThemeToken, ThemeHandle, Rgba, ChartThemeBuilder |
| `animation/` | Interpolation for zoom, scroll, price transitions |

---

## RendererBackend Trait

The core rendering contract. Any backend (wgpu, glow, Skia, software) implements
this trait to execute draw commands:

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

Object-safe: usable as `Box<dyn RendererBackend>`.

---

## DrawCommand

Universal render queue primitives. All positions use screen-space pixels with
origin at top-left `(0, 0)`. Colors are `[r, g, b, a]` in linear float.

7 variants: `DrawLine`, `DrawRect`, `DrawCircle`, `DrawTriangle`, `DrawPath`,
`DrawText`, `DrawImage`.

Convenience constructors: `DrawCommand::line()`, `dashed_line()`, `filled_rect()`,
`stroked_rect()`, `filled_circle()`, `filled_triangle()`, `polyline()`,
`filled_polygon()`, `text()`.

Each carries a `z_index` for render ordering.

---

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `serde` | off | Propagates to `fast-chart-domain/serde` |

---

## Gateway Pattern

`fast-chart` re-exports all domain types so consumers never need to import
`fast-chart-domain` directly:

```rust
// This is all you need:
use fast_chart::{Bar, Tick, Viewport, TimeSeries, ChartController, DrawCommand};
```
