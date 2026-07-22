# fc-domain

Tipos de dominio para la librería de gráficos de trading fast-chart. Proporciona lógica de dominio a nivel de chart: crosshair con imanes, herramientas de dibujo, trait de indicadores con implementaciones, marcadores, líneas de precio, escalas de precio y gestión de viewport.

## Uso

```rust
use fc_domain::{
    Crosshair, MagnetMode, PriceScale, PriceScaleMode,
    Marker, MarkerShape, MarkerPosition, Indicator, Viewport,
};

let crosshair = Crosshair::new(MagnetMode::Nearest);
let scale = PriceScale::new("left".into(), PriceScaleMode::Normal);
let marker = Marker::new(1000, 105.0, MarkerShape::ArrowUp, MarkerPosition::AboveBar);
let viewport = Viewport::default();
```

## Dependencias

- `fc-primitives` — primitivos fundamentales (Bar, TimeSeries, escalas, color)

## Estructura

| Módulo | Descripción |
|--------|-------------|
| `crosshair` | Crosshair con modo imán (`Crosshair`, `MagnetMode`) |
| `drawing` | 15 tipos de dibujo: TrendLine, Arrow, Ray, Segment, Rectangle, Fibonacci, Pitchfork, Ellipse, Path, etc. |
| `indicator` | Trait `Indicator` e `OverlayMode` para indicadores |
| `indicators` | Implementaciones: EMA, MACD, VWAP, ATR, RSI, Kagi, Renko, Heikin Ashi |
| `marker` | Marcadores de precio (`Marker`, `MarkerShape`, `MarkerPosition`) |
| `price_line` | Líneas de precio horizontales (`PriceLine`) |
| `price_scale` | Escala de precio con modos: Normal, Percentage, Indexed, Logarithmic |
| `viewport` | Gestión de viewport (rango de tiempo y valores) |
