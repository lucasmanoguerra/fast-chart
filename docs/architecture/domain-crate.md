# Domain Crate (`fc-types`)

## Purpose

`fc-types` contains all pure domain types for financial charting.
Zero external dependencies (except optional `serde`). This crate defines the
vocabulary that the entire system speaks.

---

## Key Types

### Core Data

| Type | Description |
|------|-------------|
| `Bar` | OHLCV price bar with validation (`new()` returns `Result`) |
| `Tick` | Bid/ask quote |
| `TimeSeries<T, N>` | Fixed-capacity ring buffer — zero-allocation after init |

### Viewport & Scales

| Type | Description |
|------|-------------|
| `Viewport` | Visible time/value window with zoom/pan |
| `LinearScale` | Value → pixel coordinate mapping |
| `TimeScale` | Timestamp → pixel coordinate mapping |

### Crosshair & Magnet

| Type | Description |
|------|-------------|
| `Crosshair` | Cursor position with OHLC magnet snapping |
| `MagnetMode` | Nearest price, Open, High, Low, Close |

### Price Formatting

| Type | Description |
|------|-------------|
| `PriceScale` | Price formatting and auto-fit |
| `PriceScaleId` | Left, Right, or custom |
| `PriceScaleMode` | Normal, logarithmic, percentage, indexed |
| `PriceFormatter` | Trait for price → string formatting |
| `DefaultPriceFormatter` | Standard decimal formatting |

### Markers & Price Lines

| Type | Description |
|------|-------------|
| `Marker` | Point annotation (buy/sell signals) |
| `MarkerSet` | Collection of markers per pane |
| `PriceLine` | Horizontal price level |
| `PriceLineSet` | Collection of price lines per pane |

### Drawing Tools

| Type | Description |
|------|-------------|
| `TrendLine` | Two-point line |
| `Rectangle` | Axis-aligned rectangle |
| `HorizontalLine` / `VerticalLine` | Infinite lines |
| `FibonacciRetracement` | Fibonacci levels |
| `FibonacciExtension` | Fibonacci extensions |
| `Pitchfork` | Andrews' pitchfork |
| `Ellipse` | Ellipse shape |
| `Path` | Arbitrary polyline/polygon |

### Indicators

`Indicator` trait with 16 implementations:

SMA, EMA, VWAP, RSI, MACD, Bollinger Bands, ATR, ADX, CCI, Stochastic,
Williams %R, Parabolic SAR, Ichimoku Cloud, Supertrend, Heikin Ashi, Renko, Kagi.

### Invalidation

| Type | Description |
|------|-------------|
| `InvalidationLevel` | None, Layout, Data, Style, Full |
| `InvalidationMask` | Per-pane bitmask for selective redraw |
| `PaneBitmask` | Bitfield for pane indices |

### Other

| Type | Description |
|------|-------------|
| `KineticScroll` | Momentum/inertia scrolling |
| `Localizer` | Trait for UI string localization |
| `ChartError` | Typed error enum with context |

---

## Design Principles

- **Zero dependencies** — No runtime, no GPU, no I/O
- **Validation at construction** — `Bar::new()` returns `Result<Bar, ChartError>`
- **No mutation without reason** — Immutable by default
- **Ring buffer** — `TimeSeries<T, N>` uses const generics for capacity
- **Testable** — Every type has unit tests in the same module

---

## Feature Flags

| Feature | Description |
|---------|-------------|
| `serde` | Enable serde derives on all domain types |
