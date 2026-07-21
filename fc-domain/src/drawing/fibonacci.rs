use crate::price_line::LineStyle;

use super::{ChartPoint, Drawing, DrawingId};

// ---------------------------------------------------------------------------
// FibonacciRetracement
// ---------------------------------------------------------------------------

/// Default Fibonacci levels: 0%, 23.6%, 38.2%, 50%, 61.8%, 78.6%, 100%.
const DEFAULT_FIBONACCI_LEVELS: &[f64] = &[0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0];

/// Horizontal lines at Fibonacci retracement levels between two anchor points.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FibonacciRetracement {
    /// Unique identifier.
    pub id: DrawingId,
    /// First anchor point (e.g. swing low).
    pub start: ChartPoint,
    /// Second anchor point (e.g. swing high).
    pub end: ChartPoint,
    /// Fibonacci levels as fractions (e.g. 0.382 for 38.2%).
    pub levels: Vec<f64>,
    /// Line color [r, g, b, a].
    pub color: [f32; 4],
    /// Line width in pixels.
    pub width: f32,
    /// Line style.
    pub style: LineStyle,
    /// Whether this drawing is currently selected.
    pub selected: bool,
}

impl FibonacciRetracement {
    /// Create a new Fibonacci retracement with default levels and styling.
    pub fn new(id: impl Into<String>, start: ChartPoint, end: ChartPoint) -> Self {
        Self {
            id: DrawingId(id.into()),
            start,
            end,
            levels: DEFAULT_FIBONACCI_LEVELS.to_vec(),
            color: [0.5, 0.5, 0.5, 0.8],
            width: 1.0,
            style: LineStyle::Dashed,
            selected: false,
        }
    }

    /// Set the line color.
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Set the line width.
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Set the line style.
    pub fn with_style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }

    /// Set custom Fibonacci levels.
    pub fn with_levels(mut self, levels: Vec<f64>) -> Self {
        self.levels = levels;
        self
    }

    /// Calculate the price at a specific Fibonacci level.
    ///
    /// The level is a fraction from 0.0 to 1.0 (e.g. 0.382 for 38.2%).
    /// Interpolates linearly between `start.price` and `end.price`.
    pub fn price_at_level(&self, level: f64) -> f64 {
        let range = self.end.price - self.start.price;
        self.start.price + range * level
    }

    /// Get all level prices as `(level_fraction, price)` pairs.
    pub fn level_prices(&self) -> Vec<(f64, f64)> {
        self.levels
            .iter()
            .map(|&level| (level, self.price_at_level(level)))
            .collect()
    }
}

impl Drawing for FibonacciRetracement {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.start.timestamp = self.start.timestamp.saturating_add(delta.timestamp);
        self.start.price += delta.price;
        self.end.timestamp = self.end.timestamp.saturating_add(delta.timestamp);
        self.end.price += delta.price;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// FibonacciExtension
// ---------------------------------------------------------------------------

/// Default Fibonacci extension levels including extensions beyond 100%.
const DEFAULT_FIBONACCI_EXTENSION_LEVELS: &[f64] =
    &[0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0, 1.272, 1.618];

/// Fibonacci extension projecting levels beyond the current price using three
/// anchor points: A (start of move), B (end of move), C (retracement end).
///
/// The price at a given level is: `C.price + (B.price - A.price) * level`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FibonacciExtension {
    /// Unique identifier.
    pub id: DrawingId,
    /// Start of the move.
    pub point_a: ChartPoint,
    /// End of the move.
    pub point_b: ChartPoint,
    /// Retracement end.
    pub point_c: ChartPoint,
    /// Fibonacci levels as fractions (e.g. 0.618 for 61.8%, 1.618 for 161.8%).
    pub levels: Vec<f64>,
    /// Line color [r, g, b, a].
    pub color: [f32; 4],
    /// Line width in pixels.
    pub width: f32,
    /// Line style.
    pub style: LineStyle,
    /// Whether this drawing is currently selected.
    pub selected: bool,
}

impl FibonacciExtension {
    /// Create a new Fibonacci extension with default levels and styling.
    pub fn new(id: impl Into<String>, a: ChartPoint, b: ChartPoint, c: ChartPoint) -> Self {
        Self {
            id: DrawingId(id.into()),
            point_a: a,
            point_b: b,
            point_c: c,
            levels: DEFAULT_FIBONACCI_EXTENSION_LEVELS.to_vec(),
            color: [0.5, 0.5, 0.5, 0.8],
            width: 1.0,
            style: LineStyle::Dashed,
            selected: false,
        }
    }

    /// Set the line color.
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Set the line width.
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Set the line style.
    pub fn with_style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }

    /// Set custom Fibonacci levels.
    pub fn with_levels(mut self, levels: Vec<f64>) -> Self {
        self.levels = levels;
        self
    }

    /// Calculate the price at a specific Fibonacci extension level.
    ///
    /// Formula: `C.price + (B.price - A.price) * level`
    pub fn price_at_level(&self, level: f64) -> f64 {
        let ab_range = self.point_b.price - self.point_a.price;
        self.point_c.price + ab_range * level
    }

    /// Get all level prices as `(level_fraction, price)` pairs.
    pub fn level_prices(&self) -> Vec<(f64, f64)> {
        self.levels
            .iter()
            .map(|&level| (level, self.price_at_level(level)))
            .collect()
    }
}

