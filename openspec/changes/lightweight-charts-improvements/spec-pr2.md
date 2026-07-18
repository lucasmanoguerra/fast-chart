# Price Scales Specification

## Goal

Introduce named, per-pane price scales (Left, Right, Overlay) with auto-scaling, formatters, and per-series scale assignment, replacing the single `value_min/max` scalar in `Viewport`.

## Data Structures

```rust
// --- Price Scale Identity ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PriceScaleId {
    Left,
    Right,
    Overlay(String),
}

// --- Price Scale Mode ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceScaleMode {
    Normal,      // Linear scale (default)
    Logarithmic, // Log-scale (crypto, wide-range assets)
    Percentage,  // % change from first visible bar
}

// --- Price Scale Options ---

#[derive(Debug, Clone)]
pub struct PriceScaleOptions {
    pub visible: bool,
    pub auto_scale: bool,
    pub mode: PriceScaleMode,
    pub scale_offset: f64,  // extra padding fraction (0.0–0.1)
}

// --- Price Scale ---

#[derive(Debug, Clone)]
pub struct PriceScale {
    pub id: PriceScaleId,
    pub options: PriceScaleOptions,
    pub value_min: f64,
    pub value_max: f64,
}

// --- Price Formatter Trait ---

pub trait PriceFormatter: Send + Sync {
    fn format(&self, price: f64) -> String;
    fn format_short(&self, price: f64) -> String;
}

// --- Default Formatter ---

pub struct DefaultPriceFormatter {
    pub decimal_places: Option<usize>, // None = auto-detect
}
```

## API Surface

```rust
// PriceScale
impl PriceScale {
    pub fn new(id: PriceScaleId, options: PriceScaleOptions) -> Self;
    pub fn auto_fit(&mut self, visible_data_min: f64, visible_data_max: f64); // applies scale_offset padding
    pub fn contains(&self, price: f64) -> bool;
}

// PriceScaleOptions
impl Default for PriceScaleOptions { /* visible: true, auto_scale: true, mode: Normal, scale_offset: 0.05 */ }

// DefaultPriceFormatter
impl PriceFormatter for DefaultPriceFormatter {
    fn format(&self, price: f64) -> String;   // e.g. "105.20"
    fn format_short(&self, price: f64) -> String; // e.g. "105.2"
}

// Viewport additions
impl Viewport {
    pub fn price_to_y(&self, price: f64, scale: &PriceScale, pane_height: f64) -> f32;
    pub fn y_to_price(&self, y: f32, scale: &PriceScale, pane_height: f64) -> f64;
}

// Pane addition
impl Pane {
    pub fn price_scale(&self, id: &PriceScaleId) -> Option<&PriceScale>;
    pub fn price_scale_mut(&mut self, id: &PriceScaleId) -> Option<&mut PriceScale>;
    pub fn ensure_price_scales(&mut self); // guarantees Left + Right exist
}
```

## Requirements

### Requirement: PriceScaleId identity

`PriceScaleId` MUST distinguish Left, Right, and named Overlay scales. Overlay identifiers MUST be unique within a pane. `PartialEq` + `Hash` MUST be derived for use as map keys.

#### Scenario: Left and Right are distinct

- GIVEN `PriceScaleId::Left` and `PriceScaleId::Right`
- WHEN compared
- THEN they are not equal

#### Scenario: Overlay uniqueness by name

- GIVEN `PriceScaleId::Overlay("RSI".into())` in a pane
- WHEN a second `Overlay("RSI".into())` is added
- THEN the pane rejects the duplicate (returns existing reference)

### Requirement: PriceScale holds min/max

Each `PriceScale` MUST maintain its own `value_min` and `value_max`. These replace the scalar fields on `Viewport`. The `Viewport` struct MUST retain its existing `value_min`/`value_max` fields for backward compatibility during migration but they become the default Left scale's values.

#### Scenario: Independent ranges per scale

- GIVEN a pane with Left and Right price scales
- WHEN Left has range 100–200 and Right has range 50–100
- THEN each scale maps prices independently within its own range

### Requirement: Auto-scale computation

`auto_fit(visible_data_min, visible_data_max)` MUST set `value_min` and `value_max` to the data range plus `scale_offset` padding. When `auto_scale` is false, `auto_fit` MUST be a no-op.

#### Scenario: Auto-fit with 5% padding

- GIVEN a PriceScale with `scale_offset: 0.05` and data range 100–200
- WHEN `auto_fit(100.0, 200.0)` is called
- THEN `value_min` is 95.0 and `value_max` is 205.0

#### Scenario: Auto-scale disabled

- GIVEN a PriceScale with `auto_scale: false`
- WHEN `auto_fit(100.0, 200.0)` is called
- THEN `value_min` and `value_max` are unchanged

### Requirement: PriceFormatter trait

`PriceFormatter` MUST be a trait object-safe method pair: `format(price) -> String` and `format_short(price) -> String`. `format` is for axis labels (full precision), `format_short` is for crosshair tooltips (compact).

#### Scenario: DefaultPriceFormatter auto-detects precision

- GIVEN `DefaultPriceFormatter { decimal_places: None }`
- WHEN formatting 105.2
- THEN output is "105.20" (2 decimals for prices >= 1.0)

