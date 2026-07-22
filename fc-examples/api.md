# API Reference — fc-examples

## Tipo principal

### `App`
Estructura principal de la aplicación de ejemplo. Implementa `ApplicationHandler` de winit. Gestiona el ciclo de vida de la ventana, el GPU renderer, el chart controller, el layout multi-panel y la interacción del usuario.

## Adaptadores

### `GpuRenderer`
Wrapper que integra `fc-renderer-wgpu` con la ventana winit. Métodos: `new(window)`, `render()`, `resize()`, `request_redraw()`, `set_crosshair()`, `deactivate_crosshair()`, `set_cursor()`, `canvas_width()`, `canvas_height()`, `info()`.

### `SimulatedDataProvider`
Proveedor de datos simulados para demostración. Genera velas OHLCV con random walk. Constructor: `new(symbol, initial_price, volatility)`.

### `WinitInteractionHandler`
Adaptador de interacción winit → fc-input.

### `ChartConfig`
Configuración del chart cargada desde TOML. Campos: `window.title`, `window.width`, `window.height`.

### `ConfigWatcher`
Watcher de archivos para hot-reload de configuración. Método: `check_reload() -> Option<ChartConfig>`.

## Ejemplos

### `simple_candle`
Chart básico de velas con data simulada y crosshair.

### `multi_pane`
Layout vertical con 3 paneles: candles, volume y RSI, con divisores arrastrables.

### `drawing_tools`
Demostración de herramientas de dibujo: trend lines, fibonacci, rectangles.

### `animation_demo`
Transiciones animadas con diferentes funciones de easing.

### `builder_api`
Construcción declarativa del chart usando `ChartBuilder`.

### `custom_theme`
Tema personalizado con colores custom y hot-swap en runtime.
