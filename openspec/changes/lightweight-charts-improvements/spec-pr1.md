# InvalidationMask Specification

## Goal

Replace scattered boolean dirty flags in `GpuRenderer` and `ChartState` with a single bitmask struct that encodes both invalidation level and per-pane granularity.

## Data Structures

```rust
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InvalidationLevel {
    Nothing  = 0, // No work needed
    Cursor   = 1, // Crosshair/cursor redraw only (cheapest)
    Light    = 2, // Text labels, axis ticks
    Full     = 3, // Vertex buffers, uniforms, all layers
}

bitflags::bitflags! {
    pub struct PaneBitmask: u32 {
        const PANE_0 = 1 << 0;
        const PANE_1 = 1 << 1;
        const PANE_2 = 1 << 2;
        const ALL_PANES = 0xFFFF_FFFF;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InvalidationMask {
    level: InvalidationLevel,
    panes: PaneBitmask,
}
```

## API Surface

```rust
impl InvalidationMask {
    pub fn new() -> Self;                                    // Nothing, no panes
    pub fn mark(&mut self, level: InvalidationLevel, panes: PaneBitmask); // Union: take max level
    pub fn contains(&self, level: InvalidationLevel, pane: u32) -> bool; // level >= threshold AND pane bit set
    pub fn clear(&mut self);                                 // Reset to Nothing + empty panes
    pub fn level(&self) -> InvalidationLevel;                // Read current max level
    pub fn panes(&self) -> PaneBitmask;                      // Read pane bits
}
```

## Requirements

### Requirement: InvalidationLevel ordering

The four levels MUST form a strict total ordering: Nothing < Cursor < Light < Full. Higher levels subsume lower levels. If `mark(Full, PANE_0)` is called followed by `mark(Light, PANE_0)`, the result MUST remain Full.

#### Scenario: Higher level wins on union

- GIVEN an empty mask
- WHEN `mark(Cursor, PANE_0)` then `mark(Full, PANE_0)`
- THEN `level()` returns `Full`

#### Scenario: Lower level does not downgrade

- GIVEN a mask at `Full` for `PANE_0`
- WHEN `mark(Cursor, PANE_0)`
- THEN `level()` remains `Full`

### Requirement: Pane bitmask accumulation

`mark()` MUST union the pane bitmask. Calling `mark(Full, PANE_0)` then `mark(Light, PANE_1)` MUST result in panes `PANE_0 | PANE_1` at level `Full`.

#### Scenario: Multiple panes accumulate

- GIVEN an empty mask
- WHEN `mark(Light, PANE_0)` then `mark(Light, PANE_1)`
- THEN `contains(Light, 0)` and `contains(Light, 1)` are both true

#### Scenario: ALL_PANES bitmask

- GIVEN an empty mask
- WHEN `mark(Full, PaneBitmask::ALL_PANES)`
- THEN `contains(Full, 0)`, `contains(Full, 5)`, and `contains(Full, 31)` are all true

### Requirement: contains level semantics

`contains(level, pane)` MUST return true only if the mask's stored level is >= the queried level AND the pane bit is set. This enables callers to check "is there at least Cursor-level work for pane 0?" and get true even if the mask is at Full.

#### Scenario: Full mask satisfies Light query

- GIVEN a mask at `Full` for `PANE_0`
- WHEN `contains(Light, 0)` is called
- THEN it returns true

#### Scenario: Light mask does not satisfy Full query

- GIVEN a mask at `Light` for `PANE_0`
- WHEN `contains(Full, 0)` is called
- THEN it returns false

#### Scenario: Wrong pane returns false

- GIVEN a mask at `Full` for `PANE_0`
- WHEN `contains(Full, 1)` is called
- THEN it returns false

### Requirement: clear resets completely

`clear()` MUST reset level to `Nothing` and pane bitmask to empty.

#### Scenario: Clear after marking

- GIVEN a mask with `Full` on `PANE_0 | PANE_1`
- WHEN `clear()` is called
- THEN `level()` returns `Nothing` and no pane bit is set

### Requirement: Default state is Nothing

A newly constructed mask MUST be `Nothing` with no panes set.

#### Scenario: New mask is clean

- GIVEN `InvalidationMask::new()`
- WHEN inspected
- THEN `level() == Nothing` and `panes() == empty`

### Requirement: Replace boolean flags in GpuRenderer

The four `needs_*` booleans in `GpuRenderer` (`needs_line_update`, `needs_candle_update`, `needs_text_update`, `needs_divider_update`) MUST be replaced by a single `InvalidationMask` field. All call sites that set these flags MUST be migrated to `mark()` calls.

#### Scenario: push_bars marks Full for pane 0

- GIVEN a `GpuRenderer` with a fresh `InvalidationMask`
- WHEN `push_bars()` is called with new data
- THEN `mask.contains(Full, 0)` is true

#### Scenario: set_crosshair marks Cursor for pane 0

- GIVEN a `GpuRenderer` with a fresh `InvalidationMask`
- WHEN `set_crosshair()` is called
- THEN `mask.contains(Cursor, 0)` is true and `mask.level() == Cursor`

#### Scenario: resize marks Full for all panes

- GIVEN a `GpuRenderer`
- WHEN `resize()` is called
- THEN `mask.contains(Full, 0)` and `mask.contains(Full, 1)` are true

### Requirement: Replace needs_redraw in ChartState

The `needs_redraw: bool` field in `ChartState` MUST be replaced by `InvalidationMask`. The `tick()` render check MUST use `mask.level() > Nothing` instead of `needs_redraw == true`.

#### Scenario: BarClosed marks Full on chart state

- GIVEN a `ChartState` with a fresh mask
- WHEN a `BarClosed` event is processed in `tick()`
- THEN the mask contains `Full` for all panes

#### Scenario: Render clears the mask

- GIVEN a `ChartState` with `Full` mask
- WHEN render completes
- THEN the mask is cleared to `Nothing`

### Requirement: Unit tests for bitmask operations

The `InvalidationMask` module MUST include unit tests covering: default state, mark union semantics, contains level hierarchy, clear, and multi-pane accumulation. At minimum 8 test cases.

#### Scenario: Test coverage

- GIVEN the `invalidation` module
- WHEN `cargo test` runs
- THEN all bitmask tests pass

## Edge Cases

| Case | Expected Behavior |
|------|-------------------|
| `mark(Nothing, PANE_0)` | No-op — Nothing never upgrades anything |
| `contains(Nothing, 0)` on empty mask | Returns true (0 >= 0) — callers should check `level > Nothing` instead |
| 32+ panes | PaneBitmask is u32; max 32 panes. Exceeding panes index → panic in debug, wrap in release |

## Testing Strategy

- Pure unit tests in `fc-types/src/invalidation.rs` (or `fast-chart-core`)
- Property tests: for any sequence of `mark()` calls, `level()` equals max of all marked levels
- Integration: verify `GpuRenderer::render()` dispatches correct sub-renderers based on mask

## Files Affected

| File | Change |
|------|--------|
| `fc-types/src/invalidation.rs` | **New** — `InvalidationLevel`, `PaneBitmask`, `InvalidationMask` + tests |
| `fast-chart-core/src/app/chart_controller.rs` | Replace `needs_redraw: bool` with `invalidation: InvalidationMask` |
| `fast-chart-app/src/adapters/gpu_renderer.rs` | Replace 4 `needs_*` bools with `InvalidationMask`; update all call sites |
