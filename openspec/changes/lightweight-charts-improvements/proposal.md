# Proposal: Lightweight-Charts Inspired Improvements

## Intent

After 10 PRs and 187 tests building a GPU-accelerated trading chart, the architecture needs maturity improvements inspired by TradingView's lightweight-charts v5. The current codebase has solid foundations (hexagonal architecture, ring-buffer TimeSeries, sub-renderer pipeline) but lacks: granular invalidation, multiple price scales, extensibility, magnet snapping, and 3 of 5 declared series types have no renderer. This change addresses all gaps in 6 reviewable PRs.

## Scope

### In Scope
- InvalidationMask replacing boolean flags
- Multiple price scales per pane (Left/Right/Overlay)
- Trait-based plugin system for extensibility
- Crosshair magnet snap to OHLC
- Area, Baseline, and Histogram renderers + shaders
- Price Lines (horizontal primitives)
- Markers plugin (buy/sell arrows)
- PriceFormatter trait (localization-ready)
- Kinetic scroll momentum

### Out of Scope
- Touch/mobile gestures
- Real-time WebSocket data provider
- Technical indicator UI (already working via IndicatorRegistry)
- Theme system / dark mode toggle
- Export to image/PNG

## Capabilities

### New Capabilities
- `invalidation-system`: Bitmask-based invalidation with pane granularity
- `price-scales`: Multiple price scales per pane with formatters
- `plugin-system`: Trait-based extensibility for custom series and primitives
- `magnet-mode`: Crosshair snap-to-nearest-OHLC
- `area-series`: Area fill renderer (gradient support)
- `baseline-series`: Two-color baseline renderer
- `histogram-series`: Vertical bar renderer with per-bar coloring
- `price-lines`: Horizontal price level primitives
- `markers`: Point annotation plugin (arrows, circles)
- `kinetic-scroll`: Momentum-based scrolling

### Modified Capabilities
- `domain-model`: Add PriceLine, Marker, PriceFormatter types
- `gpu-renderer`: Plugin dispatch, multiple price scale rendering, new sub-renderers
- `interaction`: Magnet toggle, kinetic scroll input handling
- `multi-pane-layout`: Price scale assignment per pane

## Approach

**6 PRs**, each independently reviewable and testable:

| PR | Title | Scope | Est. Lines |
|----|-------|-------|-----------|
| 1 | InvalidationMask | `InvalidationMask` enum + pane bitmasks, replace all bools | ~250 |
| 2 | Price Scales | `PriceScaleId`, multi-scale per pane, `PriceFormatter` trait | ~350 |
| 3 | Plugin System | `SeriesRenderer` + `PanePrimitive` traits, registry, refactor dispatch | ~300 |
| 4 | Magnet + Price Lines | Crosshair snap, PriceLine domain + renderer | ~300 |
| 5 | Area/Baseline/Histogram | 3 new renderers + WGSL shaders | ~400 |
| 6 | Markers + Localization + Kinetic Scroll | Marker plugin, formatter swap, momentum scroll | ~250 |

**Total**: ~1,850 lines across 6 PRs (within 400-line-per-PR budget via chaining).

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `fast-chart-domain/src/scale.rs` | Modified | PriceScaleId, PriceFormatter trait |
| `fast-chart-domain/src/viewport.rs` | Modified | Multiple price scales |
| `fast-chart-domain/src/crosshair.rs` | Modified | Magnet snap logic |
| `fast-chart-domain/src/price_line.rs` | New | PriceLine domain type |
| `fast-chart-domain/src/marker.rs` | New | Marker domain type |
| `fast-chart-core/src/app/chart_controller.rs` | Modified | InvalidationMask, plugin registry |
| `fast-chart-core/src/app/pane.rs` | Modified | Price scale refs, price lines |
| `fast-chart-core/src/ports/plugin.rs` | New | Plugin traits |
| `fast-chart-core/src/ports/interaction.rs` | Modified | Magnet toggle, kinetic scroll |
| `fast-chart-app/src/adapters/gpu_renderer.rs` | Modified | Mask dispatch, multi-scale, plugin call |
| `fast-chart-app/src/adapters/rendering/area_renderer.rs` | New | Area fill renderer |
| `fast-chart-app/src/adapters/rendering/baseline_renderer.rs` | New | Baseline renderer |
| `fast-chart-app/src/adapters/rendering/histogram_renderer.rs` | New | Histogram renderer |
| `fast-chart-app/src/adapters/rendering/price_line_renderer.rs` | New | Horizontal line renderer |
| `fast-chart-app/src/adapters/rendering/shaders/*.wgsl` | New | 3-4 new WGSL shaders |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Invalidation bitmask misses cause visual glitches | Medium | Property tests: every state mutation must produce correct mask bits |
| Multiple price scales complicate coordinate mapping | High | Start with 2 max per pane; unit-test map_to_y/map_from_y per scale |
| New WGSL shaders regress GPU perf | Low | Benchmark before/after each renderer PR; budget = <2ms per frame |
| Plugin trait design breaks on iteration | Medium | Ship with `#[non_exhaustive]` on trait; use cfg-gated unstable features |
| PR chain review fatigue (6 PRs) | Medium | Each PR is self-contained; PR5 is the largest but mechanically simple |

## Rollback Plan

Each PR is independently revertible via `git revert`. No PR depends on a later PR for correctness:
- PR1 revert: restore boolean flags (trivial)
- PR2 revert: remove PriceScaleId, restore scalar min/max
- PR3 revert: remove plugin traits, restore hardcoded dispatch
- PR4 revert: remove magnet + price lines
- PR5 revert: remove 3 renderers + shaders
- PR6 revert: remove markers, formatter, kinetic scroll

## Dependencies

- No new external crate dependencies (all wgpu/shader work uses existing stack)
- `bytemuck` already in use for vertex/uniform types
- Potential future: `icu_locale` for full i18n (out of scope for PR6 — trait-only)

## Success Criteria

- [ ] All 187 existing tests still pass after each PR
- [ ] New tests cover: invalidation correctness, price scale mapping, magnet snap accuracy, each new renderer
- [ ] 60fps maintained with 100K bars + Area series + 3 indicators
- [ ] Plugin trait allows adding a custom series type without modifying core/app crates
- [ ] PriceFormatter trait can swap decimal formatting without touching renderer code
- [ ] Kinetic scroll decelerates smoothly over 300ms
