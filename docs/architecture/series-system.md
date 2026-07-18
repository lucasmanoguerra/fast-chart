# Series System

## SeriesRenderer Trait

The core abstraction for rendering series data into draw commands. Each series
type implements this trait, and the engine treats all series uniformly.

```rust
pub trait SeriesRenderer: Send + Sync {
    fn update(&mut self, data: &[DrawCommand], bounds: Rect) -> Vec<DrawCommand>;
    fn hit_test(&self, x: f32, y: f32) -> Option<SeriesHit>;
    fn bounds(&self) -> Rect;
    fn layer_z_index(&self) -> i32 { 600 }
}
```

### Methods

- `update()` — Produce draw commands for the visible portion of the series
- `hit_test()` — Test if a screen point hits a data element
- `bounds()` — Bounding rectangle in screen space
- `layer_z_index()` — Which z-index layer this series renders into

Object-safe: usable as `Box<dyn SeriesRenderer>`.

---

## Built-in Series Types

### Application Layer (`fast-chart/src/series/`)

| Series | Description |
|--------|-------------|
| `VolumeSeries` | Volume bars (colored by direction) |
| `StepLineSeries` | Step line chart |
| `LineBreakSeries` | Line break chart |
| `RangeSeries` | Range bar chart |
| `PointFigureSeries` | Point & figure chart |

### Domain Layer (via `Bar`)

| Type | SeriesType Variant |
|------|-------------------|
| Candlestick | `Candle` (default) |
| OHLC Bar | `Ohlc` |
| Line | `Line` |
| Area | `Area` |
| Baseline | `Baseline` |
| Histogram | `Histogram` |
| Heikin Ashi | `HeikinAshi` |
| Renko | `Renko` |
| Kagi | `Kagi` |

---

## SeriesRef

Each series in a pane is referenced by name and type:

```rust
pub struct SeriesRef {
    pub name: String,
    pub series_type: SeriesType,
    pub price_scale_id: PriceScaleId,
}
```

---

## Custom Series

Implement `SeriesRenderer` to create any custom series type. The engine
never distinguishes built-in from custom series:

```rust
struct MyHeatmapSeries { /* ... */ }

impl SeriesRenderer for MyHeatmapSeries {
    fn update(&mut self, _data: &[DrawCommand], bounds: Rect) -> Vec<DrawCommand> {
        // Transform heatmap data into DrawCommands
        vec![/* ... */]
    }

    fn hit_test(&self, x: f32, y: f32) -> Option<SeriesHit> { None }
    fn bounds(&self) -> Rect { Rect::new(0.0, 0.0, 800.0, 600.0) }
    fn layer_z_index(&self) -> i32 { 850 } // CustomSeries layer
}
```

---

## SeriesHit

Hit-test result for mouse interaction:

```rust
pub struct SeriesHit {
    pub index: usize,    // nearest data point index
    pub distance: f32,   // distance from hit point to data point
}
```

Used by the crosshair and tooltip systems to display data values on hover.

---

## Rect

Axis-aligned bounding rectangle in screen space:

```rust
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

Methods: `contains()`, `right()`, `bottom()`, `center()`, `new()`.
