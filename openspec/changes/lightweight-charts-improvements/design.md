# Design: InvalidationMask + Price Scales

## Technical Approach

Two sequential foundation PRs that establish reusable primitives for all future chart improvements. PR1 (InvalidationMask) replaces 5 scattered booleans with a single typed bitmask, enabling per-pane selective re-render. PR2 (Price Scales) introduces named, per-pane coordinate systems that replace the flat `value_min/max` on `Viewport`, enabling multi-axis charts (e.g., price + RSI overlay).

Both changes are backward-compatible: existing call sites continue to work via deprecation, and no downstream breakage occurs within the PRs themselves.

## Architecture Decisions

### Decision: InvalidationMask lives in `fast-chart-domain`

| Option | Tradeoff | Decision |
|--------|----------|----------|
| `domain` (pure types) | Zero deps, testable without GPU, but domain crate must stay zero-dep | **Chosen** |
| `core` (chart layer) | Closer to consumer, but harder to test in isolation and couples core to rendering concepts | Rejected |
| `app` (adapter layer) | Too low-level — would force re-imports across layers | Rejected |

**Rationale**: InvalidationMask is a pure value type with no I/O. Domain is the right home for "what needs invalidating" — the renderer decides "how to invalidate." The domain crate has zero external deps, so we implement the bitmask manually with `u32` operations instead of adding `bitflags`.

### Decision: Manual u32 bitmask, not the `bitflags` crate

| Option | Tradeoff | Decision |
|--------|----------|----------|
| `bitflags` crate | Ergonomic, battle-tested, but adds a dependency to the zero-dep domain crate | Rejected |
| Manual `u32` operations | 5 lines of code, zero deps, trivially correct | **Chosen** |
| `enumset` crate | Type-safe, but heavier dependency for a simple bitmask | Rejected |

**Rationale**: The domain crate's zero-dependency constraint is an explicit architectural boundary. The bitmask is 32 bits of manual `|` and `&` — not worth a dependency. We wrap it in a newtype for safety.

### Decision: PriceScale lives in `fast-chart-domain`

| Option | Tradeoff | Decision |
|--------|----------|----------|
| `domain` | Coordinate math is pure computation, testable without GPU | **Chosen** |
| `core` | Closer to Pane, but couples scale to pane lifecycle | Rejected |

**Rationale**: PriceScale is a domain concept — "how prices map to pixels." It has no rendering dependencies. The formatter trait is pure string formatting. Keeping it in domain allows unit testing coordinate math without wgpu.

### Decision: Formatter as trait object, not enum dispatch

| Option | Tradeoff | Decision |
|--------|----------|----------|
| `trait PriceFormatter` | Extensible, users add custom formatters, but requires `Box<dyn>` | **Chosen** |
| `enum Formatter { Default, Percent, Bitcoin }` | Simpler, no heap alloc, but closed for extension | Rejected |

**Rationale**: Trading apps have domain-specific formatting (futures tick sizes, crypto decimals, percentage modes). A trait allows users to implement custom formatters without modifying the library. The heap cost of `Box<dyn>` is negligible compared to the rendering overhead.

### Decision: Pane holds `Vec<PriceScale>`, not `HashMap`

| Option | Tradeoff | Decision |
|--------|----------|----------|
| `Vec<PriceScale>` with linear lookup | Small N (2-4 scales typically), cache-friendly, no hasher overhead | **Chosen** |
| `HashMap<PriceScaleId, PriceScale>` | O(1) lookup, but hasher overhead for 2-4 entries is wasteful | Rejected |
| `ArrayVec<PriceScale, 4>` | Fixed capacity, zero alloc, but limits to 4 scales | Rejected for now |

**Rationale**: Typical panes have 2-3 scales (Left, Right, maybe one Overlay). Linear scan over 3 elements is faster than HashMap hashing. If scale count grows, we can migrate to ArrayVec later.

## Data Structures

### PR1: InvalidationMask

```rust
// fast-chart-domain/src/invalidation.rs

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InvalidationLevel {
    Nothing  = 0,
    Cursor   = 1,
    Light    = 2,
    Full     = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaneBitmask(u32);

impl PaneBitmask {
    pub const EMPTY: Self = Self(0);
    pub const ALL: Self = Self(0xFFFF_FFFF);

    pub fn single(pane_index: u32) -> Self {
        assert!(pane_index < 32, "pane_index must be < 32");
        Self(1 << pane_index)
    }

    pub fn contains(self, pane: u32) -> bool {
        (self.0 & (1 << pane)) != 0
    }

    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitOr for PaneBitmask {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self { self.union(rhs) }
}

#[derive(Debug, Clone, Copy)]
pub struct InvalidationMask {
    level: InvalidationLevel,
    panes: PaneBitmask,
}
```

