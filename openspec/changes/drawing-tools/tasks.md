# Tasks: Drawing Trait Unification + Collection Cleanup

## Overview

- **Total tasks**: 6
- **Estimated total lines**: ~1800 new, ~4100 deleted (net ~-2300)
- **Critical path**: Task 1 → Task 2 → Task 3 → Task 4 → Task 5 → Task 6

## Tasks

### Task 1: Create fc-drawing crate skeleton

**Files**: `fc-drawing/Cargo.toml`, `fc-drawing/src/lib.rs`, `fc-drawing/src/trait_def.rs`, `fc-drawing/src/hit.rs`, `fc-drawing/src/bounds.rs`
**Estimated lines**: ~200
**Dependencies**: None

**Acceptance criteria**:
- [ ] `fc-drawing/Cargo.toml` exists with correct dependencies (fc-domain, fc-render, serde, uuid)
- [ ] `fc-drawing/src/lib.rs` exports all public modules
- [ ] `Drawing` trait defined with all 8 methods (id, move_by, bounds, to_commands, hit_test, is_selected, set_selected, as_any, as_any_mut)
- [ ] `HitResult` enum defined (Miss, Body, ControlPoint(usize))
- [ ] `DrawingBounds` struct defined with `combine()` and `contains_point()` methods
- [ ] `default_aabb_hit_test()` function implemented
- [ ] `cargo build -p fc-drawing` succeeds

---

### Task 2: Move domain types to fc-drawing

**Files**: `fc-drawing/src/types/*.rs` (15 files + mod.rs)
**Estimated lines**: ~1500 (move, not write)
**Dependencies**: Task 1

**Acceptance criteria**:
- [ ] All 15 types exist in `fc-drawing/src/types/`
- [ ] Each type derives Debug, Clone, Serialize, Deserialize (with serde feature)
- [ ] Each type has `pub id: DrawingId` field
- [ ] Each type has `pub selected: bool` field (default false)
- [ ] Each type implements `Drawing` trait
- [ ] 12 types use default `hit_test` (TrendLine, Arrow, Segment, TextDrawing, ImageDrawing, LabelDrawing, HorizontalLine, VerticalLine, Rectangle, FibonacciRetracement, FibonacciExtension, Ellipse)
- [ ] 3 types override `hit_test` (Ray, Path, Pitchfork)
- [ ] `cargo build -p fc-drawing` succeeds with all types

---

### Task 3: Update fc-domain re-exports

**Files**: `fc-domain/src/drawing/mod.rs`, `fc-domain/Cargo.toml`
**Estimated lines**: ~30
**Dependencies**: Task 2

**Acceptance criteria**:
- [ ] `fc-domain/Cargo.toml` depends on `fc-drawing`
- [ ] `fc-domain/src/drawing/mod.rs` re-exports `fc_drawing::*`
- [ ] `DrawingId` and `ChartPoint` remain in fc-domain (domain primitives)
- [ ] `cargo build -p fc-domain` succeeds
- [ ] Existing fc-domain tests pass

---

### Task 4: Update fc-render to use fc-drawing

**Files**: `fc-render/src/drawing.rs` (delete), `fc-render/src/drawing_manager.rs`, `fc-render/src/drawing_interaction.rs`, `fc-render/Cargo.toml`
**Estimated lines**: ~80 (rewrite drawing_manager.rs)
**Dependencies**: Task 3

**Acceptance criteria**:
- [ ] `fc-render/src/drawing.rs` deleted (trait moved to fc-drawing)
- [ ] `fc-render/Cargo.toml` depends on `fc-drawing`
- [ ] `drawing_manager.rs` imports from `fc_drawing`
- [ ] `hit_test()` is single-loop: `for drawing in self.drawings.iter()`
- [ ] `bounds()` is single-loop: `self.drawings.iter().map(|d| d.bounds()).reduce(...)`
- [ ] `render()` is single-loop: `self.drawings.iter().flat_map(|d| d.to_commands(ctx)).collect()`
- [ ] `cargo build -p fc-render` succeeds
- [ ] All fc-render tests pass

---

### Task 5: Update all workspace callers

**Files**: All files importing from `fc_domain::drawing` or `fc_render::drawing`
**Estimated lines**: ~50 (import changes)
**Dependencies**: Task 4

**Acceptance criteria**:
- [ ] `cargo build` succeeds for entire workspace
- [ ] No "unresolved import" errors
- [ ] All `add_trend_line(x)` → `add(Box::new(x))`
- [ ] All `get_trend_line(id)` → `iter().find(|d| d.id() == id).and_then(|d| d.as_any().downcast_ref::<TrendLine>())`
- [ ] All `all_trend_lines()` → `iter().filter_map(|d| d.as_any().downcast_ref::<TrendLine>())`

---

### Task 6: Update tests and verify

**Files**: All test files
**Estimated lines**: ~100 (test updates)
**Dependencies**: Task 5

**Acceptance criteria**:
- [ ] `cargo test` passes for entire workspace (1387+ tests)
- [ ] `cargo clippy` produces 0 warnings
- [ ] All drawing-specific tests updated for new API
- [ ] No regressions in existing functionality

---

## Parallelization

- Tasks 1-6 must run sequentially (critical path)
- Within Task 2, the 15 type files can be created in parallel
- Task 6 can run partial verification after Task 4

## Risk Mitigation

- Each task has clear acceptance criteria
- `cargo build` after each task catches import errors early
- Existing 1387 tests provide safety net
- Git commit after each task enables easy rollback
