# Drawing Tools System — Exploration Report

## Current State

### What Exists

The project has a **two-layer drawing system** already in place:

#### Layer 1: Domain Types (`fc-domain/src/drawing/`)
- **Drawing trait** (minimal): `id()`, `move_by()`, `as_any()`, `as_any_mut()`
- **15 concrete types**: TrendLine, Arrow, Ray, Segment, TextDrawing, ImageDrawing, LabelDrawing, HorizontalLine, VerticalLine, Rectangle, FibonacciRetracement, FibonacciExtension, Pitchfork, Ellipse, Path
- **DrawingSet**: `Vec<Box<dyn Drawing>>` with typed add/get/all methods per type
- All types have `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]`
- Builder pattern: `.with_color()`, `.with_width()`, `.with_style()`

#### Layer 2: Render-Side (`fc-render/src/`)
- **Drawing trait** (render-aware): `id()`, `hit_test()`, `move_by()`, `bounds()`, `is_selected()`, `set_selected()`, `to_commands()`
- **HitResult** enum: `Miss`, `Body`, `ControlPoint(usize)` — but `ControlPoint` is NEVER returned by any type
- **DrawingBounds**: bounding box with `combine()`, `contains()`, `from_points()`
- **DrawingManager**: wraps DrawingSet, provides unified hit_test/render/bounds/selection
- **DrawingInteraction**: state machine with `DrawingMode` enum (14 modes), click-count tracking, drag support

#### Render Pipeline Integration
- Each type implements `to_commands(&RenderContext) -> Vec<DrawCommand>`
- DrawCommand variants used: `DrawLine`, `DrawRect`, `DrawCircle`, `DrawTriangle`, `DrawPath`, `DrawText`, `DrawImage`
- Z-index ordering: fills at 5-6, strokes at 8-10, selection indicators at 25

### What's Working
- All 15 types can be created, stored, moved, hit-tested, and rendered
- Selection management (single select/deselect)
- DrawingInteraction state machine handles click sequences for all modes
- Drag-to-place for Rectangle and Ellipse
- Serde support (behind feature flag)

---

## Gap Analysis

### Critical Gaps (must-have for professional quality)

| Gap | Description | Impact |
|-----|-------------|--------|
| **Control Point Handles** | `HitResult::ControlPoint(usize)` exists but no type returns it. No resize/reshape logic. | Users can't resize or reshape drawings |
| **Snap to Price/Time** | No magnetic snapping to candle OHLC, nearest price, or time axis | Drawing placement feels imprecise |
| **Layer Z-Ordering** | Drawings have no explicit z-order; they render in type-iteration order | Can't reorder drawings |
| **Undo/Redo** | No command history or state snapshots | Users can't undo mistakes |
| **Multi-Select** | Only single selection supported | Can't move/delete groups |
| **Drawing Toolbar UI** | DrawingMode enum exists but no UI toolbar component | No way to select tools |
| **Persistence/Load** | Serde derives exist but no save/load API | Drawings lost on reload |
| **Text Editing** | TextDrawing has fixed text; no inline editing | Can't create/edit text annotations |
| **Keyboard Shortcuts** | No hotkey system for tool selection or drawing operations | Power users can't work fast |
| **Drawing Properties Panel** | No UI to edit color/width/style of selected drawing | Limited customization |

### Important Gaps (expected in v1)

| Gap | Description |
|-----|-------------|
| **Crosshair Integration** | Drawing preview should show at crosshair position during placement |
| **Drawing Labels** | Price/time labels on drawing endpoints (like TradingView) |
| **Magnet Mode** | Snap to nearest candle OHLC for precise anchor points |
| **Drawing Templates** | Save/load drawing configurations |
| **Copy/Paste Drawings** | Duplicate drawings |
| **Drawing Groups** | Group related drawings together |

### Nice-to-Have (future)

- Drawing alerts (notify when price reaches a drawing level)
- Drawing sharing/collaboration
- Animated drawing transitions
- Drawing templates library (harmonic patterns, etc.)

---

## Architectural Issues

### Issue 1: Two Parallel Drawing Traits
The domain Drawing trait (`fc-domain`) and render Drawing trait (`fc-render`) are separate and have different method signatures. The domain trait is used by DrawingSet, while the render trait is used by DrawingManager.

**This is an OCP/SRP violation**: adding a new drawing type requires implementing TWO traits in TWO different crates.

### Issue 2: DrawingManager Code Duplication
`DrawingManager::hit_test()`, `bounds()`, and `render()` each have 15 nearly-identical loops iterating type-specific collections. Adding a new type requires modifying 3 methods in DrawingManager.