impl Drawing for FibonacciExtension {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.point_a.timestamp = self.point_a.timestamp.saturating_add(delta.timestamp);
        self.point_a.price += delta.price;
        self.point_b.timestamp = self.point_b.timestamp.saturating_add(delta.timestamp);
        self.point_b.price += delta.price;
        self.point_c.timestamp = self.point_c.timestamp.saturating_add(delta.timestamp);
        self.point_c.price += delta.price;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// Pitchfork
// ---------------------------------------------------------------------------

/// Andrews' Pitchfork: a three-point drawing tool with a median line and
/// parallel upper/lower channel lines.
///
/// - **A** (handle/pivot): the anchor from which all lines project.
/// - **B**: first high or low.
/// - **C**: second high or low.
///
/// The median line passes through A and the midpoint of B and C.
/// The upper and lower lines are parallel to the median, offset by the
/// price distance from the midpoint to B and C respectively.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pitchfork {
    /// Unique identifier.
    pub id: DrawingId,
    /// Pivot point (handle).
    pub point_a: ChartPoint,
    /// First high or low.
    pub point_b: ChartPoint,
    /// Second high or low.
    pub point_c: ChartPoint,
    /// Line color [r, g, b, a].
    pub color: [f32; 4],
    /// Line width in pixels.
    pub width: f32,
    /// Line style.
    pub style: LineStyle,
    /// Whether this drawing is currently selected.
    pub selected: bool,
}

impl Pitchfork {
    /// Create a new pitchfork with default styling.
    pub fn new(id: impl Into<String>, a: ChartPoint, b: ChartPoint, c: ChartPoint) -> Self {
        Self {
            id: DrawingId(id.into()),
            point_a: a,
            point_b: b,
            point_c: c,
            color: [0.5, 0.5, 0.5, 0.8],
            width: 1.0,
            style: LineStyle::Solid,
            selected: false,
        }
    }

    /// Set the line color.
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Set the line width.
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Set the line style.
    pub fn with_style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }

    /// Price at the midpoint of B and C.
    fn midpoint_price(&self) -> f64 {
        (self.point_b.price + self.point_c.price) / 2.0
    }

    /// Timestamp span from A to the midpoint of B and C.
    fn midpoint_timestamp(&self) -> u64 {
        (self.point_b.timestamp + self.point_c.timestamp) / 2
    }

    /// Linear interpolation factor for a given timestamp relative to A and the
    /// midpoint of B and C.
    ///
    /// Returns `0.0` at A's timestamp and `1.0` at the midpoint's timestamp.
    /// When the denominator is zero (A and midpoint share the same timestamp),
    /// returns `0.0`.
    fn interpolation_factor(&self, timestamp: u64) -> f64 {
        let ts_a = self.point_a.timestamp;
        let ts_mid = self.midpoint_timestamp();
        let denominator = ts_mid.saturating_sub(ts_a);
        if denominator == 0 {
            return 0.0;
        }
        let t = timestamp.saturating_sub(ts_a);
        t as f64 / denominator as f64
    }

    /// Calculate the median line price at a given timestamp.
    ///
    /// The median line passes through A and the midpoint of B and C.
    /// Interpolates linearly along that line.
    pub fn median_price_at(&self, timestamp: u64) -> f64 {
        let t = self.interpolation_factor(timestamp);
        let mid = self.midpoint_price();
        self.point_a.price + (mid - self.point_a.price) * t
    }

    /// Calculate the upper channel price at a given timestamp.
    ///
    /// The upper line is parallel to the median, offset by the price distance
    /// from the midpoint to B (`B.price - midpoint_price`).
    pub fn upper_price_at(&self, timestamp: u64) -> f64 {
        let t = self.interpolation_factor(timestamp);
        let mid = self.midpoint_price();
        let offset = self.point_b.price - mid;
        let median = self.point_a.price + (mid - self.point_a.price) * t;
        median + offset
    }

    /// Calculate the lower channel price at a given timestamp.
    ///
    /// The lower line is parallel to the median, offset by the price distance
    /// from the midpoint to C (`C.price - midpoint_price`).
    pub fn lower_price_at(&self, timestamp: u64) -> f64 {
        let t = self.interpolation_factor(timestamp);
        let mid = self.midpoint_price();
        let offset = self.point_c.price - mid;
        let median = self.point_a.price + (mid - self.point_a.price) * t;
        median + offset
    }
}

impl Drawing for Pitchfork {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.point_a.timestamp = self.point_a.timestamp.saturating_add(delta.timestamp);
        self.point_a.price += delta.price;
        self.point_b.timestamp = self.point_b.timestamp.saturating_add(delta.timestamp);
        self.point_b.price += delta.price;
        self.point_c.timestamp = self.point_c.timestamp.saturating_add(delta.timestamp);
        self.point_c.price += delta.price;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
