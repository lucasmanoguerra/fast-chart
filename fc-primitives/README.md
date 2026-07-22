# fc-primitives

Primitivos de dominio puros para la librería de gráficos de trading fast-chart. Sin dependencias externas (serde opcional). Contiene los bloques fundamentales: barras OHLCV, ticks, series temporales, escalas, colores, viewport, invalidación, localización y scroll cinético.

## Uso

```rust
use fc_primitives::{Bar, TimeSeries, LinearScale, Rgba, Rect};

let bar = Bar::new(1000, 100.0, 105.0, 99.0, 102.0, 5000).unwrap();
let mut series: TimeSeries<Bar, 1000> = TimeSeries::new();
series.push(bar);
let latest = series.latest().unwrap();
let color = Rgba::new(0.0, 1.0, 0.0, 1.0);
let rect = Rect::new(0.0, 0.0, 800.0, 600.0);
```

## Dependencias

- `thiserror` — manejo de errores tipados
- `serde` (opcional) — serialización/deserialización

## Estructura

| Módulo | Descripción |
|--------|-------------|
| `bar` | Vela OHLCV (`Bar`) |
| `tick` | Punto de precio (`Tick`) |
| `series` | Serie temporal de tamaño fijo (`TimeSeries<T, N>`) |
| `series_type` | Tipo de serie: `Candle`, `Line`, `Area`, `Histogram`, etc. |
| `scale` | Escalas lineales y de tiempo (`LinearScale`, `TimeScale`) |
| `color` | Color RGBA (`Rgba`) |
| `rect` | Rectángulo 2D (`Rect`) |
| `line_style` | Estilos de línea: `Solid`, `Dashed`, `Dotted` |
| `kinetic` | Scroll cinético con inercia (`KineticScroll`) |
| `invalidation` | Niveles de invalidación para redraw selectivo |
| `localization` | Localización (inglés/español) |
| `error` | Error del charting (`ChartError`) |