### PR2: PriceScale types

```rust
// fast-chart-domain/src/price_scale.rs

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PriceScaleId {
    Left,
    Right,
    Overlay(String),
}

impl PriceScaleId {
    pub fn is_overlay(name: &str) -> Self {
        Self::Overlay(name.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceScaleMode {
    Normal,
    Logarithmic,
    Percentage,
}

#[derive(Debug, Clone)]
pub struct PriceScaleOptions {
    pub visible: bool,
    pub auto_scale: bool,
    pub mode: PriceScaleMode,
    pub scale_offset: f64,
}

impl Default for PriceScaleOptions {
    fn default() -> Self {
        Self {
            visible: true,
            auto_scale: true,
            mode: PriceScaleMode::Normal,
            scale_offset: 0.05,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PriceScale {
    pub id: PriceScaleId,
    pub options: PriceScaleOptions,
    pub value_min: f64,
    pub value_max: f64,
}

pub trait PriceFormatter: Send + Sync {
    fn format(&self, price: f64) -> String;
    fn format_short(&self, price: f64) -> String;
}

pub struct DefaultPriceFormatter {
    pub decimal_places: Option<usize>,
}
```

## API Design

### InvalidationMask API

```rust
impl InvalidationMask {
    pub fn new() -> Self {
        Self { level: InvalidationLevel::Nothing, panes: PaneBitmask::EMPTY }
    }

    /// Union: take max level, OR pane bits.
    pub fn mark(&mut self, level: InvalidationLevel, panes: PaneBitmask) {
        if level > self.level { self.level = level; }
        self.panes = self.panes | panes;
    }

    /// true if stored level >= threshold AND pane bit is set.
    pub fn contains(&self, level: InvalidationLevel, pane: u32) -> bool {
        self.level >= level && self.panes.contains(pane)
    }

    pub fn clear(&mut self) { *self = Self::new(); }
    pub fn level(&self) -> InvalidationLevel { self.level }
    pub fn panes(&self) -> PaneBitmask { self.panes }
}
```

### PriceScale API

```rust
impl PriceScale {
    pub fn new(id: PriceScaleId, options: PriceScaleOptions) -> Self {
        Self { id, options, value_min: 0.0, value_max: 100.0 }
    }

    /// Apply auto-fit with scale_offset padding. No-op if auto_scale is false.
    pub fn auto_fit(&mut self, visible_data_min: f64, visible_data_max: f64) {
        if !self.options.auto_scale { return; }
        let range = visible_data_max - visible_data_min;
        if range.abs() < f64::EPSILON {
            self.value_min = visible_data_min - 1.0;
            self.value_max = visible_data_max + 1.0;
            return;
        }
        let pad = range * self.options.scale_offset;
        self.value_min = visible_data_min - pad;
        self.value_max = visible_data_max + pad;
    }

    pub fn contains(&self, price: f64) -> bool {
        price >= self.value_min && price <= self.value_max
    }
}
```

### Viewport integration

```rust
// Added to fast-chart-domain/src/viewport.rs

impl Viewport {
    pub fn price_to_y(&self, price: f64, scale: &PriceScale, pane_height: f32) -> f32 {
        let range = scale.value_max - scale.value_min;
        if range.abs() < f64::EPSILON {
            return pane_height / 2.0;
        }
        let ratio = (price - scale.value_min) / range;
        let clamped = ratio.clamp(0.0, 1.0);
        pane_height * (1.0 - clamped as f32)  // y-flipped: top=0
    }

    pub fn y_to_price(&self, y: f32, scale: &PriceScale, pane_height: f32) -> f64 {
        if pane_height.abs() < f32::EPSILON {
            return (scale.value_min + scale.value_max) / 2.0;
        }
        let ratio = 1.0 - (y / pane_height);
        let clamped = ratio.clamp(0.0, 1.0);
        scale.value_min + clamped as f64 * (scale.value_max - scale.value_min)
    }
}
```

### Pane integration

