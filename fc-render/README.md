# fc-render

Motor de renderizado para gráficos financieros fast-chart. Define los primitivos de renderizado universales, sistema de capas, pipeline de coordenadas, passes de renderizado y traits centrales. Es agnóstico al backend y produce valores `DrawCommand` que pueden ejecutarse en cualquier adaptador gráfico.

## Uso

```rust
use fc_render::{
    DrawCommand, RenderPipeline, DrawLayer, CoordinatePipeline,
    ScreenPoint, WorldPoint, DirtyRegionTracker, RenderContext,
};

let pipeline = CoordinatePipeline::new(
    (0.0, 1000.0),   // rango de tiempo
    (90.0, 110.0),    // rango de precio
    0.0, 0.0,         // offset
    800.0, 600.0,     // tamaño del canvas
    60.0,             // price scale width
    1.0,              // device pixel ratio
);

let screen: ScreenPoint = pipeline.world_to_screen(WorldPoint { x: 500.0, y: 105.0 });
let commands: Vec<DrawCommand> = vec![
    DrawCommand::DrawLine { x0: 0.0, y0: 0.0, x1: 100.0, y1: 100.0, color: [1.0; 4], width: 1.0 },
];
```

## Dependencias

- `smallvec`, `num-traits`
- `fc-primitives`, `fc-domain`, `fc-drawing`
- `approx` (dev-dependencies)

## Estructura

| Módulo | Descripción |
|--------|-------------|
| `backend` | Trait `RendererBackend` para backends gráficos |
| `commands` | `DrawCommand` — comandos de dibujo de bajo nivel |
| `context` | `RenderContext` — contexto de renderizado por frame |
| `coordinates` | `CoordinatePipeline`, `ScreenPoint`, `WorldPoint` |
| `dirty` | `DirtyRegion`, `DirtyRegionTracker` — renderizado selectivo |
| `layers` | `DrawLayer` — sistema de capas por ZIndex |
| `passes` | `RenderPass`, `PassTracker` — passes de renderizado |
| `pipeline` | `RenderPipeline` — pipeline completo de renderizado |
| `pixel_perfect` | Alineación de píxel para líneas nítidas |
| `series_renderer` | `SeriesRenderer` — trait para renderizado de series |
| `indicator_renderer` | `IndicatorRenderer` — renderizado de indicadores |
| `drawing_manager` | `DrawingManager` — gestión de objetos de dibujo |
| `drawing_interaction` | `DrawingInteraction`, `DrawingMode`, `DrawingAction` |
| `renderable_drawing` | `RenderableDrawing` — dibujo listo para renderizar |
