# Exploration: Lightweight-Charts Inspired Improvements

## Current State

The fast-chart project has a solid hexagonal architecture with three crates:

- **fast-chart-domain**: Pure domain types — `Bar`, `Crosshair`, `Viewport`, `LinearScale`, `TimeScale`, `SeriesType` enum (Candle/Bar/Line/Area/Baseline), `TimeSeries<T,N>` ring buffer, `Indicator` trait.
- **fast-chart-core**: Application orchestration — `ChartController` (tick loop + input routing), `Pane` (viewport + series refs), `LayoutManager` (vertical pane stack), `ViewportManager` (zoom/pan/auto-fit), `IndicatorRegistry`. Ports: `ChartRenderer`, `DataProvider`, `InteractionHandler`.
- **fast-chart-app**: Adapters — `GpuRenderer` (wgpu), sub-renderers (CandleRenderer, LineRenderer, GridRenderer, CrosshairRenderer, TextRenderer), `SimulatedDataProvider`, `WinitInteractionHandler`.

### Test count: 187 tests across all crates.

---

## Affected Areas by Improvement

### 1. Invalidation Model (P0)

**Current**: `ChartState.needs_redraw: bool` in `chart_controller.rs` (line 18). `GpuRenderer` has 4 additional booleans: `needs_line_update`, `needs_candle_update`, `needs_divider_update`, `needs_text_update` (gpu_renderer.rs lines 45-48).

**Gap**: Single boolean means EVERY input event triggers a full re-render. No way to distinguish cursor-only moves (cheap) from data changes (expensive). The GpuRenderer's 4 booleans are private and never checked — they're dead code.

**Files affected**:
- `fast-chart-core/src/app/chart_controller.rs` — `ChartState.needs_redraw` → `InvalidationMask`
- `fast-chart-app/src/adapters/gpu_renderer.rs` — 4 boolean fields → single mask
- `fast-chart-core/src/ports/interaction.rs` — `ViewportCommand::RequestRedraw` → `RequestInvalidation(InvalidationMask)`

### 2. Price Scales (P0)

**Current**: `Viewport` has a single `value_min`/`value_max` pair (viewport.rs lines 4-5). `LinearScale` is a simple {min, max, height} struct (scale.rs lines 2-6). `ViewportManager.create_linear_scale()` creates one scale per pane (viewport_management.rs line 88).

**Gap**: No concept of Left/Right/Overlay price scales. Each pane gets exactly one auto-scaled y-axis. No formatter abstraction. No way to attach a series to a specific price scale.

**Files affected**:
- `fast-chart-domain/src/scale.rs` — new `PriceScaleId`, `PriceScaleConfig`
- `fast-chart-domain/src/viewport.rs` — `Viewport` holds `Vec<PriceScale>` instead of scalar min/max
- `fast-chart-core/src/app/pane.rs` — `Pane` holds price scale references
- `fast-chart-core/src/app/viewport_management.rs` — auto-fit per price scale
- `fast-chart-app/src/adapters/gpu_renderer.rs` — render multiple y-axes

### 3. Plugin System (P1)

**Current**: No plugin system. Series rendering is hardcoded in `GpuRenderer` dispatch (match on `SeriesType`). Indicator trait exists but is separate from rendering.

**Gap**: Adding a new series type requires modifying `GpuRenderer` directly. No way for third-party code to add custom rendering primitives.

**Files affected**:
- New: `fast-chart-core/src/ports/plugin.rs` — plugin traits
- `fast-chart-core/src/app/chart_controller.rs` — plugin registry
- `fast-chart-app/src/adapters/gpu_renderer.rs` — dispatch through plugins

### 4. Magnet (P1)

**Current**: `Crosshair.price` is set by linear interpolation from screen_y (crosshair.rs line 29). `find_bar_at_crosshair()` exists in GpuRenderer (line 615) but only for tooltip display.

**Gap**: Crosshair price snaps to arbitrary interpolated price, not to nearest bar's O/H/L/C. No magnet mode.

**Files affected**:
- `fast-chart-domain/src/crosshair.rs` — magnet snap logic
- `fast-chart-core/src/app/chart_controller.rs` — magnet state
- `fast-chart-core/src/ports/interaction.rs` — magnet toggle command

