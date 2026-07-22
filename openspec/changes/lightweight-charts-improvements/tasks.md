# Tasks: InvalidationMask + Price Scales

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~550 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 (Tasks 1–5) → PR 2 (Tasks 6–12) |
| Delivery strategy | ask-on-risk |
| Chain strategy | feature-branch-chain |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: feature-branch-chain
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| PR1: InvalidationMask | Replace boolean dirty flags with bitmask | PR 1 | `cargo test -p fc-types -- invalidation` | `cargo test -p fast-chart-core` (chart_controller) | `fc-types/src/invalidation.rs` + call-site edits; revert restores booleans |
| PR2: Price Scales | Named per-pane coordinate systems | PR 2 | `cargo test -p fc-types -- price_scale` | `cargo test -p fast-chart-core` (pane tests) | `fc-types/src/price_scale.rs` + viewport/pane edits; revert restores flat value_min/max |

## PR 1: InvalidationMask

### Phase 1: Domain Types

- [x] 1.1 Create `fc-types/src/invalidation.rs` — `InvalidationLevel` enum (Nothing/Cursor/Light/Full, `#[repr(u8)]`, derives `PartialOrd`), `PaneBitmask` newtype over `u32` with `EMPTY`/`ALL` constants, `single()`, `contains()`, `union()`, `is_empty()`, `BitOr` impl. Zero external deps.
- [x] 1.2 Add `InvalidationMask` struct in same file — fields `level: InvalidationLevel`, `panes: PaneBitmask`. Methods: `new()`, `mark(&mut self, level, panes)` (max-level union + pane OR), `contains(&self, level, pane)` (level >= threshold AND pane bit set), `clear()`, `level()`, `panes()`.
- [x] 1.3 Add `#[cfg(test)] mod tests` in `invalidation.rs` — 10 tests: `default_is_nothing`, `mark_union_level_wins`, `mark_no_downgrade`, `mark_union_panes_accumulate`, `contains_level_hierarchy`, `contains_wrong_level`, `contains_wrong_pane`, `clear_resets`, `nothing_mark_is_noop`, `all_panes_covers_32`.
- [x] 1.4 Add `pub mod invalidation;` to `fc-types/src/lib.rs`.

### Phase 2: ChartState Migration

- [x] 2.1 In `fast-chart-core/src/app/chart_controller.rs`: add `use fc_types::invalidation::*;` to imports. Replace `pub needs_redraw: bool` in `ChartState` with `pub invalidation: InvalidationMask`. Initialize as `InvalidationMask::new()` in `ChartState::new()`.
- [x] 2.2 Replace all `self.state.needs_redraw = true` in `tick()` and `handle_input()` with `self.state.invalidation.mark(Full, PaneBitmask::ALL)` or `mark(Cursor, PaneBitmask::ALL)` per the mapping in design.md (crosshair → Cursor, everything else → Full).
- [x] 2.3 Replace render gate `if self.state.needs_redraw` with `if self.state.invalidation.level() > InvalidationLevel::Nothing`. Replace `self.state.needs_redraw = false` after render with `self.state.invalidation.clear()`.
- [x] 2.4 Update existing tests in `chart_controller.rs` — replace `needs_redraw` assertions with `invalidation.level() == InvalidationLevel::Nothing` checks. Update `ChartStateSnapshot` mock field.

### Phase 3: GpuRenderer Migration

- [x] 3.1 In `fast-chart-app/src/adapters/gpu_renderer.rs`: add `use fc_types::invalidation::*;`. Replace 4 `needs_*: bool` fields with `invalidation: InvalidationMask`.
- [x] 3.2 Replace all `self.needs_line_update = true` / `self.needs_candle_update = true` / `self.needs_text_update = true` / `self.needs_divider_update = true` assignments with appropriate `self.invalidation.mark(...)` calls.
- [x] 3.3 Replace all `if self.needs_*` dispatch checks in `render()` with `if self.invalidation.contains(Full, pane_index)` or `self.invalidation.contains(Light, pane_index)` as appropriate.
- [x] 3.4 Update `GpuRenderer::new()` — initialize `invalidation: InvalidationMask::new()` (with `divider` and `text` pre-marked as in current init).

### Verification

- [x] 4.1 Run `cargo test -p fc-types` — all invalidation unit tests pass.
- [x] 4.2 Run `cargo test -p fast-chart-core` — chart_controller tests pass with mask-based redraw logic.
- [x] 4.3 Run `cargo test` — full workspace green, no regressions.

---

## PR 2: Price Scales

**Depends on: PR 1 merged** (PaneBitmask and InvalidationMask are stable; this PR builds on clean domain).

### Phase 1: Domain Types