```rust
// Changes to fast-chart-core/src/app/pane.rs

use fast_chart_domain::price_scale::{PriceScale, PriceScaleId, PriceScaleOptions};

pub struct Pane {
    // ... existing fields ...
    pub price_scales: Vec<PriceScale>,
    pub primary_scale_id: PriceScaleId,
}

impl Pane {
    pub fn ensure_price_scales(&mut self) {
        if self.price_scales.is_empty() {
            self.price_scales.push(PriceScale::new(
                PriceScaleId::Left, PriceScaleOptions::default()
            ));
            self.price_scales.push(PriceScale::new(
                PriceScaleId::Right, PriceScaleOptions::default()
            ));
        }
    }

    pub fn price_scale(&self, id: &PriceScaleId) -> Option<&PriceScale> {
        self.price_scales.iter().find(|s| &s.id == id)
    }

    pub fn price_scale_mut(&mut self, id: &PriceScaleId) -> Option<&mut PriceScale> {
        self.price_scales.iter_mut().find(|s| &s.id == id)
    }

    pub fn primary_scale(&self) -> &PriceScale {
        self.price_scale(&self.primary_scale_id)
            .expect("primary scale always exists after ensure_price_scales()")
    }
}

// SeriesRef gains price_scale_id:
pub struct SeriesRef {
    pub name: String,
    pub series_type: SeriesType,
    pub price_scale_id: PriceScaleId,  // default: Left
}

impl Default for SeriesRef {
    fn default() -> Self {
        Self {
            name: String::new(),
            series_type: SeriesType::default(),
            price_scale_id: PriceScaleId::Left,
        }
    }
}
```

## Integration Points

### PR1: GpuRenderer boolean replacement

Current state in `gpu_renderer.rs`:
```rust
needs_line_update: bool,
needs_candle_update: bool,
needs_divider_update: bool,
needs_text_update: bool,
```

Replacement:
```rust
invalidation: InvalidationMask,
```

Mapping of existing call sites:
| Current code | New code |
|---|---|
| `self.needs_line_update = true` | `self.invalidation.mark(Full, PaneBitmask::ALL)` |
| `self.needs_candle_update = true` | `self.invalidation.mark(Full, PaneBitmask::ALL)` |
| `self.needs_divider_update = true` | `self.invalidation.mark(Full, PaneBitmask::ALL)` |
| `self.needs_text_update = true` | `self.invalidation.mark(Light, PaneBitmask::ALL)` |
| `self.set_crosshair(x, y, active)` | `self.invalidation.mark(Cursor, PaneBitmask::ALL)` |
| `if self.needs_line_update` | `if self.invalidation.contains(Full, pane_index)` |

**Migration strategy**: Phase 1 adds InvalidationMask alongside existing booleans. Phase 2 migrates call sites one method at a time. Phase 3 removes booleans. Each phase compiles and passes tests.

### PR1: ChartState migration

```rust
// Before:
pub needs_redraw: bool,

// After:
pub invalidation: InvalidationMask,
```

Tick loop becomes:
```rust
if self.state.invalidation.level() > InvalidationLevel::Nothing {
    let _ = self.renderer.render(&self.state);
    self.state.invalidation.clear();
}
```

### PR2: GpuRenderer coordinate mapping

The renderer currently hardcodes `self.viewport.value_min/max` for coordinate math. After PR2:

1. Each `Pane` owns its `PriceScale` instances
2. `GpuRenderer::update_line_from_vec()` receives `(viewport, scale, pane_height)` and delegates to `Viewport::price_to_y()`
3. `screen_y_to_price()` uses `Viewport::y_to_price()` with the target series' scale
4. Axis labels call `PriceFormatter::format()` instead of the standalone `format_price()`

### PR2: Viewport backward compatibility

`Viewport.value_min/value_max` is retained during migration. During the transition:
- `price_to_y()` accepts a `PriceScale` reference (new path)
- Old code that reads `viewport.value_min/max` still works
- After full migration, `value_min/max` can be deprecated or removed

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `fast-chart-domain/src/invalidation.rs` | **Create** | `InvalidationLevel`, `PaneBitmask`, `InvalidationMask` + 10 unit tests |
| `fast-chart-domain/src/price_scale.rs` | **Create** | `PriceScaleId`, `PriceScaleMode`, `PriceScaleOptions`, `PriceScale`, `PriceFormatter`, `DefaultPriceFormatter` + tests |
| `fast-chart-domain/src/lib.rs` | **Modify** | Add `pub mod invalidation; pub mod price_scale;` |
| `fast-chart-domain/src/viewport.rs` | **Modify** | Add `price_to_y()`, `y_to_price()` methods accepting `PriceScale` |
| `fast-chart-core/src/app/pane.rs` | **Modify** | Add `price_scales: Vec<PriceScale>`, `primary_scale_id`, `ensure_price_scales()`, lookup methods; add `price_scale_id` to `SeriesRef` |
| `fast-chart-core/src/app/chart_controller.rs` | **Modify** | Replace `needs_redraw: bool` with `InvalidationMask` in `ChartState`; update `tick()` and `handle_input()` |
| `fast-chart-app/src/adapters/gpu_renderer.rs` | **Modify** | Replace 4 `needs_*` bools with `InvalidationMask`; update `render()` dispatch; update coordinate mapping to use `PriceScale` |

