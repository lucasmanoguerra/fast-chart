# Tasks: fast-chart — Native GPU-Accelerated Trading Chart

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 3500–5000 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | 10 chained PRs (feature-branch-chain) |
| Delivery strategy | force-chained |
| Chain strategy | feature-branch-chain |

Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: feature-branch-chain
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | PR | Focused test | Runtime harness | Rollback boundary |
|------|------|-----|-------------|-----------------|-------------------|
| 1 | Domain crate + tests | PR#1 base=tracker | `cargo test -p fast-chart-domain` | N/A — pure lib | Revert `fast-chart-domain/` |
| 2 | Core crate (ports+controller) | PR#2 base=PR#1 | `cargo test -p fast-chart-core` | N/A — pure lib | Revert `fast-chart-core/` |
| 3 | App: winit+wgpu init | PR#3 base=PR#2 | `cargo check -p fast-chart-app` | Window appears | Revert `fast-chart-app/` |
| 4 | SimulatedDataProvider | PR#4 base=PR#3 | `cargo test -p fast-chart-app -- data` | Log output of bars | Revert `adapters/data/` |
| 5 | Grid + line series | PR#5 base=PR#4 | `cargo test -p fast-chart-app -- rendering` | Lines visible in window | Revert `rendering/` + `shaders/line.wgsl` |
| 6 | Candle + zoom/pan | PR#6 base=PR#5 | `cargo test -p fast-chart-app -- interaction` | Pan/zoom works | Revert `adapters/input/` + `shaders/candle.wgsl` |
| 7 | Indicator engine SIMD | PR#7 base=PR#6 | `cargo test -p fast-chart-core -- indicators` | N/A — pure lib | Revert indicator modules |
| 8 | Multi-pane + overlays | PR#8 base=PR#7 | `cargo test -p fast-chart-core -- layout` | Multi-pane visible | Revert layout modules |
| 9 | Text + crosshair | PR#9 base=PR#8 | `cargo check -p fast-chart-app` | Text labels + crosshair | Revert `text_renderer.rs` + crosshair |
| 10 | Config + polish | PR#10 base=PR#9 | `cargo test` (full suite) | Config hot-reload | Revert config + `shaders/fill.wgsl` |

---

## Phase 1: Domain Crate

- [x] 1.1 Create root `Cargo.toml` workspace + `fast-chart-domain/Cargo.toml` (zero deps)
- [x] 1.2 Create `fast-chart-domain/src/bar.rs` — `Bar` struct, OHLCV validation, `Result` on invalid
- [x] 1.3 Create `fast-chart-domain/src/tick.rs` — `Tick` struct (bid/ask/last/volume)
- [x] 1.4 Create `fast-chart-domain/src/series_type.rs` — `SeriesType` enum (Candle/Bar/Line/Area/Baseline)
- [x] 1.5 Create `fast-chart-domain/src/error.rs` — `ChartError` enum (InvalidPriceData/InsufficientData/OutOfRange)
- [x] 1.6 Create `fast-chart-domain/src/viewport.rs` — time range + value range + zoom level
- [x] 1.7 Create `fast-chart-domain/src/scale.rs` — `LinearScale` (price→y) + `TimeScale` (epoch→x)
- [x] 1.8 Create `fast-chart-domain/src/crosshair.rs` — position + snap-to-bar lookup
- [x] 1.9 Create `fast-chart-domain/src/indicator.rs` — `Indicator` trait (`calculate`, `name`)
- [x] 1.10 Create `fast-chart-domain/src/series.rs` — `TimeSeries<T, N>` ring buffer with `MaybeUninit`
- [x] 1.11 Create `fast-chart-domain/src/lib.rs` — module exports + unit tests for all types

## Phase 2: Core Crate

- [ ] 2.1 Create `fast-chart-core/Cargo.toml` (depends on domain only)
- [ ] 2.2 Create `fast-chart-core/src/ports/render.rs` — `ChartRenderer` trait
- [ ] 2.3 Create `fast-chart-core/src/ports/data_provider.rs` — `DataProvider` async trait + `DataEvent` enum
- [ ] 2.4 Create `fast-chart-core/src/ports/interaction.rs` — `InteractionHandler` trait + `InteractionCommand` enum
- [ ] 2.5 Create `fast-chart-core/src/app/chart_controller.rs` — `ChartController` orchestrator
- [ ] 2.6 Create `fast-chart-core/src/app/indicator_service.rs` — `IndicatorRegistry` + calc dispatch
- [ ] 2.7 Create `fast-chart-core/src/app/viewport_management.rs` — zoom/pan/bounds logic
- [ ] 2.8 Create `fast-chart-core/src/lib.rs` — module exports + unit tests with mocked ports

