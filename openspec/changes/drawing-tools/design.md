# Design: Drawing Trait Unification + Collection Cleanup

## Architecture

New crate `fc-drawing` lives between `fc-domain` and `fc-render` in the dependency graph. It owns the unified `Drawing` trait and all 15 concrete types. `fc-domain` re-exports everything for backward compat. `fc-render` imports from `fc-drawing` directly.

```
fc-primitives (no deps)
    ↑
fc-domain (imports fc-drawing for re-export)
    ↑
fc-drawing (UNIFIED: trait + 15 types + hit_test + bounds + DrawCommand)
    ↑
fc-render (imports fc-drawing::Drawing, removes its own trait)
```

## Unified Trait

```rust
// fc-drawing/src/trait_def.rs
pub trait Drawing: Send + Sync {
    fn id(&self) -> &DrawingId;
    fn move_by(&mut self, delta: ChartPoint);
    fn bounds(&self) -> DrawingBounds;
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand>;

    // Default implementations
    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        let b = self.bounds();
        // AABB check: is point within tolerance of bounds?
        // Returns HitResult::Body or HitResult::Miss
        default_aabb_hit_test(&b, point, tolerance)
    }

    fn is_selected(&self) -> bool { false }
    fn set_selected(&mut self, _selected: bool) {}

    // Type erasure
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    // Debug
    fn type_name(&self) -> &'static str { std::any::type_name::<Self>() }
}
```

### Why default `hit_test`?

Most types (Rectangle, Ellipse, HorizontalLine, VerticalLine, TrendLine, Arrow, Segment, TextDrawing, ImageDrawing, LabelDrawing, FibonacciRetracement, FibonacciExtension) can be hit-tested with an AABB bounds check. Only **Ray**, **Path**, and **Pitchfork** need custom overrides because their geometry extends beyond their bounding box or has complex shapes.

### Types with custom `hit_test` override

| Type | Reason |
|------|--------|
| `Ray` | Infinite extension beyond bounds — must check ray-line distance |
| `Path` | Complex multi-segment geometry — must check distance to each segment |
| `Pitchfork` | Three prongs extending beyond bounds — must check each prong |

All other types use the default AABB implementation.

## Crate Structure

```
fc-drawing/
├── Cargo.toml
└── src/
    ├── lib.rs              # pub mod + re-exports
    ├── trait_def.rs        # Drawing trait definition (~40 lines)
    ├── hit.rs              # HitResult, default_aabb_hit_test (~50 lines)
    ├── bounds.rs           # DrawingBounds struct + combine/contains (~60 lines)
    ├── types/
    │   ├── mod.rs           # pub use for all types
    │   ├── trend_line.rs
    │   ├── arrow.rs
    │   ├── ray.rs           # custom hit_test
    │   ├── segment.rs
    │   ├── text.rs
    │   ├── image.rs
    │   ├── label.rs
    │   ├── horizontal_line.rs
    │   ├── vertical_line.rs
    │   ├── rectangle.rs
    │   ├── fibonacci.rs
    │   ├── fibonacci_ext.rs
    │   ├── pitchfork.rs     # custom hit_test
    │   ├── ellipse.rs
    │   └── path.rs          # custom hit_test
    └── render.rs            # DrawingRenderContext, DrawCommand re-export (~30 lines)
```

**Note**: `DrawCommand` and `RenderContext` stay in `fc-render` — `fc-drawing` depends on `fc-render` for those types. This keeps the render pipeline in one place.

## DrawingSet Redesign

Before (436 lines, 45 typed methods):
```rust
impl DrawingSet {
    pub fn add_trend_line(&mut self, line: TrendLine) { ... }
    pub fn add_arrow(&mut self, arrow: Arrow) { ... }
    // ... 13 more add_* methods
    pub fn get_trend_line(&self, id: &DrawingId) -> Option<&TrendLine> { ... }
    // ... 14 more get_* methods
    pub fn all_trend_lines(&self) -> Vec<&TrendLine> { ... }
    // ... 14 more all_* methods
}
```

After (~60 lines, 6 generic methods):
```rust
impl DrawingSet {
    pub fn new() -> Self { ... }
    pub fn add(&mut self, drawing: Box<dyn Drawing>) { ... }
    pub fn iter(&self) -> impl Iterator<Item = &dyn Drawing> { ... }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut dyn Drawing> { ... }
    pub fn remove(&mut self, id: &DrawingId) -> bool { ... }
    pub fn len(&self) -> usize { ... }
    pub fn is_empty(&self) -> bool { ... }
}
```

**Callers that used `add_trend_line()` now do**: `set.add(Box::new(trend_line))`
**Callers that used `get_trend_line()` now do**: `set.iter().find(|d| d.id() == id).and_then(|d| d.as_any().downcast_ref::<TrendLine>())`

