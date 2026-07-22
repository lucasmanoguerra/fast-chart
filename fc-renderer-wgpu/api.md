# API Reference — fc-renderer-wgpu

## Tipos principales

### `WgpuBackend`
Backend de renderizado que implementa `RendererBackend` usando wgpu. Maneja device, queue, surface, pipeline, buffers de vértices e índices. Métodos: `new()`, `configure_surface()`, `begin_frame()`, `end_frame()`, `execute()`, `resize()`, `set_crosshair()`, `deactivate_crosshair()`, `set_cursor()`, `request_redraw()`, `canvas_width()`, `canvas_height()`, `info()`.

### `WgpuRenderer`
Wrapper de alto nivel sobre `WgpuBackend`. Métodos: `new()`, `backend()`, `backend_mut()`.

## Módulos

### `pipeline`
Configuración del pipeline de renderizado wgpu: shaders, bind groups, formatos.

### `renderers`
Renderizadores especializados para cada tipo de serie (candle, line, area, histogram, bar, baseline, volume).

### `cache`
Caché de geometría GPU para evitar re-generación de vértices.

### `scissor`
Clipping rectangular — cada panel se renderiza dentro de su región.

### `types`
- `Vertex` — vértice con posición, color y coordenadas de textura.
- `Uniforms` — uniforms del shader (resolución, tiempo, etc.).

### `vertex_gen`
Generación de vértices e índices para primitivas gráficas (líneas, rectángulos, triángulos, círculos aproximados).
