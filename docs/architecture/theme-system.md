# Theme System

## Design Tokens

All chart colors are identified by `ThemeToken` — a type-safe enum of 33
semantic color slots:

| Category | Tokens |
|----------|--------|
| Background | `Background`, `PaneBackground` |
| Grid | `GridLine` |
| Text | `TextPrimary`, `TextSecondary` |
| Price Scale | `PriceScaleBackground`, `PriceScaleText`, `PriceScaleBorder` |
| Time Scale | `TimeScaleBackground`, `TimeScaleText`, `TimeScaleBorder` |
| Series | `Bullish`, `BullishFill`, `Bearish`, `BearishFill`, `LineColor`, `AreaFill`, `VolumeBullish`, `VolumeBearish` |
| Crosshair | `CrosshairLine`, `CrosshairLabelBg`, `CrosshairLabelText` |
| Selection | `SelectionBorder`, `SelectionFill` |
| Hover | `HoverBorder` |
| Markers | `MarkerUp`, `MarkerDown`, `MarkerNeutral` |
| Drawings | `DrawingLine`, `DrawingFill`, `DrawingText` |
| Other | `Divider`, `Watermark` |

---

## Built-in Themes

```rust
let dark = ChartTheme::dark();   // Trading terminal style
let light = ChartTheme::light(); // Clean light style
let any = ChartTheme::preset("dark"); // By name
```

---

## ChartThemeBuilder

Custom themes via builder pattern:

```rust
use fast_chart::theme::{ChartThemeBuilder, ThemeToken, Rgba};

let theme = ChartThemeBuilder::new()              // starts from dark
    .with("background", Rgba::rgb(0.05, 0.05, 0.1))
    .with_token(ThemeToken::Bullish, Rgba::rgb(0.0, 0.9, 0.5))
    .with_token(ThemeToken::Bearish, Rgba::rgb(0.9, 0.2, 0.2))
    .build();

// Or start from an existing theme
let custom = ChartThemeBuilder::from_theme(ChartTheme::light())
    .with_token(ThemeToken::GridLine, Rgba::rgb(0.8, 0.8, 0.8))
    .build();
```

---

## Hot-Swap

Colors can be changed at runtime without rebuilding the theme:

```rust
let mut theme = ChartTheme::dark();

// Single token
theme.set_color(ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0));

// Batch update
theme.set_colors(&[
    (ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0)),
    (ThemeToken::Bearish, Rgba::rgb(1.0, 0.0, 0.0)),
]);
```

---

## ThemeHandle (Thread-Safe)

For sharing theme state between UI and renderer threads:

```rust
use fast_chart::theme::{ThemeHandle, ChartTheme, ThemeToken, Rgba};

let handle = ThemeHandle::new(ChartTheme::dark());

// Clone shares state
let renderer_handle = handle.clone();

// UI thread: hot-swap a color
handle.set_color(ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0));

// Renderer thread: read current theme
let theme = handle.read();
let color = theme.get_color(ThemeToken::Bullish);

// Or swap entire theme
handle.set(ChartTheme::light());
```

---

## Rgba Color Type

Colors use `Rgba(f64, f64, f64, f64)` with channels in `[0.0, 1.0]`:

```rust
let opaque = Rgba::rgb(1.0, 0.0, 0.0);         // red, fully opaque
let transparent = Rgba::new(0.0, 1.0, 0.0, 0.5); // green, 50% alpha
let from_hex = Rgba::from_hex(0xFF0000FF);       // red from 0xRRGGBBAA
```
