# fast-chart Architecture Overview

## Vision

fast-chart is a Rust library for GPU-accelerated financial charting. Applications
embed it; the library owns chart logic and produces `DrawCommand`s. The host
application owns the GPU surface and executes them.

Target: visual quality indistinguishable from TradingView Lightweight Charts, with
Rust's safety, performance, and extensibility guarantees.

---

## Crate Hierarchy

```
fc-types          Pure types, zero dependencies
                           Bar, Tick, Viewport, LinearScale, TimeScale,
                           Crosshair, PriceScale, Marker, Drawing tools,
                           Indicator trait + 16 implementations, Error types

fast-chart                 Library crate — chart logic, rendering abstractions
                           ChartController, Pane, LayoutManager,
                           Ports (ChartRenderer, DataProvider, InteractionHandler),
                           DrawCommand, SeriesRenderer, CoordinatePipeline,
                           CacheSystem, ThemeEngine, InputEngine

fc-renderer-wgpu   Optional GPU renderer (depends on wgpu)
                           WgpuBackend: implements RendererBackend trait
                           Sub-renderers for each series type

fc-examples        Example applications (winit + wgpu)
                           Simulated data provider, GPU renderer, input handler
```

**Dependency direction**: `fc-types` ← `fast-chart` ← `fc-renderer-wgpu`
← `fc-examples`

The core library crate (`fast-chart`) has zero GPU dependencies — only depends
on `fc-types`.

---

## Hexagonal Architecture (Ports & Adapters)

```
                    ┌─────────────────────────────┐
                    │      fc-types       │
                    │   (Pure types, zero deps)    │
                    └──────────────┬──────────────┘
                                   │
                    ┌──────────────▼──────────────┐
                    │        fast-chart            │
                    │   (Library: logic + ports)   │
                    │                              │
                    │  ┌─────────┐  ┌──────────┐  │
                    │  │   app/  │  │  ports/  │  │
                    │  │         │  │          │  │
                    │  │ ChartController       │  │
                    │  │ Pane   │  │ ChartRenderer│
                    │  │ LayoutManager         │  │
                    │  │ ViewportManager       │  │
                    │  └─────────┘  └──────────┘  │
                    └──────────────┬──────────────┘
                                   │
              ┌────────────────────┼────────────────────┐
              │                    │                     │
   ┌──────────▼────────┐ ┌────────▼────────┐ ┌─────────▼─────────┐
   │  fast-chart-      │ │   Data Provider │ │  Interaction      │
   │  renderer-wgpu    │ │   Adapter       │ │  Handler          │
   │  (WgpuBackend)    │ │   (Simulated)   │ │  (Winit)          │
   └───────────────────┘ └─────────────────┘ └───────────────────┘
```

### Ports (Interfaces)

| Port | Location | Purpose |
|------|----------|---------|
| `RendererBackend` | `fast-chart/src/render/backend.rs` | Execute DrawCommands on GPU/software |
| `DataProvider` | `fast-chart/src/ports/data_provider.rs` | Market data source (bars, ticks) |
| `InteractionHandler` | `fast-chart/src/ports/interaction.rs` | Input event processing |
| `SeriesRenderer` | `fast-chart/src/render/series_renderer.rs` | Series → DrawCommands |
| `ChartRenderer` | `fast-chart/src/ports/render.rs` | High-level rendering surface |

---

## Data Flow

```
Market Data → DataProvider → DataEvent (mpsc) → ChartController → ChartState
                                                                    │
User Input → InputEvent → InteractionEngine → ViewportCommand
                                                                    │
                                                            ChartState
                                                                 │
                                                     DirtyRegionTracker
                                                                 │
                                                     RenderPipeline
                                                                 │
                                                     Vec<DrawCommand>
                                                                 │
                                                     RendererBackend
                                                                 │
                                                             GPU Output
```

---

## Key Design Decisions

1. **Hexagonal Architecture** — Domain never depends on adapters
2. **Zero GPU deps in library** — Renderer-agnostic by design
3. **Gateway pattern** — `fast-chart` re-exports all domain types
4. **Port abstractions** — All I/O behind traits
5. **Ring buffer series** — Fixed-capacity, zero-allocation time series
6. **Validation at construction** — `Bar::new()` returns `Result`
7. **Pixel-perfect rendering** — `floor(x) + 0.5` for crisp 1px lines
8. **Dirty rendering** — Only re-render changed regions
9. **Design token theming** — Hot-swap colors at runtime
10. **Platform-agnostic input** — `InputEvent` enum, not winit types

---

## Testing Strategy

- **Domain**: 363+ unit tests + doc tests — pure logic, no I/O
- **Library**: 120+ unit tests + integration tests — mocked ports
- **Examples**: 71+ unit tests — adapter logic
- **Total**: 564+ tests, all passing

Each layer is testable in isolation through port abstractions.
