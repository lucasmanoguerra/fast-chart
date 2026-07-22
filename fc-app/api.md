# API Reference — fc-app

## Tipos principales

### `ChartController`
Controller central que coordina data, input, viewport y render. Métodos: `new()`, `state()`, `state_mut()`, `handle_input()`, `tick()`, `stop_data_provider()`, `stop_kinetic()`, `update_kinetic()`.

### `ChartState`
Estado central del chart: viewport, series, crosshair, drawings, markers. Única fuente de verdad para el renderer.

### `FrameCounter`
Contador de FPS para monitoreo de rendimiento.

### `LayoutManager`
Gestor de layout multi-panel con divisores arrastrables. Métodos: `new()`, `sync_time_range()`, `hit_test_divider()`, `start_drag()`, `end_drag()`, `update_drag()`.

### `ChartBuilder`
Constructor declarativo del chart. Métodos: `new()`, `with_data_provider()`, `with_interaction_handler()`, `build()`.

## Tipos de layout

### `VerticalStack`
Layout vertical de paneles apilados.

### `HorizontalSplit`
Layout horizontal dividido.

### `GridLayout`
Layout en grilla.

### `Pane`
Un panel individual del chart con su propio viewport y serie.

### `PaneDivider`
Divisor arrastrable entre paneles.

### `ViewportManager`
Gestión del viewport: pan, zoom, auto-fit.

## Puertos (Ports)

### `DataProvider`
Trait para proveedores de datos. Métodos: `start()`, `name()`, `subscribe()`, `latest()`.

### `InteractionHandler`
Trait para manejo de interacción del usuario.

### `ChartRenderer`
Trait para renderizadores. Métodos: `render()`, `resize()`, `request_redraw()`.

### `DataEvent`, `DataError`
Eventos y errores del data provider.

### `InteractionCommand`, `ViewportCommand`
Comandos de interacción: `PanBy`, `ZoomAtCursor`, `UpdateCrosshair`, `DeactivateCrosshair`.

### `FrameState`
Estado del frame actual para el renderer.
