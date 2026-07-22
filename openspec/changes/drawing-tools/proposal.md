# Proposal: Drawing Trait Unification + Collection Cleanup

## Intent

The drawing system has **two parallel `Drawing` traits** (fc-domain + fc-render), **45 typed methods** on DrawingSet, and **15-loop code duplication** in DrawingManager. This makes adding new drawing types painful (4+ files, 2 traits) and violates OCP/SRP. Phase 1 unifies everything into a single trait in a new `fc-drawing` crate and erases type-specific methods.

## Scope

### In Scope
- New `fc-drawing` crate with unified `Drawing` trait (all methods: id, move_by, as_any, hit_test, bounds, is_selected, set_selected, to_commands)
- Move all 15 domain types from `fc-domain/src/drawing/` into `fc-drawing`
- Eliminate all typed methods from DrawingSet — generic `add()` + `iter()` only
- Fix DrawingManager to iterate once over `Vec<Box<dyn Drawing>>`
- Default `hit_test()` impl based on `bounds()` with override per type
- Default `is_selected()` / `set_selected()` via `selected` field pattern
- Update all imports across workspace

### Out of Scope
- Control points / resize handles (Phase 2)
- Snap-to-price/time (Phase 2)
- Undo/redo (Phase 3)
- Toolbar UI (Phase 4)
- Serde persistence changes (stays behind feature flag)

## Capabilities

### New Capabilities
- `drawing-core`: Unified Drawing trait, DrawingSet with generic operations, HitResult, DrawingBounds — the complete drawing type system

### Modified Capabilities
- `domain-model`: Remove drawing types module (moved to fc-drawing)
- `gpu-renderer`: Update drawing render integration to use fc-drawing trait

## Approach

1. Create `fc-drawing` crate with unified trait + all 15 types
2. Add default `hit_test()` using AABB bounds check
3. Add default `is_selected()` / `set_selected()` via `selected: bool` field
4. Keep custom overrides for types needing precision (Ray, Path, Pitchfork)
5. DrawingSet: `add(&mut self, drawing: Box<dyn Drawing>)` + `iter()` + `remove()`
6. DrawingManager: single `for d in self.drawings.iter()` loop per operation
7. fc-domain re-exports fc-drawing for backward compatibility
8. Update fc-render imports

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `fc-drawing/` | **New** | New crate with unified trait + 15 types |
| `fc-domain/src/drawing/` | Removed | Types move to fc-drawing; mod.rs becomes re-export |
| `fc-render/src/drawing.rs` | Modified | Remove Drawing impls; use fc-drawing trait |
| `fc-render/src/drawing_manager.rs` | Modified | Single-loop iteration |
| `fc-render/src/drawing_interaction.rs` | Minor | Update imports |
| All Cargo.toml | Modified | Add fc-drawing dependency |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Breaking internal callers | Medium | grep all `use fc_domain::drawing` + `use fc_render::drawing`; update in same PR |
| Test regressions | Low | 1387 existing tests + 20+ drawing-specific tests provide safety net |
| Serde compatibility | Low | Serde derives stay on concrete types; trait unification doesn't affect serialization |

## Rollback Plan

- Revert the fc-drawing crate addition
- Restore fc-domain/src/drawing/ from git history
- Restore fc-render/src/drawing.rs from git history
- All tests should pass on the revert commit

## Success Criteria

- [ ] Single `Drawing` trait across the entire workspace
- [ ] DrawingSet has zero typed methods (add_trend_line, get_arrow, etc. all removed)
- [ ] DrawingManager hit_test/render/bounds each have ONE loop, not 15
- [ ] Adding a new drawing type requires: 1 struct + 1 trait impl + 1 DrawingMode variant
- [ ] All 1387+ tests pass
- [ ] 0 clippy warnings