### 5-7. Area/Baseline/Histogram Series (P2)

**Current**: `SeriesType` enum has `Area` and `Baseline` variants but NO rendering implementation. Only `CandleRenderer` and `LineRenderer` exist. `SeriesType::Bar` also unimplemented for rendering.

**Gap**: Enum variants exist as dead code. No triangle-strip fill renderer (Area/Baseline), no vertical-bar renderer (Histogram).

**Files affected**:
- New: `fast-chart-app/src/adapters/rendering/area_renderer.rs`
- New: `fast-chart-app/src/adapters/rendering/baseline_renderer.rs`
- New: `fast-chart-app/src/adapters/rendering/histogram_renderer.rs`
- New: WGSL shaders for each
- `fast-chart-app/src/adapters/gpu_renderer.rs` — dispatch to new renderers

### 8. Price Lines (P2)

**Current**: No horizontal line primitive. Price lines would be new domain objects rendered as thin quads.

**Files affected**:
- New: `fast-chart-domain/src/price_line.rs` — domain type
- New: `fast-chart-app/src/adapters/rendering/price_line_renderer.rs`
- `fast-chart-core/src/app/pane.rs` — price lines per pane

### 9. Markers Plugin (P3)

**Current**: No marker concept. Markers are point annotations (arrows, circles) at specific timestamps.

**Files affected**:
- New: `fast-chart-domain/src/marker.rs` — domain type
- Integrated into plugin system (improvement #3)

### 10. Localization + Kinetic Scroll (P3)

**Current**: `format_price()` and `format_price_short()` are hardcoded in gpu_renderer.rs (lines 804-823). Scrolling uses simple `ViewportCommand::PanBy` with fixed delta.

**Files affected**:
- `fast-chart-domain/src/scale.rs` — `PriceFormatter` trait
- `fast-chart-core/src/app/viewport_management.rs` — kinetic scroll momentum

---

## Architecture Assessment

| Aspect | Current | Gap vs lightweight-charts |
|--------|---------|--------------------------|
| Invalidation | `bool` + 4 dead booleans | Bitmask with pane granularity |
| Price scales | 1 per pane (implicit) | Multiple per pane (Left/Right/Overlay) |
| Series types | 5 enum variants, 2 rendered | Need 4 more renderers |
| Plugin system | None (hardcoded dispatch) | Trait-based extensibility |
| Crosshair | Linear interpolation | Magnet snap to OHLC |
| Price lines | None | Horizontal primitives |
| Markers | None | Point annotations |
| Formatters | Hardcoded `format!` | Pluggable `PriceFormatter` trait |
| Scrolling | Fixed-step pan | Kinetic momentum |

---

## Dependency Graph Between Improvements

```
PR1: InvalidationMask ──────────────────────────────────────────┐
PR2: Price Scales ──────────────────────┐                       │
PR3: Plugin System ─────────────────────┼───────────────────────┤
PR4: Magnet + Price Lines ──────────────┼───────────────────────┤
PR5: Area + Baseline + Histogram ───────┴─── (needs plugins) ──┤
PR6: Markers + Localization + Kinetic Scroll ───────────────────┘
```

- PR1 (Invalidation) is independent — foundational, no dependencies
- PR2 (Price Scales) is independent — foundational, no dependencies
- PR3 (Plugin System) depends on PR1 + PR2 being stable
- PR4 (Magnet + Price Lines) depends on PR1 (invalidation for crosshair snap) + PR2 (price lines need price scales)
- PR5 (Area/Baseline/Histogram) depends on PR3 (plugin system for new renderers)
- PR6 (Markers + Localization) depends on PR3 (markers as plugin) + PR2 (formatters)

---

## Risks

1. **GPU shader proliferation**: Each new series type needs a new WGSL shader. Risk of shader maintenance burden.
2. **Invalidation correctness**: Bitmask invalidation is subtle — missed invalidation causes visual glitches. Thorough test suite needed.
3. **Price scale complexity**: Multiple price scales per pane complicates coordinate mapping and hit testing significantly.
4. **Plugin ABI stability**: Trait-based plugins must be designed carefully to avoid breaking changes as the system evolves.
5. **Performance regression risk**: Area/Baseline fill rendering (triangle strips) must not regress the 60fps target.