### Issue 3: DrawingSet Typed Methods
DrawingSet has 15 `add_X()`, 15 `get_X()`, and 15 `all_X()` methods (45 methods total). This defeats the purpose of `Vec<Box<dyn Drawing>>`.

---

## Approach Options

### Approach A: Unify Traits + Erase Types (Recommended)

**Core idea**: Single Drawing trait across the whole stack. DrawingSet stores only `Vec<Box<dyn Drawing>>` without typed accessors.

**Changes**:
1. Merge the two Drawing traits into one super-trait with all methods
2. Remove all typed `add_X/get_X/all_X` from DrawingSet — use generic `add()` and `iter()`
3. DrawingManager iterates `self.drawings.iter()` instead of 15 type-specific loops
4. Each new drawing type implements ONE trait in ONE place

**Pros**: OCP-compliant, minimal code duplication, easy to extend
**Cons**: Breaking change to existing API, larger trait object surface

### Approach B: Keep Two Layers, Add Bridge

**Core idea**: Keep domain and render traits separate but add a bridge/converter.

**Changes**:
1. Add `fn as_render(&self) -> &dyn RenderDrawing` to domain Drawing trait
2. DrawingManager uses bridge to access render methods
3. DrawingSet remains domain-only

**Pros**: Less breaking change, clearer separation of concerns
**Cons**: Still two traits to implement, bridge adds indirection

### Approach C: ECS-Inspired (Future-Proof)

**Core idea**: Treat drawings as components with separate systems for rendering, interaction, persistence.

**Changes**:
1. Drawing data is pure data (no trait methods)
2. Separate HitTestSystem, RenderSystem, InteractionSystem
3. Each system dispatches by type enum

**Pros**: Most scalable, easiest to add new systems
**Cons**: Biggest refactor, overkill for current scale

---

## Recommended Path

**Start with Approach A (Unify Traits)**, then iteratively add features:

### Phase 1: Trait Unification + Collection Cleanup
- Merge domain + render Drawing traits
- Clean up DrawingSet to use generic operations
- Fix DrawingManager to iterate once
- **Size: M** (~300-400 lines changed)

### Phase 2: Control Points + Snap
- Implement ControlPoint hit-testing for all types
- Add resize/reshape logic via control points
- Add price/time snapping to DrawingInteraction
- **Size: L** (~500-600 lines)

### Phase 3: Undo/Redo + Multi-Select
- Command pattern for drawing operations
- Multi-selection support
- Batch move/delete
- **Size: L** (~400-500 lines)

### Phase 4: Persistence + Toolbar
- Save/load API using existing serde derives
- Drawing toolbar component (tool selection UI)
- Properties panel for selected drawing
- **Size: XL** (~600-800 lines)

### Phase 5: Polish
- Keyboard shortcuts
- Drawing labels
- Magnet mode
- Crosshair preview during placement
- **Size: M** (~300-400 lines)

---

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Trait unification breaks existing callers | HIGH | Search all `use fc_domain::drawing::Drawing` and `use fc-render::drawing::Drawing` references; update imports |
| DrawingManager refactor introduces regressions | MEDIUM | Existing 20+ tests cover hit_test, render, bounds, selection |
| Control point logic is complex per-type | MEDIUM | Start with simple types (TrendLine: 2 endpoints), defer complex (Pitchfork: 3 points + median) |
| Serde compatibility with unified trait | LOW | Serde derives are on concrete types, not traits |

---

## Effort Summary

| Phase | T-Shirt | Lines Changed |
|-------|---------|---------------|
| Phase 1: Trait Unification | M | 300-400 |
| Phase 2: Control Points + Snap | L | 500-600 |
| Phase 3: Undo/Redo + Multi-Select | L | 400-500 |
| Phase 4: Persistence + Toolbar | XL | 600-800 |
| Phase 5: Polish | M | 300-400 |
| **Total** | **XL** | **2100-2700** |

---

## Key Files

| File | Role | Lines |
|------|------|-------|
| `fc-domain/src/drawing/mod.rs` | Drawing trait + DrawingSet | 436 |
| `fc-domain/src/drawing/types.rs` | 15 domain types | 1474 |
| `fc-domain/src/drawing/tests.rs` | Domain tests | 1762 |
| `fc-render/src/drawing.rs` | Render Drawing trait + impls for all 15 types | ~1450 |
| `fc-render/src/drawing_manager.rs` | DrawingManager (CRUD + render) | 507 |
| `fc-render/src/drawing_interaction.rs` | Interaction state machine | 311 |
