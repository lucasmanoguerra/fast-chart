# fc-renderer-wgpu

Backend de renderizado WGPU para la librería de gráficos de trading fast-chart. Implementa el trait `RendererBackend` de fc-render usando wgpu para aceleración por GPU. Soporta Vulkan, Metal, DX12 y WebGPU.

## Uso

```rust
use fc_renderer_wgpu::{WgpuBackend, WgpuRenderer};

// Crear el backend con device y queue de wgpu
let backend = WgpuBackend::new(device, queue);
let mut renderer = WgpuRenderer::new(backend);

// Configurar surface y pipeline
renderer.backend_mut().configure_surface(width, height);

// Renderizar un frame
renderer.backend_mut().begin_frame();
renderer.backend_mut().execute(&draw_commands);
renderer.backend_mut().end_frame();
```

## Dependencias

- `wgpu` — API gráfica cross-platform
- `bytemuck` — casting seguro de tipos
- `log` — logging
- `fc-primitives`, `fc-domain`, `fc-render`

## Estructura

| Módulo | Descripción |
|--------|-------------|
| `backend` | `WgpuBackend` — implementación wgpu de `RendererBackend` |
| `renderer` | `WgpuRenderer` — wrapper de alto nivel sobre el backend |
| `pipeline` | Configuración del pipeline de renderizado wgpu |
| `renderers` | Renderizadores especializados por tipo de serie |
| `cache` | Caché de geometría y textos para el GPU |
| `scissor` | Clipping rectangular por panel |
| `types` | Tipos GPU: `Vertex`, `Uniforms` |
| `vertex_gen` | Generación de vértices para primitivas |