## DrawingManager Simplification

Before (15 loops × 3 methods = 45 loops):
```rust
pub fn hit_test(&self, ...) -> ... {
    for item in self.drawings.all_trend_lines() { ... }
    for item in self.drawings.all_arrows() { ... }
    // ... 13 more
}
pub fn bounds(&self) -> ... {
    for item in self.drawings.all_trend_lines() { ... }
    // ... 14 more
}
pub fn render(&self, ...) -> ... {
    for item in self.drawings.all_trend_lines() { ... }
    // ... 14 more
}
```

After (3 loops total):
```rust
pub fn hit_test(&self, point: ChartPoint, tolerance: f32) -> Option<(DrawingId, HitResult)> {
    for drawing in self.drawings.iter() {
        let result = drawing.hit_test(point, tolerance);
        if result != HitResult::Miss {
            return Some((drawing.id().clone(), result));
        }
    }
    None
}

pub fn bounds(&self) -> Option<DrawingBounds> {
    self.drawings.iter().map(|d| d.bounds()).reduce(|acc, b| acc.combine(&b))
}

pub fn render(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
    let mut cmds: Vec<DrawCommand> = self.drawings.iter()
        .flat_map(|d| d.to_commands(ctx))
        .collect();
    cmds.sort_by_key(|c| c.z_index());
    cmds
}
```

## Migration Strategy

### Step 1: Create `fc-drawing` crate
- Add to workspace `Cargo.toml`
- Define unified trait + HitResult + DrawingBounds
- Move all 15 types from `fc-domain/src/drawing/types.rs`
- Implement `Drawing` for each type (copy from `fc-render/src/drawing.rs`)

### Step 2: Update `fc-domain`
- Remove `types.rs` contents (moved to fc-drawing)
- `mod.rs` becomes re-export: `pub use fc_drawing::*;`
- Keep `DrawingId`, `ChartPoint` in fc-domain (they're domain primitives)

### Step 3: Update `fc-render`
- Delete `drawing.rs` (trait + all 15 impls moved to fc-drawing)
- `drawing_manager.rs` imports from `fc_drawing` instead
- Single-loop iteration in hit_test/bounds/render

### Step 4: Update all callers
- `grep` for `use fc_domain::drawing::` and `use fc_render::drawing::`
- Update imports to `use fc_drawing::`
- Replace `add_trend_line(x)` with `add(Box::new(x))`
- Replace `get_trend_line(id)` with `iter().find(...).downcast_ref()`

### Step 5: Update tests
- 1387 existing tests should pass with import changes
- 20+ drawing-specific tests updated for new API

## Affected Files

| File | Action | Est. Lines |
|------|--------|------------|
| `Cargo.toml` (workspace) | Add fc-drawing | ~1 |
| `fc-drawing/Cargo.toml` | Create | ~20 |
| `fc-drawing/src/lib.rs` | Create | ~15 |
| `fc-drawing/src/trait_def.rs` | Create | ~40 |
| `fc-drawing/src/hit.rs` | Create | ~50 |
| `fc-drawing/src/bounds.rs` | Create | ~60 |
| `fc-drawing/src/types/*.rs` | Move from fc-domain | ~1500 |
| `fc-drawing/src/render.rs` | Create | ~30 |
| `fc-domain/src/drawing/mod.rs` | Simplify to re-export | ~20 |
| `fc-domain/src/drawing/types.rs` | Delete (moved) | -1474 |
| `fc-render/src/drawing.rs` | Delete (moved) | -2596 |
| `fc-render/src/drawing_manager.rs` | Rewrite 3 methods | ~80 |
| `fc-render/src/drawing_interaction.rs` | Update imports | ~5 |
| Various test files | Update imports | ~50 |
| **Total new** | | **~1800** |
| **Total deleted** | | **~-4100** |
| **Net** | | **~-2300** |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Import migration misses callers | Medium | `cargo build 2>&1 \| grep "unresolved import"` finds all |
| Test regressions | Low | 1387 existing tests + 20+ drawing-specific |
| Serde breakage | Low | Serde derives on concrete types only, not trait |
| Circular dependency | Low | fc-drawing depends on fc-render for DrawCommand — verify no cycle |

## Success Criteria

- [ ] Single `Drawing` trait in `fc-drawing`
- [ ] DrawingSet: 6 generic methods, 0 typed methods
- [ ] DrawingManager: 3 single-loop methods (hit_test, bounds, render)
- [ ] Adding new type = 1 struct + 1 trait impl
- [ ] All 1387+ tests pass
- [ ] 0 clippy warnings
- [ ] `cargo build` succeeds
