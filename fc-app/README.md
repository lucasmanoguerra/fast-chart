# fc-app

Capa de aplicación para la librería de gráficos de trading fast-chart. Contiene el `ChartController`, puertos de arquitectura hexagonal y gestión de layout multi-panel. Es el punto de entrada principal para construir una aplicación de charting.

## Uso

```rust
use fc_app::{ChartController, ChartBuilder, ChartState};

let controller = ChartController::new(
    Box::new(data_provider),
    Box::new(interaction_handler),
);

// Acceder al estado del chart
let state: &ChartState = controller.state();
let viewport = &state.viewport;

// Construcción declarativa con el builder
let chart = ChartBuilder::new()
    .with_data_provider(provider)
    .with_interaction_handler(handler)
    .build();
```

## Dependencias

- `fc-primitives`, `fc-domain`, `fc-render`, `fc-theme`, `fc-cache`, `fc-animation`, `fc-sessions`, `fc-input`
- `log`, `smallvec`, `num-traits`
- `rayon` (opcional, paralelismo)
- `notify` (opcional, hot-reload de config)
- `toml` (opcional, configuración)

## Estructura

| Módulo | Descripción |
|--------|-------------|
| `app` | `ChartController`, `ChartState`, `FrameCounter`, layout, pane, viewport management |
| `builder` | Constructor declarativo `ChartBuilder` |
| `ports` | Puertos: `DataProvider`, `InteractionHandler`, `ChartRenderer` |
| `cache` | Caché de la capa de aplicación |
| `input` | Adaptación de input al dominio |
| `render` | Coordinación de renderizado |
| `series` | Gestión de series en el controller |
| `theme` | Integración de temas |
| `animation` | Integración de animaciones (feature flag) |
| `config_watcher` | Hot-reload de configuración TOML (feature flag) |
