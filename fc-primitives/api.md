# API Reference — fc-primitives

## Tipos principales

### `Bar`
Vela OHLCV con timestamp (i64), open, high, low, close y volume. Constructor `Bar::new(timestamp, open, high, low, close, volume) -> Result<Self, ChartError>`.

### `Tick`
Punto de precio individual con timestamp y valor.

### `TimeSeries<T, N>`
Serie temporal de tamaño fijo con capacidad `N`. Almacena los últimos `N` elementos usando un buffer circular.

### `SeriesType`
Enum con variantes: `Candle`, `Bar`, `Line`, `Area`, `Baseline`, `Histogram`, `StepLine`, `Volume`, `HeikinAshi`.

### `LinearScale`
Escala lineal con rango fijo (min, max). Métodos: `value_to_pixel`, `pixel_to_value`, `fit`.

### `TimeScale`
Escala de tiempo para el eje X del chart.

### `Rgba`
Color con componentes f64 (r, g, b, a). Constructor: `Rgba::new(r, g, b, a)`, `Rgba::from_hex(0xRRGGBBAA)`, `Rgba::rgb(r, g, b)`.

### `Rect`
Rectángulo 2D con x, y, width, height.

### `LineStyle`
Estilo de línea: `Solid`, `Dashed`, `Dotted`.

### `KineticScroll`
Scroll cinético con velocidad, fricción y umbral de detección.

### `InvalidationLevel`
Nivel de invalidación: `None`, `Geometry`, `Full`.

### `InvalidationMask`
Máscara de bits para invalidar paneles específicos.

### `PaneBitmask`
Bitmask de paneles afectados.

## Traits

### `Localizer`
Trait para localización de textos. Implementaciones: `EnglishLocalizer`, `SpanishLocalizer`.

## Funciones clave

### `ChartError`
Enum de errores: `InvalidPrice`, `InvalidTimestamp`, `InvalidVolume`, `EmptySeries`.
