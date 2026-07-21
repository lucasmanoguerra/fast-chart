# Pane Layout System Refactor — Technical Specification

**Date:** 2026-07-19
**Status:** Draft
**Scope:** fc-app, fc-render, fc-examples

---

## 1. Problem Statement

The pane layout system has **3 critical structural gaps** that prevent multi-pane rendering:

| # | Gap | Impact |
|---|-----|--------|
| 1 | Main series renderers (candle, line, area, histogram, baseline) render to the **entire canvas** without per-pane scissor clipping | Multi-pane charts render all series on top of each other |
| 2 | `LayoutEngine` trait (stateless rect computation) is **disconnected** from `LayoutManager` (runtime state + drag) | Two independent systems that should be one |
| 3 | `PaneDivider` and `LayoutManager` have **duplicated drag logic** | Two independent drag implementations that can diverge |

---

## 2. Current Architecture

```
fc-app/src/app/
├── layout/
│   └── mod.rs           — LayoutEngine trait + VerticalStack/HorizontalSplit/GridLayout
├── layout_manager.rs    — LayoutManager (holds Panes, dividers, drag logic)
├── pane/
│   ├── mod.rs           — Pane struct (viewport, series, layers, scales, drawings)
│   ├── divider.rs       — PaneDivider (hit-test, drag, cursor)
│   └── events.rs        — PaneEvent enum
└── chart_controller.rs  — ChartState (single viewport, single pane_heights)

fc-examples/src/adapters/
└── gpu_renderer.rs      — GpuRenderer::render() (lines 277-441)
```

### What works
- `Pane` struct is complete (viewport, series, layers, scales, drawings, markers, price lines)
- `LayoutManager` handles vertical stacking, dividers, add/remove, time sync
- `PaneDivider` has standalone hit-testing and cursor management
- `ScissorManager` in wgpu crate supports stack-based scissor rects

### What's broken
1. `GpuRenderer::render()` lines 393-413: candles/line/area/histogram/baseline render without scissor
2. `LayoutEngine::compute_rects()` is never called by `LayoutManager`
3. `PaneDivider::update_drag()` and `LayoutManager::update_drag()` do the same thing differently

---

## 3. Proposed Architecture

### 3.1 Gap 1: Per-Pane Scissor for Main Series Renderers

**Goal:** Each pane's series renderers render only within that pane's pixel bounds.

**Approach: Per-Pane Render Pass Loop**

The current render method has a single pass for all series. The fix wraps each series renderer in a per-pane scissor loop:

```
Current:
  grid.render()
  candle.render()        ← renders to ENTIRE canvas
  line.render()          ← renders to ENTIRE canvas
  area.render()          ← renders to ENTIRE canvas
  divider.render()
  crosshair.render()
  [per-pane scissor] marker.render() + price_line.render()

Proposed:
  for pane_idx in 0..layout.panes.len():
    set_scissor_rect(pane.offset, pane.height)
    grid.render(pane)
    candle.render(pane)
    line.render(pane)
    area.render(pane)
    histogram.render(pane)
    baseline.render(pane)
  reset_scissor()
  divider.render()
  crosshair.render()
  [per-pane scissor] marker.render() + price_line.render()
  text.render()
```

**Prerequisites:**
- Each renderer needs to accept a `PaneRef` (viewport + price scale) to generate correct vertices
- `ChartState` must expose per-pane viewports (currently only has one global viewport)
- Vertex generation must use pane-specific coordinate transforms

**Files to modify:**
- `fc-examples/src/adapters/gpu_renderer.rs` — restructure render loop
- `fc-app/src/app/chart_controller.rs` — expose per-pane viewports from LayoutManager
- `fc-render/src/coordinates.rs` — CoordinatePipeline accepts per-pane area bounds

**Implementation steps:**

1. **Add `PaneRef` to renderers:**
   ```rust
   pub struct PaneRef {
       pub viewport: Viewport,
       pub price_scales: Vec<PriceScale>,
       pub pixel_offset: f32,  // y offset from canvas top
       pub pixel_height: f32,  // pane height in pixels
   }
   ```