#### Scenario: DefaultPriceFormatter for sub-1 prices

- GIVEN `DefaultPriceFormatter { decimal_places: None }`
- WHEN formatting 0.00523
- THEN output is "0.00523" (5 decimals for prices < 0.01)

#### Scenario: Explicit decimal places

- GIVEN `DefaultPriceFormatter { decimal_places: Some(4) }`
- WHEN formatting 105.2
- THEN output is "105.2000"

### Requirement: Pane holds multiple price scales

Each `Pane` MUST hold `price_scales: Vec<PriceScale>` initialized with Left and Right by default. `ensure_price_scales()` MUST guarantee at least these two exist.

#### Scenario: New pane gets Left + Right

- GIVEN `Pane::new(0, 0.7)`
- WHEN `ensure_price_scales()` is called
- THEN `price_scales.len() == 2` with Left at index 0, Right at index 1

#### Scenario: Overlay scale added

- GIVEN a pane with Left and Right
- WHEN `price_scales.push(PriceScale::new(Overlay("RSI".into()), ..))`
- THEN `price_scales.len() == 3` and lookup by `Overlay("RSI")` succeeds

### Requirement: Series targets a price scale

`SeriesRef` MUST include a `price_scale_id: PriceScaleId` field. Default is `PriceScaleId::Left`. The renderer MUST use the series' target scale for coordinate mapping.

#### Scenario: Series on Right scale

- GIVEN a series with `price_scale_id: PriceScaleId::Right`
- WHEN the renderer maps prices to y-coordinates
- THEN it uses the Right scale's min/max, not Left

#### Scenario: Default series uses Left

- GIVEN a `SeriesRef` constructed without explicit `price_scale_id`
- WHEN the price_scale_id is read
- THEN it defaults to `PriceScaleId::Left`

### Requirement: Coordinate mapping through PriceScale

`Viewport::price_to_y(price, scale, pane_height) -> f32` MUST map a price to a pixel y-coordinate using the given scale's `value_min/value_max` and the pane's height. `y_to_price(y, scale, pane_height) -> f64` MUST be the inverse.

#### Scenario: price_to_y midpoint

- GIVEN scale min=100, max=200, pane_height=400
- WHEN `price_to_y(150.0, ..)` is called
- THEN result is 200.0 (midpoint, y-flipped: 0 = top)

#### Scenario: y_to_price roundtrip

- GIVEN scale min=100, max=200, pane_height=400
- WHEN `y_to_price(price_to_y(150.0, ..), ..)` is called
- THEN result is approximately 150.0 (within f64 precision)

#### Scenario: Zero range returns midpoint

- GIVEN scale min=100, max=100, pane_height=400
- WHEN `price_to_y(100.0, ..)` is called
- THEN result is 200.0 (pane_height / 2)

### Requirement: Unit tests for price mapping

The price mapping functions MUST have unit tests covering: midpoint, boundaries, zero-range, roundtrip, and multi-scale independence. Minimum 6 test cases.

#### Scenario: All tests pass

- GIVEN the price scale module
- WHEN `cargo test` runs
- THEN all price mapping tests pass

## Edge Cases

| Case | Expected Behavior |
|------|-------------------|
| `auto_fit` with empty data range (min == max) | Set both to the single value ± 1.0 minimum range |
| `Overlay("")` empty string | Rejected — Overlay names MUST be non-empty |
| Price outside scale range in `price_to_y` | Clamps to 0.0 (top) or pane_height (bottom), no panic |
| `format` with NaN | Returns "NaN" string |
| `format` with infinity | Returns "∞" or "-∞" string |
| Multiple panes with same Overlay name | Allowed — Overlay names are unique per pane, not globally |

## Testing Strategy

- Unit tests in `fc-types/src/price_scale.rs` for `PriceScale`, `PriceScaleOptions`, `DefaultPriceFormatter`
- Unit tests in `fc-types/src/viewport.rs` for `price_to_y` / `y_to_price`
- Property tests: roundtrip `y_to_price(price_to_y(p)) ≈ p` for random prices within scale range
- Integration: verify `GpuRenderer` uses per-series scale for axis labels

## Files Affected

| File | Change |
|------|--------|
| `fc-types/src/price_scale.rs` | **New** — `PriceScaleId`, `PriceScaleMode`, `PriceScaleOptions`, `PriceScale`, `PriceFormatter`, `DefaultPriceFormatter` + tests |
| `fc-types/src/viewport.rs` | Add `price_to_y()`, `y_to_price()` methods that accept a `PriceScale` reference |
| `fc-types/src/scale.rs` | No structural changes — `LinearScale` remains for internal use; `PriceScale` is the public API |
| `fast-chart-core/src/app/pane.rs` | Add `price_scales: Vec<PriceScale>`, `ensure_price_scales()`, lookup methods |
| `fast-chart-core/src/app/pane.rs` | `SeriesRef` gains `price_scale_id: PriceScaleId` field with Left default |
| `fast-chart-app/src/adapters/gpu_renderer.rs` | Use `PriceScale` for coordinate mapping in `update_line_from_vec`, `screen_y_to_price`, and axis label generation |
