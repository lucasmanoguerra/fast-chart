# API Reference — fc-render

## Tipos principales

### `DrawCommand`
Enum de comandos de dibujo de bajo nivel: `DrawRect`, `DrawLine`, `DrawText`, `DrawCircle`, `DrawPath`, `DrawImage`.

### `RenderPipeline`
Pipeline completo de renderizado que orquesta passes, layers y batch.

### `DrawLayer`
Capa de renderizado con ZIndex para orden de dibujado.

### `CoordinatePipeline`
Pipeline de transformación de coordenadas mundo → pantalla. Constructor: `new(time_range, value_range, offset_x, offset_y, width, height, scale_width, dpr)`.

### `ScreenPoint`
Punto en coordenadas de pantalla (x: f32, y: f32).

### `WorldPoint`
Punto en coordenadas del mundo (x: f64, y: f64).

### `DirtyRegion`
Región del canvas que necesita re-dibujado.

### `DirtyRegionTracker`
Tracker que acumula regiones sucias para renderizado selectivo.

### `ScreenRect`
Rectángulo en pantalla (x, y, width, height en f32).

### `RenderContext`
Contexto de renderizado que contiene estado del frame actual.

### `PassTracker`
Tracker de passes completados en un frame.

### `FrameStats`
Estadísticas del frame: tiempo, draws, batches.

### `RenderableDrawing`
Dibujo con geometría pre-calculada lista para enviar al backend.

### `DrawBuffer`
Buffer stack-allocated para comandos de dibujo (SmallVec con 32 elementos).

## Traits

### `RendererBackend`
Trait para backends gráficos. Método principal: `execute(commands: &[DrawCommand])`.

### `SeriesRenderer`
Trait para renderizado de series. Método: `render()`.

## Enums auxiliares

### `RenderPass`
Pass de renderizado: `Background`, `Grid`, `Series`, `Indicator`, `Drawing`, `Overlay`, `Crosshair`, `Tooltip`, `Debug`.

### `DrawingMode`
Modo de dibujo: `None`, `Drawing`, `Selecting`, `Dragging`.

### `DrawingAction`
Acción de dibujo: `Start`, `Update`, `Finish`, `Cancel`.

## Re-exports

- `fc_drawing::{Drawing, DrawingBounds, HitResult}` — tipos de dibujo unificados
- `LineStyle` — estilo de línea (desde fc-primitives)
- `SeriesHit` — resultado de hit-test en series