## Testing Strategy

### PR1 Unit Tests (10+ cases)

| Test | What it verifies |
|------|-----------------|
| `default_is_nothing` | New mask = Nothing + empty panes |
| `mark_union_level_wins` | Full after Cursor → Full |
| `mark_no_downgrade` | Cursor after Full → Full |
| `mark_union_panes_accumulate` | PANE_0 then PANE_1 → both set |
| `contains_level_hierarchy` | Full satisfies Light query |
| `contains_wrong_level` | Light does not satisfy Full |
| `contains_wrong_pane` | PANE_0 mask → PANE_1 is false |
| `clear_resets` | Clear → Nothing + empty |
| `nothing_mark_is_noop` | mark(Nothing, PANE_0) → no change |
| `all_panes_covers_32` | ALL_PANES covers index 0..31 |

### PR2 Unit Tests (8+ cases)

| Test | What it verifies |
|------|-----------------|
| `price_to_y_midpoint` | 150 in 100-200 range → center |
| `price_to_y_top` | max price → y = 0 |
| `price_to_y_bottom` | min price → y = pane_height |
| `y_to_price_roundtrip` | `y_to_price(price_to_y(p)) ≈ p` |
| `price_to_y_zero_range` | min == max → pane_height / 2 |
| `price_to_y_clamps` | Price outside range clamps to 0 or height |
| `auto_fit_padds_range` | 100-200 with 0.05 offset → 95-205 |
| `auto_fit_disabled_noop` | auto_scale=false → unchanged |
| `default_format_detects_precision` | 105.2 → "105.20", 0.00523 → "0.00523" |
| `explicit_format` | decimal_places=Some(4) → "105.2000" |

### Integration Tests

- `GpuRenderer::render()` dispatches sub-renderers based on InvalidationMask contents
- `ChartState::tick()` clears mask after render
- Pane lookup by `PriceScaleId` returns correct scale
- `SeriesRef` defaults to Left scale

## Migration Plan

### PR1: InvalidationMask (3 commits)

1. **Commit 1**: Add `fast-chart-domain/src/invalidation.rs` with types + unit tests. Update `lib.rs`. Zero call-site changes — purely additive.
2. **Commit 2**: Add `InvalidationMask` field to `ChartState` and `GpuRenderer` alongside existing booleans. Wire up `mark()` calls next to existing boolean sets. Both systems coexist.
3. **Commit 3**: Remove 4 `needs_*` booleans from `GpuRenderer` and `needs_redraw` from `ChartState`. Replace all check sites with `invalidation.contains()`. Remove boolean-setting code.

### PR2: Price Scales (4 commits)

1. **Commit 1**: Add `fast-chart-domain/src/price_scale.rs` with types + `PriceFormatter` + unit tests. Update `lib.rs`. Zero call-site changes.
2. **Commit 2**: Add `price_to_y()`/`y_to_price()` to `Viewport` with unit tests. Add `price_scales` field to `Pane` with `ensure_price_scales()`. Update `SeriesRef` with `price_scale_id` default.
3. **Commit 3**: Wire `GpuRenderer` to use `PriceScale` for coordinate mapping in `update_line_from_vec()`, `screen_y_to_price()`, and axis labels. Replace standalone `format_price()` with `PriceFormatter`.
4. **Commit 4**: Add `auto_fit()` integration — call `PriceScale::auto_fit()` in the render path when data changes. Deprecate direct `Viewport.value_min/max` writes for price data.

## Threat Matrix

N/A — no routing, shell, subprocess, VCS/PR automation, executable-file classification, or process-integration boundary.

## Open Questions

- [ ] Should `PaneBitmask` panic on index >= 32 or use a runtime error type? Current design panics in debug only.
- [ ] When removing `Viewport.value_min/max` eventually, should we add a `#[deprecated]` attribute in PR2 or leave it for PR3?
- [ ] For the formatter trait: should `format()` return `Cow<'_, str>` to avoid allocation on the hot path, or is `String` acceptable given axis labels are infrequent?