## Phase 3: App — Workspace + Window

- [ ] 3.1 Create `fast-chart-app/Cargo.toml` (wgpu/winit/glyphon/tokio/rkyv/serde deps)
- [ ] 3.2 Create `fast-chart-app/src/main.rs` — winit event loop skeleton with `ControlFlow::Poll`
- [ ] 3.3 Create `fast-chart-app/src/adapters/mod.rs` — adapter module structure
- [ ] 3.4 Init wgpu surface/device/queue from winit window handle

## Phase 4: Simulated Data

- [ ] 4.1 Create `fast-chart-app/src/adapters/data/simulated.rs` — drift-diffusion OHLC generator with seeded RNG
- [ ] 4.2 Create `fast-chart-app/src/adapters/data/rkyv_archive.rs` — zero-copy persistence for ring buffers
- [ ] 4.3 Wire tokio task → `mpsc::Sender` → render loop `try_recv()` in `about_to_wait()`

## Phase 5: Grid + Line Series

- [ ] 5.1 Write `fast-chart-app/src/adapters/rendering/shaders/line.wgsl` — line vertex/fragment shader
- [ ] 5.2 Create `fast-chart-app/src/adapters/rendering/pipelines.rs` — pipeline layout + shader compilation
- [ ] 5.3 Create `fast-chart-app/src/adapters/rendering/layers.rs` — draw order (grid→series→crosshair→HUD)
- [ ] 5.4 Create `fast-chart-app/src/adapters/rendering/wgpu_renderer.rs` — `ChartRenderer` impl
- [ ] 5.5 Add grid line rendering (horizontal + vertical reference lines)

## Phase 6: Candle + Zoom/Pan

- [ ] 6.1 Write `fast-chart-app/src/adapters/rendering/shaders/candle.wgsl` — candlestick shader
- [ ] 6.2 Add uniform buffer with `mat4x4<f32>` projection matrix
- [ ] 6.3 Implement zoom/pan uniform update via `queue.write_buffer()` (no vertex rebuild)
- [ ] 6.4 Create `fast-chart-app/src/adapters/input/winit_input.rs` — `InteractionHandler` impl
- [ ] 6.5 Wire winit events → viewport update → uniform → `window.request_redraw()`

## Phase 7: Indicator Engine

- [ ] 7.1 Implement SIMD SMA, EMA, RSI, MACD, Bollinger Bands, Stochastic, Ichimoku via `core::simd`
- [ ] 7.2 Implement scalar fallback for each indicator
- [ ] 7.3 Property tests: SIMD output ≈ scalar output within 1e-12 for all indicators

## Phase 8: Multi-Pane Layout

- [ ] 8.1 Create `Pane` struct (viewport + series list + indicator overlays)
- [ ] 8.2 Create `LayoutManager` — vertical stack, main pane ≥50% height
- [ ] 8.3 Shared x-axis sync across all panes on zoom/pan
- [ ] 8.4 Draggable pane dividers with minimum height enforcement (60px default)
- [ ] 8.5 Render indicator overlays in dedicated panes

## Phase 9: Text + Crosshair

- [ ] 9.1 Create `fast-chart-app/src/adapters/rendering/text_renderer.rs` — glyphon integration
- [ ] 9.2 Axis labels (y-axis: price ticks, x-axis: time labels)
- [ ] 9.3 Crosshair vertical/horizontal lines + OHLC tooltip near cursor
- [ ] 9.4 Pane title labels with symbol name

## Phase 10: Config + Polish

- [ ] 10.1 Create `fast-chart-app/src/config/chart_config.rs` — `ChartConfig` TOML serde with defaults
- [ ] 10.2 Hot-reload via `notify` crate → `ConfigReloadEvent`
- [ ] 10.3 Write `fast-chart-app/src/adapters/rendering/shaders/fill.wgsl` — area fill shader
- [ ] 10.4 Timeframe keyboard shortcuts (1/5/15/60/D/W)
- [ ] 10.5 Benchmark frame times (target: 60 FPS on 100k bars)