- [x] 5.1 Create `fc-types/src/price_scale.rs` — `PriceScaleId` enum (Left, Right, Overlay(String)), derives `PartialEq`, `Eq`, `Hash`, `Clone`. Add `is_overlay(name: &str) -> Self` helper.
- [x] 5.2 Add `PriceScaleMode` enum (Normal, Logarithmic, Percentage), `PriceScaleOptions` struct (visible, auto_scale, mode, scale_offset), `Default` impl (true, true, Normal, 0.05).
- [x] 5.3 Add `PriceScale` struct (id, options, value_min, value_max). Methods: `new(id, options)`, `auto_fit(&mut self, data_min, data_max)` with scale_offset padding and zero-range guard, `contains(&self, price) -> bool`.
- [x] 5.4 Add `PriceFormatter` trait (`Send + Sync`, `format(f64) -> String`, `format_short(f64) -> String`). Add `DefaultPriceFormatter` struct with `decimal_places: Option<usize>`. Implement auto-detect precision: >=1.0 → 2 decimals, <0.01 → 5 decimals, else 4. Explicit `Some(n)` overrides.
- [x] 5.5 Add `#[cfg(test)] mod tests` in `price_scale.rs` — tests for `PriceScaleId` equality, `auto_fit` padding, `auto_fit` disabled noop, zero-range auto_fit, `DefaultPriceFormatter` auto-detect, explicit decimals, `format_short`.
- [x] 5.6 Add `pub mod price_scale;` to `fc-types/src/lib.rs`.

### Phase 2: Viewport + Pane Integration

- [x] 6.1 In `fc-types/src/viewport.rs`: add `use crate::price_scale::PriceScale;`. Add `price_to_y(&self, price: f64, scale: &PriceScale, pane_height: f32) -> f32` — ratio mapping with y-flip, clamp, zero-range midpoint fallback.
- [x] 6.2 Add `y_to_price(&self, y: f32, scale: &PriceScale, pane_height: f32) -> f64` — inverse mapping with clamp and zero-range fallback.
- [x] 6.3 Add viewport tests: `price_to_y_midpoint`, `price_to_y_top`, `price_to_y_bottom`, `y_to_price_roundtrip`, `price_to_y_zero_range`, `price_to_y_clamps`.
- [x] 6.4 In `fast-chart-core/src/app/pane.rs`: add `use fc_types::price_scale::*;`. Add `price_scales: Vec<PriceScale>` and `primary_scale_id: PriceScaleId` fields to `Pane`. Initialize `primary_scale_id` as `PriceScaleId::Left` in `Pane::new()`.
- [x] 6.5 Add methods to `Pane`: `ensure_price_scales()` (push Left + Right defaults if empty), `price_scale(&self, id) -> Option<&PriceScale>`, `price_scale_mut(&mut self, id) -> Option<&mut PriceScale>`, `primary_scale(&self) -> &PriceScale`.
- [x] 6.6 Add `price_scale_id: PriceScaleId` to `SeriesRef` with `Default` impl (Left). Update `Pane::add_series()` to default the new field. Add `Default` impl for `SeriesRef`.

### Phase 3: Renderer Wiring

- [x] 7.1 In `fast-chart-app/src/adapters/gpu_renderer.rs`: add `use fc_types::price_scale::PriceScale;`. Update `update_line_from_vec()` signature to accept `&PriceScale` (or look up from pane). Delegate coordinate math to `Viewport::price_to_y()`.
- [x] 7.2 Update `screen_y_to_price()` to accept `&PriceScale` and use `Viewport::y_to_price()`.
- [x] 7.3 Replace standalone `format_price()` calls with `PriceFormatter::format()` on the target series' scale formatter.

### Phase 4: Tests & Verification

- [x] 8.1 Run `cargo test -p fc-types` — price_scale and viewport mapping tests pass.
- [x] 8.2 Run `cargo test -p fast-chart-core` — pane tests pass with price_scales field.
- [x] 8.3 Run `cargo test` — full workspace green, no regressions.

## Dependency Graph

```
PR1: 1.1 → 1.2 → 1.3 → 1.4 → 2.1 → 2.2 → 2.3 → 2.4 → 3.1 → 3.2 → 3.3 → 3.4 → 4.1 → 4.2 → 4.3
PR2: 5.1 → 5.2 → 5.3 → 5.4 → 5.5 → 5.6 → 6.1 → 6.2 → 6.3 → 6.4 → 6.5 → 6.6 → 7.1 → 7.2 → 7.3 → 8.1 → 8.2 → 8.3
PR2 depends on PR1 complete.
```

## PR Boundaries

| PR | Tasks | Lines est. | Scope |
|----|-------|------------|-------|
| PR 1 | 1.1–4.3 (14 tasks) | ~200 | InvalidationMask domain type + ChartState + GpuRenderer boolean replacement |
| PR 2 | 5.1–8.3 (14 tasks) | ~350 | PriceScale domain type + Viewport methods + Pane integration + Renderer wiring |

## Focused Test Commands

```bash
# PR 1 — InvalidationMask
cargo test -p fc-types -- invalidation
cargo test -p fast-chart-core -- chart_controller

# PR 2 — Price Scales
cargo test -p fc-types -- price_scale
cargo test -p fc-types -- viewport::tests
cargo test -p fast-chart-core -- pane

# Full regression (both PRs)
cargo test
```
