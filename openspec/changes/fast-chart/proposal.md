# Proposal: fast-chart — Native GPU-Accelerated Trading Chart

## Intent

Build a greenfield native desktop trading chart application in Rust. GPU-accelerated rendering via wgpu, hexagonal architecture for testability, and SIMD-optimized indicator calculations. MVP delivers simulated data on a multi-pane layout with a full indicator suite.

## Scope

### In Scope
- 3-crate workspace: `fast-chart-domain` (zero deps) → `fast-chart-core` (ports + app) → `fast-chart-app` (adapters + main)
- Domain types: Bar, Tick, Indicator trait, Viewport, Scale, Crosshair, SeriesType
- wgpu rendering: line, candle, bar, fill WGSL pipelines + glyphon text
- Zoom/pan via projection uniform update (vertex buffers unchanged)
- Ring buffers with const generic `MaybeUninit` capacities
- Multi-resolution cascade: Tick → 1m → 5m → 15m → 1H → 1D
- SIMD indicators: SMA, EMA, RSI, MACD, Bollinger Bands, Stochastic, Ichimoku
- Multi-pane layout: main chart + indicator panels below (TradingView-style)
- Simulated data generator for dev/testing
- Visual config system (colors, timeframes, symbols)
- Crosshair with OHLC tooltip

### Out of Scope
- Live data connections (WebSocket, FIX)
- Order entry UI
- Drawing tools / annotations

## Capabilities

### New Capabilities
- `domain-model`: Core domain types — Bar, Tick, TimeSeries, Indicator trait, Viewport, Scale, Crosshair, SeriesType, ChartError. Zero external deps, 100% testable.
- `gpu-renderer`: wgpu rendering pipeline with line/candle/bar/fill WGSL shaders, glyphon text integration, multi-layer compositing, uniform-driven viewport transform.
- `data-pipeline`: Ring buffer time series (const generic capacity), multi-resolution aggregation cascade, DataProvider port trait, tokio mpsc push model.
- `indicator-engine`: SIMD-accelerated indicator calculation framework via `core::simd`. Extensible via Indicator trait. SMA, EMA, RSI, MACD, Bollinger Bands, Stochastic, Ichimoku.
- `multi-pane-layout`: Multi-pane layout management — main chart area + resizable indicator panels below.
- `interaction`: Input handling — pan (click-drag), scroll zoom, crosshair mouse tracking, timeframe keyboard shortcuts.
- `simulated-data`: Synthetic OHLC market data generator with configurable volatility and volume for dev/testing.
- `chart-config`: Configuration system for visual settings (colors, fonts), timeframes, symbols, layout preferences.

### Modified Capabilities
None — greenfield project.

## Approach

3-crate hexagonal Rust workspace. Domain crate with zero external deps. Core crate defines port traits (`ChartRenderer`, `DataProvider`, `InteractionHandler`) + `ChartController` orchestration. App crate implements adapters: `WgpuRenderer`, `WinitInputAdapter`, `SimulatedDataProvider`.

Single wgpu render pass, draw-order layering: grid → price series → indicators → crosshair → HUD. Zoom/pan updates projection matrix uniform only — no vertex buffer rebuild. `core::simd` for rolling indicator math. tokio async data pipeline → mpsc channel → sync render loop. rkyv zero-copy archives for hot time series.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `fast-chart-domain/src/` | New | Domain types (zero external deps) |
| `fast-chart-core/src/` | New | Port traits + ChartController |
| `fast-chart-app/src/` | New | wgpu/winit/glyphon adapters, shaders, main |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| wgpu API churn | Medium | Pin exact version, isolate behind ChartRenderer trait |
| glyphon breaking changes | Medium | Thin wrapper — fallback to raw swash |
| Workspace overhead | Low | Easy to merge into single crate |
| SIMD portability (ARM) | Low | core::simd handles NEON; scalar fallback |
| rkyv schema evolution | Medium | Version-tag archives, forward-compat attributes |

## Rollback Plan

Greenfield project — no production to roll back. Architecture direction can be corrected within crate boundaries: replaceable layers (renderer, data, input) without restarting.

## Dependencies

- Rust 1.96+, wgpu 29.x, winit 0.30+, glyphon 0.5+
- tokio 1.x, rkyv 0.8, serde 1.x, time 0.3

## Success Criteria

- [ ] `cargo check -p fast-chart-domain` compiles with zero external deps
- [ ] Domain types have full test coverage
- [ ] Simulated data renders as line + candle series in winit window
- [ ] Zoom/pan updates projection uniform without vertex buffer change
- [ ] At least 3 indicators calculate correctly with SIMD
- [ ] Multi-resolution aggregation produces correct OHLC cascades
- [ ] Crosshair tracks mouse with OHLC tooltip display
- [ ] Multi-pane layout renders chart + indicator panels