2. **Modify `update_*_from_state` methods** to accept `&[PaneRef]` and generate per-pane vertex data. Each pane gets its own vertex range in the buffer.

3. **Modify render loop** to iterate per-pane with scissor rects:
   ```rust
   for pane in &pane_refs {
       render_pass.set_scissor_rect(0, pane.pixel_offset, w, pane.pixel_height);
       self.candle_renderer.render_pane(&mut render_pass, pane.index);
       self.line_renderer.render_pane(&mut render_pass, pane.index);
       // ... etc
   }
   ```

4. **Wire `ChartState` to `LayoutManager`** — add method to extract `Vec<PaneRef>` from layout + per-pane viewports.

**Risk:** Medium — vertex buffer layout changes affect all renderers. Mitigated by keeping single-buffer approach with per-pane index ranges.

---

### 3.2 Gap 2: Integrate LayoutEngine with LayoutManager

**Goal:** `LayoutManager` delegates rect computation to `LayoutEngine` instead of doing its own math.

**Current state:**
- `LayoutEngine::compute_rects(parent_rect, pane_count) -> Vec<Rect>` — stateless
- `LayoutManager::pane_y_offset(index)`, `pane_pixel_offset()`, `pane_pixel_height()` — inline math

**Proposed:**

`LayoutManager` owns a `Box<dyn LayoutEngine>` and delegates:

```rust
pub struct LayoutManager {
    engine: Box<dyn LayoutEngine>,        // NEW: delegates rect computation
    pub panes: Vec<Pane>,
    dividers: Vec<PaneDivider>,           // CHANGED: from Vec<f64> to Vec<PaneDivider>
    min_pane_height: f64,
    dragging_divider: Option<usize>,
}
```

**Key changes:**

1. **`LayoutManager::compute_rects(canvas: Rect) -> Vec<Rect>`** — delegates to `self.engine.compute_rects(canvas, self.panes.len())`, then adjusts for divider gaps.

2. **`pane_pixel_offset(index)` and `pane_pixel_height(index)`** — computed from `compute_rects()` result instead of inline math. This eliminates the manual `pane_y_offset` calculations.

3. **`engine()` accessor** — allows swapping layout strategies at runtime:
   ```rust
   layout.set_engine(VerticalStack::with_heights(vec![0.7, 0.2, 0.1]));
   ```

4. **`LayoutManager::new()` default** — uses `VerticalStack::new()` as default engine.

**Files to modify:**
- `fc-app/src/app/layout_manager.rs` — add engine field, delegate computations
- `fc-app/src/app/layout/mod.rs` — no changes to trait, add `update_heights()` method to VerticalStack

**Compatibility:**
- Existing `pane_y_offset()`, `pane_pixel_offset()`, `pane_pixel_height()` methods preserved but reimplemented via engine
- All existing tests pass without changes

---

### 3.3 Gap 3: Unify PaneDivider with LayoutManager

**Goal:** Eliminate duplicated drag logic. `PaneDivider` becomes a pure UI concern.

**Current state:**
- `PaneDivider::start_drag()` / `update_drag()` / `end_drag()` — manages its own position
- `LayoutManager::start_drag()` / `update_drag()` / `end_drag()` — manages divider positions via `dividers: Vec<f64>`

**Two systems, same job, not connected.**

**Proposed:**

`LayoutManager` owns `Vec<PaneDivider>` and manages all drag state. `PaneDivider` loses its drag methods and becomes a pure rendering/UI struct:

```rust
// PaneDivider — UI only
pub struct PaneDivider {
    pub upper_pane_index: usize,
    pub position: f64,
    pub height: f32,
    pub hit_zone_height: f32,
    pub cursor: DividerCursor,
    // REMOVED: is_dragging, min_position, max_position
    // REMOVED: start_drag(), update_drag(), end_drag(), set_hover()
}

// LayoutManager — owns all state + drag logic
impl LayoutManager {
    pub fn hit_test_divider(&self, screen_y: f64, canvas_height: f64) -> Option<usize> {
        self.dividers.iter().position(|d| d.hit_test(screen_y, canvas_height))
    }

    pub fn start_drag(&mut self, divider_index: usize) {
        self.dragging_divider = Some(divider_index);
    }

    pub fn update_drag(&mut self, delta_y: f64, canvas_height: f64) {
        if let Some(idx) = self.dragging_divider {
            let delta_frac = delta_y / canvas_height;
            let new_pos = self.dividers[idx].position + delta_frac;
            // ... clamp, update pane heights, update divider position
            self.dividers[idx].position = clamped;
            self.panes[idx].height = clamped - prev;
            self.panes[idx + 1].height = next - clamped;
        }
    }

    pub fn end_drag(&mut self) {
        self.dragging_divider = None;
    }
}
```

**Files to modify:**
- `fc-app/src/app/pane/divider.rs` — remove drag methods, keep hit-test/rect/cursor
- `fc-app/src/app/layout_manager.rs` — change `dividers: Vec<f64>` to `dividers: Vec<PaneDivider>`, consolidate drag logic
- `fc-app/src/app/pane/events.rs` — remove duplicate events (keep PaneEvent variants)

**Compatibility:**
- `PaneDivider::hit_test()`, `rect()`, `hit_rect()`, `set_hover()` preserved
- `LayoutManager::hit_test_divider()` now delegates to `PaneDivider::hit_test()`
- `LayoutManager::update_drag()` computes new position and updates `PaneDivider.position`

---

## 4. Implementation Order

| Phase | What | Files | Risk | Tests |
|-------|------|-------|------|-------|
| **1** | Unify PaneDivider (Gap 3) | divider.rs, layout_manager.rs | Low | Existing + new integration tests |
| **2** | Integrate LayoutEngine (Gap 2) | layout_manager.rs, layout/mod.rs | Low | Existing + new engine delegation tests |
| **3** | Per-pane scissor (Gap 1) | gpu_renderer.rs, chart_controller.rs, coordinates.rs | Medium | New multi-pane rendering tests |

**Rationale:** Phase 1-2 are pure refactors (no behavior change). Phase 3 depends on clean layout state from phases 1-2.

---

## 5. Test Plan

### Phase 1 tests
- `pane_divider_hit_test_via_layout_manager` — hit test delegates to PaneDivider
- `pane_divider_drag_updates_position` — drag updates PaneDivider.position
- `pane_divider_drag_respects_constraints` — min/max height enforcement
- `pane_divider_cursor_changes` — hover/drag cursor states

### Phase 2 tests
- `layout_manager_delegates_to_engine` — compute_rects calls engine
- `layout_manager_swappable_engine` — change engine at runtime
- `vertical_stack_with_divider_gaps` — rects account for divider thickness
- `pane_pixel_dimensions_from_engine` — offset/height computed from engine rects

### Phase 3 tests
- `multi_pane_scissor_rendering` — each pane renders within its bounds
- `per_pane_viewport_coordination` — pane 0 (candles) and pane 1 (volume) have different y-ranges
- `divider_between_panes` — visual separator between rendered panes

---

## 6. Migration Notes

- **No breaking public API changes** — all existing methods preserved
- **`ChartState.pane_heights`** — deprecated in favor of `LayoutManager::compute_rects()`
- **`PaneDivider`** — loses `is_dragging`, `min_position`, `max_position` fields (were only used internally)
- **GpuRenderer** — `render()` signature unchanged, internal loop structure changes

---

## 7. Future Work (Out of Scope)

- Horizontal split rendering (Gap 2 enables this)
- Grid layout rendering (Gap 2 enables this)
- Nested layouts (requires recursive pane tree)
- Per-pane dirty tracking (PaneBitmask → per-pane vertex buffer invalidation)
- Pane resize animation (lerp between divider positions)
