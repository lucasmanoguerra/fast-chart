use crate::price_line::LineStyle;

/// Unique identifier for a drawing tool.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DrawingId(pub String);

/// A point on the chart defined by timestamp and price.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChartPoint {
    pub timestamp: u64,
    pub price: f64,
}

impl ChartPoint {
    /// Create a new chart point.
    pub fn new(timestamp: u64, price: f64) -> Self {
        Self { timestamp, price }
    }
}

// ---------------------------------------------------------------------------
// TrendLine
// ---------------------------------------------------------------------------

/// A line segment connecting two points on the chart.
#[derive(Debug, Clone)]
pub struct TrendLine {
    /// Unique identifier.
    pub id: DrawingId,
    /// Start point.
    pub start: ChartPoint,
    /// End point.
    pub end: ChartPoint,
    /// Line color [r, g, b, a].
    pub color: [f32; 4],
    /// Line width in pixels.
    pub width: f32,
    /// Line style.
    pub style: LineStyle,
}

impl TrendLine {
    /// Create a new trend line between two points with default styling.
    pub fn new(id: impl Into<String>, start: ChartPoint, end: ChartPoint) -> Self {
        Self {
            id: DrawingId(id.into()),
            start,
            end,
            color: [1.0, 1.0, 1.0, 1.0],
            width: 1.0,
            style: LineStyle::Solid,
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
}

// ---------------------------------------------------------------------------
// HorizontalLine
// ---------------------------------------------------------------------------

/// A horizontal line at a specific price level spanning across the chart.
#[derive(Debug, Clone)]
pub struct HorizontalLine {
    /// Unique identifier.
    pub id: DrawingId,
    /// Price level.
    pub price: f64,
    /// Line color [r, g, b, a].
    pub color: [f32; 4],
    /// Line width in pixels.
    pub width: f32,
    /// Line style.
    pub style: LineStyle,
    /// Extend the line to the left edge of the chart.
    pub extend_left: bool,
    /// Extend the line to the right edge of the chart.
    pub extend_right: bool,
}

impl HorizontalLine {
    /// Create a new horizontal line at the given price with default styling.
    pub fn new(id: impl Into<String>, price: f64) -> Self {
        Self {
            id: DrawingId(id.into()),
            price,
            color: [0.5, 0.5, 0.5, 0.8],
            width: 1.0,
            style: LineStyle::Solid,
            extend_left: true,
            extend_right: true,
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

    /// Set whether the line extends to the left edge.
    pub fn with_extend_left(mut self, extend: bool) -> Self {
        self.extend_left = extend;
        self
    }

    /// Set whether the line extends to the right edge.
    pub fn with_extend_right(mut self, extend: bool) -> Self {
        self.extend_right = extend;
        self
    }
}

// ---------------------------------------------------------------------------
// VerticalLine
// ---------------------------------------------------------------------------

/// A vertical line at a specific timestamp spanning across the chart.
#[derive(Debug, Clone)]
pub struct VerticalLine {
    /// Unique identifier.
    pub id: DrawingId,
    /// Timestamp position.
    pub timestamp: u64,
    /// Line color [r, g, b, a].
    pub color: [f32; 4],
    /// Line width in pixels.
    pub width: f32,
    /// Line style.
    pub style: LineStyle,
}

impl VerticalLine {
    /// Create a new vertical line at the given timestamp with default styling.
    pub fn new(id: impl Into<String>, timestamp: u64) -> Self {
        Self {
            id: DrawingId(id.into()),
            timestamp,
            color: [0.5, 0.5, 0.5, 0.8],
            width: 1.0,
            style: LineStyle::Solid,
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
}

// ---------------------------------------------------------------------------
// Rectangle
// ---------------------------------------------------------------------------

/// A rectangle defined by two corner points.
#[derive(Debug, Clone)]
pub struct Rectangle {
    /// Unique identifier.
    pub id: DrawingId,
    /// Top-left corner of the rectangle.
    pub top_left: ChartPoint,
    /// Bottom-right corner of the rectangle.
    pub bottom_right: ChartPoint,
    /// Border color [r, g, b, a].
    pub color: [f32; 4],
    /// Border width in pixels.
    pub width: f32,
    /// Border line style.
    pub style: LineStyle,
    /// Optional fill color [r, g, b, a].
    pub fill_color: Option<[f32; 4]>,
}

impl Rectangle {
    /// Create a new rectangle with default styling and no fill.
    pub fn new(id: impl Into<String>, top_left: ChartPoint, bottom_right: ChartPoint) -> Self {
        Self {
            id: DrawingId(id.into()),
            top_left,
            bottom_right,
            color: [1.0, 1.0, 1.0, 1.0],
            width: 1.0,
            style: LineStyle::Solid,
            fill_color: None,
        }
    }

    /// Set the border color.
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Set the border width.
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Set the border line style.
    pub fn with_style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the fill color.
    pub fn with_fill(mut self, fill_color: [f32; 4]) -> Self {
        self.fill_color = Some(fill_color);
        self
    }

    /// Calculate width in timestamp units.
    pub fn width_ts(&self) -> u64 {
        let left = self.top_left.timestamp.min(self.bottom_right.timestamp);
        let right = self.top_left.timestamp.max(self.bottom_right.timestamp);
        right - left
    }

    /// Calculate height in price units.
    pub fn height_price(&self) -> f64 {
        let high = self.top_left.price.max(self.bottom_right.price);
        let low = self.top_left.price.min(self.bottom_right.price);
        high - low
    }
}

// ---------------------------------------------------------------------------
// FibonacciRetracement
// ---------------------------------------------------------------------------

/// Default Fibonacci levels: 0%, 23.6%, 38.2%, 50%, 61.8%, 78.6%, 100%.
const DEFAULT_FIBONACCI_LEVELS: &[f64] = &[0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0];

/// Horizontal lines at Fibonacci retracement levels between two anchor points.
#[derive(Debug, Clone)]
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

// ---------------------------------------------------------------------------
// DrawingSet
// ---------------------------------------------------------------------------

/// A collection of drawing tools for a chart pane.
#[derive(Debug, Default)]
pub struct DrawingSet {
    trend_lines: Vec<TrendLine>,
    horizontal_lines: Vec<HorizontalLine>,
    vertical_lines: Vec<VerticalLine>,
    rectangles: Vec<Rectangle>,
    fibonacci_retracements: Vec<FibonacciRetracement>,
    fibonacci_extensions: Vec<FibonacciExtension>,
    pitchforks: Vec<Pitchfork>,
}

impl DrawingSet {
    /// Create an empty drawing set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a trend line.
    pub fn add_trend_line(&mut self, line: TrendLine) {
        self.trend_lines.push(line);
    }

    /// Add a horizontal line.
    pub fn add_horizontal_line(&mut self, line: HorizontalLine) {
        self.horizontal_lines.push(line);
    }

    /// Add a vertical line.
    pub fn add_vertical_line(&mut self, line: VerticalLine) {
        self.vertical_lines.push(line);
    }

    /// Add a rectangle.
    pub fn add_rectangle(&mut self, rect: Rectangle) {
        self.rectangles.push(rect);
    }

    /// Add a Fibonacci retracement.
    pub fn add_fibonacci_retracement(&mut self, fib: FibonacciRetracement) {
        self.fibonacci_retracements.push(fib);
    }

    /// Add a Fibonacci extension.
    pub fn add_fibonacci_extension(&mut self, ext: FibonacciExtension) {
        self.fibonacci_extensions.push(ext);
    }

    /// Add a pitchfork.
    pub fn add_pitchfork(&mut self, pf: Pitchfork) {
        self.pitchforks.push(pf);
    }

    /// Remove a drawing by ID. Returns `true` if found and removed.
    pub fn remove(&mut self, id: &DrawingId) -> bool {
        if let Some(pos) = self.trend_lines.iter().position(|l| l.id == *id) {
            self.trend_lines.remove(pos);
            return true;
        }
        if let Some(pos) = self.horizontal_lines.iter().position(|l| l.id == *id) {
            self.horizontal_lines.remove(pos);
            return true;
        }
        if let Some(pos) = self.vertical_lines.iter().position(|l| l.id == *id) {
            self.vertical_lines.remove(pos);
            return true;
        }
        if let Some(pos) = self.rectangles.iter().position(|r| r.id == *id) {
            self.rectangles.remove(pos);
            return true;
        }
        if let Some(pos) = self.fibonacci_retracements.iter().position(|f| f.id == *id) {
            self.fibonacci_retracements.remove(pos);
            return true;
        }
        if let Some(pos) = self.fibonacci_extensions.iter().position(|e| e.id == *id) {
            self.fibonacci_extensions.remove(pos);
            return true;
        }
        if let Some(pos) = self.pitchforks.iter().position(|p| p.id == *id) {
            self.pitchforks.remove(pos);
            return true;
        }
        false
    }

    /// Get a trend line by ID.
    pub fn get_trend_line(&self, id: &DrawingId) -> Option<&TrendLine> {
        self.trend_lines.iter().find(|l| l.id == *id)
    }

    /// Get a horizontal line by ID.
    pub fn get_horizontal_line(&self, id: &DrawingId) -> Option<&HorizontalLine> {
        self.horizontal_lines.iter().find(|l| l.id == *id)
    }

    /// Get a vertical line by ID.
    pub fn get_vertical_line(&self, id: &DrawingId) -> Option<&VerticalLine> {
        self.vertical_lines.iter().find(|l| l.id == *id)
    }

    /// Get a rectangle by ID.
    pub fn get_rectangle(&self, id: &DrawingId) -> Option<&Rectangle> {
        self.rectangles.iter().find(|r| r.id == *id)
    }

    /// Get a Fibonacci retracement by ID.
    pub fn get_fibonacci_retracement(&self, id: &DrawingId) -> Option<&FibonacciRetracement> {
        self.fibonacci_retracements.iter().find(|f| f.id == *id)
    }

    /// Get all trend lines.
    pub fn all_trend_lines(&self) -> &[TrendLine] {
        &self.trend_lines
    }

    /// Get all horizontal lines.
    pub fn all_horizontal_lines(&self) -> &[HorizontalLine] {
        &self.horizontal_lines
    }

    /// Get all vertical lines.
    pub fn all_vertical_lines(&self) -> &[VerticalLine] {
        &self.vertical_lines
    }

    /// Get all rectangles.
    pub fn all_rectangles(&self) -> &[Rectangle] {
        &self.rectangles
    }

    /// Get all Fibonacci retracements.
    pub fn all_fibonacci_retracements(&self) -> &[FibonacciRetracement] {
        &self.fibonacci_retracements
    }

    /// Get a Fibonacci extension by ID.
    pub fn get_fibonacci_extension(&self, id: &DrawingId) -> Option<&FibonacciExtension> {
        self.fibonacci_extensions.iter().find(|e| e.id == *id)
    }

    /// Get all Fibonacci extensions.
    pub fn all_fibonacci_extensions(&self) -> &[FibonacciExtension] {
        &self.fibonacci_extensions
    }

    /// Get a pitchfork by ID.
    pub fn get_pitchfork(&self, id: &DrawingId) -> Option<&Pitchfork> {
        self.pitchforks.iter().find(|p| p.id == *id)
    }

    /// Get all pitchforks.
    pub fn all_pitchforks(&self) -> &[Pitchfork] {
        &self.pitchforks
    }

    /// Total number of drawings across all types.
    pub fn len(&self) -> usize {
        self.trend_lines.len()
            + self.horizontal_lines.len()
            + self.vertical_lines.len()
            + self.rectangles.len()
            + self.fibonacci_retracements.len()
            + self.fibonacci_extensions.len()
            + self.pitchforks.len()
    }

    /// Check if the set contains no drawings.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- ChartPoint ----

    #[test]
    fn chart_point_new() {
        let p = ChartPoint::new(1000, 50.5);
        assert_eq!(p.timestamp, 1000);
        assert_eq!(p.price, 50.5);
    }

    // ---- TrendLine ----

    #[test]
    fn trend_line_new_defaults() {
        let start = ChartPoint::new(100, 10.0);
        let end = ChartPoint::new(200, 20.0);
        let tl = TrendLine::new("tl1", start, end);

        assert_eq!(tl.id, DrawingId("tl1".to_string()));
        assert_eq!(tl.start.timestamp, 100);
        assert_eq!(tl.end.price, 20.0);
        assert_eq!(tl.color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(tl.width, 1.0);
        assert_eq!(tl.style, LineStyle::Solid);
    }

    #[test]
    fn trend_line_builder() {
        let tl = TrendLine::new(
            "tl2",
            ChartPoint::new(1, 5.0),
            ChartPoint::new(2, 10.0),
        )
        .with_color([1.0, 0.0, 0.0, 1.0])
        .with_width(2.5)
        .with_style(LineStyle::Dashed);

        assert_eq!(tl.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(tl.width, 2.5);
        assert_eq!(tl.style, LineStyle::Dashed);
    }

    #[test]
    fn trend_line_clone() {
        let tl = TrendLine::new("c", ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0));
        let cloned = tl.clone();
        assert_eq!(cloned.id, tl.id);
    }

    // ---- HorizontalLine ----

    #[test]
    fn horizontal_line_new_defaults() {
        let hl = HorizontalLine::new("hl1", 150.0);

        assert_eq!(hl.id, DrawingId("hl1".to_string()));
        assert_eq!(hl.price, 150.0);
        assert_eq!(hl.color, [0.5, 0.5, 0.5, 0.8]);
        assert_eq!(hl.width, 1.0);
        assert_eq!(hl.style, LineStyle::Solid);
        assert!(hl.extend_left);
        assert!(hl.extend_right);
    }

    #[test]
    fn horizontal_line_builder() {
        let hl = HorizontalLine::new("hl2", 200.0)
            .with_color([0.0, 1.0, 0.0, 1.0])
            .with_width(3.0)
            .with_style(LineStyle::Dotted)
            .with_extend_left(false)
            .with_extend_right(false);

        assert_eq!(hl.color, [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(hl.width, 3.0);
        assert_eq!(hl.style, LineStyle::Dotted);
        assert!(!hl.extend_left);
        assert!(!hl.extend_right);
    }

    // ---- VerticalLine ----

    #[test]
    fn vertical_line_new_defaults() {
        let vl = VerticalLine::new("vl1", 500);

        assert_eq!(vl.id, DrawingId("vl1".to_string()));
        assert_eq!(vl.timestamp, 500);
        assert_eq!(vl.color, [0.5, 0.5, 0.5, 0.8]);
        assert_eq!(vl.width, 1.0);
        assert_eq!(vl.style, LineStyle::Solid);
    }

    #[test]
    fn vertical_line_builder() {
        let vl = VerticalLine::new("vl2", 600)
            .with_color([1.0, 1.0, 0.0, 1.0])
            .with_width(1.5)
            .with_style(LineStyle::Dashed);

        assert_eq!(vl.color, [1.0, 1.0, 0.0, 1.0]);
        assert_eq!(vl.width, 1.5);
        assert_eq!(vl.style, LineStyle::Dashed);
    }

    // ---- Rectangle ----

    #[test]
    fn rectangle_new_defaults() {
        let tl = ChartPoint::new(100, 200.0);
        let br = ChartPoint::new(300, 100.0);
        let rect = Rectangle::new("r1", tl, br);

        assert_eq!(rect.id, DrawingId("r1".to_string()));
        assert_eq!(rect.top_left.timestamp, 100);
        assert_eq!(rect.bottom_right.price, 100.0);
        assert_eq!(rect.color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(rect.width, 1.0);
        assert_eq!(rect.style, LineStyle::Solid);
        assert!(rect.fill_color.is_none());
    }

    #[test]
    fn rectangle_builder() {
        let rect = Rectangle::new(
            "r2",
            ChartPoint::new(10, 50.0),
            ChartPoint::new(20, 30.0),
        )
        .with_color([1.0, 0.0, 0.0, 0.5])
        .with_width(2.0)
        .with_style(LineStyle::Dotted)
        .with_fill([0.0, 1.0, 0.0, 0.3]);

        assert_eq!(rect.color, [1.0, 0.0, 0.0, 0.5]);
        assert_eq!(rect.width, 2.0);
        assert_eq!(rect.style, LineStyle::Dotted);
        assert_eq!(rect.fill_color, Some([0.0, 1.0, 0.0, 0.3]));
    }

    #[test]
    fn rectangle_width_ts() {
        let rect = Rectangle::new(
            "r3",
            ChartPoint::new(500, 10.0),
            ChartPoint::new(800, 20.0),
        );
        assert_eq!(rect.width_ts(), 300);
    }

    #[test]
    fn rectangle_width_ts_reversed_corners() {
        // top_left has a later timestamp than bottom_right — should still work
        let rect = Rectangle::new(
            "r4",
            ChartPoint::new(800, 20.0),
            ChartPoint::new(500, 10.0),
        );
        assert_eq!(rect.width_ts(), 300);
    }

    #[test]
    fn rectangle_height_price() {
        let rect = Rectangle::new(
            "r5",
            ChartPoint::new(1, 150.0),
            ChartPoint::new(2, 80.0),
        );
        assert!((rect.height_price() - 70.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rectangle_height_price_reversed_corners() {
        let rect = Rectangle::new(
            "r6",
            ChartPoint::new(1, 80.0),
            ChartPoint::new(2, 150.0),
        );
        assert!((rect.height_price() - 70.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rectangle_zero_size() {
        let rect = Rectangle::new("r7", ChartPoint::new(100, 50.0), ChartPoint::new(100, 50.0));
        assert_eq!(rect.width_ts(), 0);
        assert!((rect.height_price()).abs() < f64::EPSILON);
    }

    #[test]
    fn rectangle_clone() {
        let rect = Rectangle::new("rc", ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0));
        let cloned = rect.clone();
        assert_eq!(cloned.id, rect.id);
    }

    // ---- FibonacciRetracement ----

    #[test]
    fn fibonacci_new_defaults() {
        let start = ChartPoint::new(100, 100.0);
        let end = ChartPoint::new(200, 200.0);
        let fib = FibonacciRetracement::new("f1", start, end);

        assert_eq!(fib.id, DrawingId("f1".to_string()));
        assert_eq!(fib.start.price, 100.0);
        assert_eq!(fib.end.price, 200.0);
        assert_eq!(fib.levels, vec![0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0]);
        assert_eq!(fib.color, [0.5, 0.5, 0.5, 0.8]);
        assert_eq!(fib.width, 1.0);
        assert_eq!(fib.style, LineStyle::Dashed);
    }

    #[test]
    fn fibonacci_builder() {
        let fib = FibonacciRetracement::new(
            "f2",
            ChartPoint::new(0, 50.0),
            ChartPoint::new(1, 100.0),
        )
        .with_color([0.0, 0.0, 1.0, 1.0])
        .with_width(2.0)
        .with_style(LineStyle::Solid)
        .with_levels(vec![0.0, 0.5, 1.0]);

        assert_eq!(fib.color, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(fib.width, 2.0);
        assert_eq!(fib.style, LineStyle::Solid);
        assert_eq!(fib.levels, vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn fibonacci_price_at_level() {
        let fib = FibonacciRetracement::new(
            "f3",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 200.0),
        );

        // range = 100.0
        assert!((fib.price_at_level(0.0) - 100.0).abs() < f64::EPSILON);
        assert!((fib.price_at_level(0.5) - 150.0).abs() < f64::EPSILON);
        assert!((fib.price_at_level(1.0) - 200.0).abs() < f64::EPSILON);
        assert!((fib.price_at_level(0.382) - 138.2).abs() < 1e-10);
        assert!((fib.price_at_level(0.618) - 161.8).abs() < 1e-10);
    }

    #[test]
    fn fibonacci_price_at_level_downtrend() {
        // start price > end price (downtrend)
        let fib = FibonacciRetracement::new(
            "f4",
            ChartPoint::new(0, 200.0),
            ChartPoint::new(1, 100.0),
        );

        // range = -100.0
        assert!((fib.price_at_level(0.0) - 200.0).abs() < f64::EPSILON);
        assert!((fib.price_at_level(0.5) - 150.0).abs() < f64::EPSILON);
        assert!((fib.price_at_level(1.0) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fibonacci_level_prices_count() {
        let fib = FibonacciRetracement::new(
            "f5",
            ChartPoint::new(0, 0.0),
            ChartPoint::new(1, 100.0),
        );
        let prices = fib.level_prices();
        assert_eq!(prices.len(), 7); // default levels count
    }

    #[test]
    fn fibonacci_level_prices_custom() {
        let fib = FibonacciRetracement::new(
            "f6",
            ChartPoint::new(0, 50.0),
            ChartPoint::new(1, 150.0),
        )
        .with_levels(vec![0.0, 0.5, 1.0]);

        let prices = fib.level_prices();
        assert_eq!(prices.len(), 3);
        assert!((prices[0].1 - 50.0).abs() < f64::EPSILON);
        assert!((prices[1].1 - 100.0).abs() < f64::EPSILON);
        assert!((prices[2].1 - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fibonacci_zero_range() {
        let fib = FibonacciRetracement::new(
            "f7",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 100.0),
        );
        // All levels should return the same price
        let prices = fib.level_prices();
        for &(_, price) in &prices {
            assert!((price - 100.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn fibonacci_clone() {
        let fib = FibonacciRetracement::new(
            "fc",
            ChartPoint::new(0, 0.0),
            ChartPoint::new(1, 100.0),
        );
        let cloned = fib.clone();
        assert_eq!(cloned.id, fib.id);
        assert_eq!(cloned.levels, fib.levels);
    }

    // ---- FibonacciExtension ----

    #[test]
    fn fibonacci_extension_new_defaults() {
        let a = ChartPoint::new(100, 100.0);
        let b = ChartPoint::new(200, 200.0);
        let c = ChartPoint::new(250, 160.0);
        let ext = FibonacciExtension::new("fe1", a, b, c);

        assert_eq!(ext.id, DrawingId("fe1".to_string()));
        assert_eq!(ext.point_a.price, 100.0);
        assert_eq!(ext.point_b.price, 200.0);
        assert_eq!(ext.point_c.price, 160.0);
        assert_eq!(
            ext.levels,
            vec![0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0, 1.272, 1.618]
        );
        assert_eq!(ext.color, [0.5, 0.5, 0.5, 0.8]);
        assert_eq!(ext.width, 1.0);
        assert_eq!(ext.style, LineStyle::Dashed);
    }

    #[test]
    fn fibonacci_extension_builder() {
        let ext = FibonacciExtension::new(
            "fe2",
            ChartPoint::new(0, 50.0),
            ChartPoint::new(1, 100.0),
            ChartPoint::new(2, 80.0),
        )
        .with_color([0.0, 0.0, 1.0, 1.0])
        .with_width(2.0)
        .with_style(LineStyle::Solid)
        .with_levels(vec![0.0, 0.5, 1.0, 1.618]);

        assert_eq!(ext.color, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(ext.width, 2.0);
        assert_eq!(ext.style, LineStyle::Solid);
        assert_eq!(ext.levels, vec![0.0, 0.5, 1.0, 1.618]);
    }

    #[test]
    fn fibonacci_extension_price_at_level() {
        // A=100, B=200, C=160 → ab_range=100
        // level 0.0  → 160 + 100*0.0   = 160.0
        // level 0.5  → 160 + 100*0.5   = 210.0
        // level 1.0  → 160 + 100*1.0   = 260.0
        // level 1.618→ 160 + 100*1.618 = 321.8
        let ext = FibonacciExtension::new(
            "fe3",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 200.0),
            ChartPoint::new(2, 160.0),
        );

        assert!((ext.price_at_level(0.0) - 160.0).abs() < f64::EPSILON);
        assert!((ext.price_at_level(0.5) - 210.0).abs() < f64::EPSILON);
        assert!((ext.price_at_level(1.0) - 260.0).abs() < f64::EPSILON);
        assert!((ext.price_at_level(1.618) - 321.8).abs() < 1e-10);
    }

    #[test]
    fn fibonacci_extension_price_at_level_downtrend() {
        // A=200, B=100, C=140 → ab_range=-100
        // level 0.0  → 140 + (-100)*0.0 = 140.0
        // level 1.0  → 140 + (-100)*1.0 = 40.0
        // level 1.618→ 140 + (-100)*1.618 = -21.8
        let ext = FibonacciExtension::new(
            "fe4",
            ChartPoint::new(0, 200.0),
            ChartPoint::new(1, 100.0),
            ChartPoint::new(2, 140.0),
        );

        assert!((ext.price_at_level(0.0) - 140.0).abs() < f64::EPSILON);
        assert!((ext.price_at_level(1.0) - 40.0).abs() < f64::EPSILON);
        assert!((ext.price_at_level(1.618) - (-21.8)).abs() < 1e-10);
    }

    #[test]
    fn fibonacci_extension_level_prices_count() {
        let ext = FibonacciExtension::new(
            "fe5",
            ChartPoint::new(0, 0.0),
            ChartPoint::new(1, 100.0),
            ChartPoint::new(2, 50.0),
        );
        let prices = ext.level_prices();
        assert_eq!(prices.len(), 9); // default extension levels count
    }

    #[test]
    fn fibonacci_extension_level_prices_custom() {
        let ext = FibonacciExtension::new(
            "fe6",
            ChartPoint::new(0, 50.0),
            ChartPoint::new(1, 150.0),
            ChartPoint::new(2, 100.0),
        )
        .with_levels(vec![0.0, 1.0, 1.618]);

        let prices = ext.level_prices();
        assert_eq!(prices.len(), 3);
        // ab_range = 100, C = 100
        assert!((prices[0].1 - 100.0).abs() < f64::EPSILON); // 100 + 100*0
        assert!((prices[1].1 - 200.0).abs() < f64::EPSILON); // 100 + 100*1
        assert!((prices[2].1 - 261.8).abs() < 1e-10); // 100 + 100*1.618
    }

    #[test]
    fn fibonacci_extension_zero_range() {
        let ext = FibonacciExtension::new(
            "fe7",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 100.0),
            ChartPoint::new(2, 100.0),
        );
        // All levels should return C.price = 100.0
        let prices = ext.level_prices();
        for &(_, price) in &prices {
            assert!((price - 100.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn fibonacci_extension_clone() {
        let ext = FibonacciExtension::new(
            "fec",
            ChartPoint::new(0, 0.0),
            ChartPoint::new(1, 100.0),
            ChartPoint::new(2, 50.0),
        );
        let cloned = ext.clone();
        assert_eq!(cloned.id, ext.id);
        assert_eq!(cloned.levels, ext.levels);
        assert_eq!(cloned.point_a, ext.point_a);
    }

    // ---- Pitchfork ----

    #[test]
    fn pitchfork_new_defaults() {
        let a = ChartPoint::new(100, 100.0);
        let b = ChartPoint::new(200, 200.0);
        let c = ChartPoint::new(200, 120.0);
        let pf = Pitchfork::new("pf1", a, b, c);

        assert_eq!(pf.id, DrawingId("pf1".to_string()));
        assert_eq!(pf.point_a.price, 100.0);
        assert_eq!(pf.point_b.price, 200.0);
        assert_eq!(pf.point_c.price, 120.0);
        assert_eq!(pf.color, [0.5, 0.5, 0.5, 0.8]);
        assert_eq!(pf.width, 1.0);
        assert_eq!(pf.style, LineStyle::Solid);
    }

    #[test]
    fn pitchfork_builder() {
        let pf = Pitchfork::new(
            "pf2",
            ChartPoint::new(0, 50.0),
            ChartPoint::new(1, 100.0),
            ChartPoint::new(2, 60.0),
        )
        .with_color([1.0, 0.0, 0.0, 1.0])
        .with_width(2.5)
        .with_style(LineStyle::Dashed);

        assert_eq!(pf.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(pf.width, 2.5);
        assert_eq!(pf.style, LineStyle::Dashed);
    }

    #[test]
    fn pitchfork_median_at_a() {
        // At A's timestamp, median should be A's price
        let a = ChartPoint::new(0, 100.0);
        let b = ChartPoint::new(100, 200.0);
        let c = ChartPoint::new(100, 120.0);
        let pf = Pitchfork::new("pf3", a, b, c);

        let price = pf.median_price_at(0);
        assert!((price - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pitchfork_median_at_midpoint() {
        // B and C both at timestamp 100 → midpoint ts = 100
        // midpoint price = (200 + 120) / 2 = 160
        let a = ChartPoint::new(0, 100.0);
        let b = ChartPoint::new(100, 200.0);
        let c = ChartPoint::new(100, 120.0);
        let pf = Pitchfork::new("pf4", a, b, c);

        let price = pf.median_price_at(100);
        assert!((price - 160.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pitchfork_median_interpolation() {
        // Linear interpolation: at t=50 (halfway between 0 and 100),
        // median = 100 + (160 - 100) * 0.5 = 130
        let a = ChartPoint::new(0, 100.0);
        let b = ChartPoint::new(100, 200.0);
        let c = ChartPoint::new(100, 120.0);
        let pf = Pitchfork::new("pf5", a, b, c);

        let price = pf.median_price_at(50);
        assert!((price - 130.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pitchfork_upper_at_a() {
        // At A's timestamp, upper = A.price + (B.price - midpoint)
        // midpoint = (200+120)/2 = 160, offset = 200-160 = 40
        // at t=0: upper = 100 + 0 + 40 = 140
        let a = ChartPoint::new(0, 100.0);
        let b = ChartPoint::new(100, 200.0);
        let c = ChartPoint::new(100, 120.0);
        let pf = Pitchfork::new("pf6", a, b, c);

        let price = pf.upper_price_at(0);
        assert!((price - 140.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pitchfork_upper_at_midpoint() {
        // At midpoint: median = 160, upper = 160 + 40 = 200
        let a = ChartPoint::new(0, 100.0);
        let b = ChartPoint::new(100, 200.0);
        let c = ChartPoint::new(100, 120.0);
        let pf = Pitchfork::new("pf7", a, b, c);

        let price = pf.upper_price_at(100);
        assert!((price - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pitchfork_lower_at_a() {
        // At A's timestamp: lower = A.price + 0 + (C.price - midpoint)
        // midpoint = 160, offset = 120 - 160 = -40
        // at t=0: lower = 100 + 0 + (-40) = 60
        let a = ChartPoint::new(0, 100.0);
        let b = ChartPoint::new(100, 200.0);
        let c = ChartPoint::new(100, 120.0);
        let pf = Pitchfork::new("pf8", a, b, c);

        let price = pf.lower_price_at(0);
        assert!((price - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pitchfork_lower_at_midpoint() {
        // At midpoint: lower = 160 + (-40) = 120
        let a = ChartPoint::new(0, 100.0);
        let b = ChartPoint::new(100, 200.0);
        let c = ChartPoint::new(100, 120.0);
        let pf = Pitchfork::new("pf9", a, b, c);

        let price = pf.lower_price_at(100);
        assert!((price - 120.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pitchfork_asymmetric_b_c() {
        // B and C at different timestamps
        // B at t=80, C at t=120 → midpoint ts = 100
        // midpoint price = (200 + 80) / 2 = 140
        let a = ChartPoint::new(0, 100.0);
        let b = ChartPoint::new(80, 200.0);
        let c = ChartPoint::new(120, 80.0);
        let pf = Pitchfork::new("pf10", a, b, c);

        // At t=100 (midpoint), median should be midpoint price
        let median = pf.median_price_at(100);
        assert!((median - 140.0).abs() < f64::EPSILON);

        // Upper offset = 200 - 140 = 60
        let upper = pf.upper_price_at(100);
        assert!((upper - 200.0).abs() < f64::EPSILON);

        // Lower offset = 80 - 140 = -60
        let lower = pf.lower_price_at(100);
        assert!((lower - 80.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pitchfork_past_a() {
        // Before A: saturating_sub clamps t to 0, factor = 0
        // median = A.price = 100
        let a = ChartPoint::new(100, 100.0);
        let b = ChartPoint::new(200, 200.0);
        let c = ChartPoint::new(200, 120.0);
        let pf = Pitchfork::new("pf11", a, b, c);

        let price = pf.median_price_at(0);
        assert!((price - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pitchfork_zero_span() {
        // A and midpoint share the same timestamp → denominator = 0
        let a = ChartPoint::new(100, 100.0);
        let b = ChartPoint::new(100, 200.0);
        let c = ChartPoint::new(100, 120.0);
        let pf = Pitchfork::new("pf12", a, b, c);

        // factor = 0, median = A.price = 100
        let price = pf.median_price_at(100);
        assert!((price - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pitchfork_clone() {
        let pf = Pitchfork::new(
            "pfc",
            ChartPoint::new(0, 0.0),
            ChartPoint::new(1, 100.0),
            ChartPoint::new(2, 50.0),
        );
        let cloned = pf.clone();
        assert_eq!(cloned.id, pf.id);
        assert_eq!(cloned.point_a, pf.point_a);
    }

    // ---- DrawingSet ----

    #[test]
    fn drawing_set_starts_empty() {
        let set = DrawingSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn drawing_set_add_trend_line() {
        let mut set = DrawingSet::new();
        set.add_trend_line(TrendLine::new(
            "t1",
            ChartPoint::new(0, 0.0),
            ChartPoint::new(1, 1.0),
        ));
        assert_eq!(set.len(), 1);
        assert!(!set.is_empty());
        assert!(set.get_trend_line(&DrawingId("t1".to_string())).is_some());
    }

    #[test]
    fn drawing_set_add_horizontal_line() {
        let mut set = DrawingSet::new();
        set.add_horizontal_line(HorizontalLine::new("h1", 100.0));
        assert_eq!(set.len(), 1);
        assert!(set.get_horizontal_line(&DrawingId("h1".to_string())).is_some());
    }

    #[test]
    fn drawing_set_add_vertical_line() {
        let mut set = DrawingSet::new();
        set.add_vertical_line(VerticalLine::new("v1", 42));
        assert_eq!(set.len(), 1);
        assert!(set.get_vertical_line(&DrawingId("v1".to_string())).is_some());
    }

    #[test]
    fn drawing_set_remove_trend_line() {
        let mut set = DrawingSet::new();
        set.add_trend_line(TrendLine::new(
            "t1",
            ChartPoint::new(0, 0.0),
            ChartPoint::new(1, 1.0),
        ));
        assert!(set.remove(&DrawingId("t1".to_string())));
        assert_eq!(set.len(), 0);
        assert!(set.get_trend_line(&DrawingId("t1".to_string())).is_none());
    }

    #[test]
    fn drawing_set_remove_horizontal_line() {
        let mut set = DrawingSet::new();
        set.add_horizontal_line(HorizontalLine::new("h1", 100.0));
        assert!(set.remove(&DrawingId("h1".to_string())));
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn drawing_set_remove_vertical_line() {
        let mut set = DrawingSet::new();
        set.add_vertical_line(VerticalLine::new("v1", 42));
        assert!(set.remove(&DrawingId("v1".to_string())));
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn drawing_set_remove_nonexistent() {
        let mut set = DrawingSet::new();
        set.add_horizontal_line(HorizontalLine::new("h1", 100.0));
        assert!(!set.remove(&DrawingId("nope".to_string())));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn drawing_set_remove_from_mixed() {
        let mut set = DrawingSet::new();
        set.add_trend_line(TrendLine::new(
            "t1",
            ChartPoint::new(0, 0.0),
            ChartPoint::new(1, 1.0),
        ));
        set.add_horizontal_line(HorizontalLine::new("h1", 50.0));
        set.add_vertical_line(VerticalLine::new("v1", 10));
        assert_eq!(set.len(), 3);

        assert!(set.remove(&DrawingId("h1".to_string())));
        assert_eq!(set.len(), 2);
        assert!(set.get_horizontal_line(&DrawingId("h1".to_string())).is_none());
        assert!(set.get_trend_line(&DrawingId("t1".to_string())).is_some());
        assert!(set.get_vertical_line(&DrawingId("v1".to_string())).is_some());
    }

    #[test]
    fn drawing_set_len_counts_all_types() {
        let mut set = DrawingSet::new();
        set.add_trend_line(TrendLine::new(
            "t1",
            ChartPoint::new(0, 0.0),
            ChartPoint::new(1, 1.0),
        ));
        set.add_trend_line(TrendLine::new(
            "t2",
            ChartPoint::new(2, 2.0),
            ChartPoint::new(3, 3.0),
        ));
        set.add_horizontal_line(HorizontalLine::new("h1", 100.0));
        set.add_horizontal_line(HorizontalLine::new("h2", 200.0));
        set.add_horizontal_line(HorizontalLine::new("h3", 300.0));
        set.add_vertical_line(VerticalLine::new("v1", 1));

        assert_eq!(set.len(), 6);
        assert_eq!(set.all_trend_lines().len(), 2);
        assert_eq!(set.all_horizontal_lines().len(), 3);
        assert_eq!(set.all_vertical_lines().len(), 1);
    }

    #[test]
    fn drawing_set_is_empty_after_removing_last() {
        let mut set = DrawingSet::new();
        set.add_vertical_line(VerticalLine::new("v1", 1));
        assert!(!set.is_empty());
        set.remove(&DrawingId("v1".to_string()));
        assert!(set.is_empty());
    }

    // ---- DrawingSet: Rectangle ----

    #[test]
    fn drawing_set_add_rectangle() {
        let mut set = DrawingSet::new();
        set.add_rectangle(Rectangle::new(
            "r1",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 50.0),
        ));
        assert_eq!(set.len(), 1);
        assert!(set.get_rectangle(&DrawingId("r1".to_string())).is_some());
    }

    #[test]
    fn drawing_set_remove_rectangle() {
        let mut set = DrawingSet::new();
        set.add_rectangle(Rectangle::new(
            "r1",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 50.0),
        ));
        assert!(set.remove(&DrawingId("r1".to_string())));
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn drawing_set_all_rectangles() {
        let mut set = DrawingSet::new();
        set.add_rectangle(Rectangle::new(
            "r1",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 50.0),
        ));
        set.add_rectangle(Rectangle::new(
            "r2",
            ChartPoint::new(2, 200.0),
            ChartPoint::new(3, 150.0),
        ));
        assert_eq!(set.all_rectangles().len(), 2);
    }

    // ---- DrawingSet: FibonacciRetracement ----

    #[test]
    fn drawing_set_add_fibonacci() {
        let mut set = DrawingSet::new();
        set.add_fibonacci_retracement(FibonacciRetracement::new(
            "f1",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 200.0),
        ));
        assert_eq!(set.len(), 1);
        assert!(set
            .get_fibonacci_retracement(&DrawingId("f1".to_string()))
            .is_some());
    }

    #[test]
    fn drawing_set_remove_fibonacci() {
        let mut set = DrawingSet::new();
        set.add_fibonacci_retracement(FibonacciRetracement::new(
            "f1",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 200.0),
        ));
        assert!(set.remove(&DrawingId("f1".to_string())));
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn drawing_set_mixed_with_new_types() {
        let mut set = DrawingSet::new();
        set.add_trend_line(TrendLine::new(
            "t1",
            ChartPoint::new(0, 0.0),
            ChartPoint::new(1, 1.0),
        ));
        set.add_rectangle(Rectangle::new(
            "r1",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 50.0),
        ));
        set.add_fibonacci_retracement(FibonacciRetracement::new(
            "f1",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 200.0),
        ));
        assert_eq!(set.len(), 3);

        assert!(set.remove(&DrawingId("r1".to_string())));
        assert_eq!(set.len(), 2);
        assert!(set.get_trend_line(&DrawingId("t1".to_string())).is_some());
        assert!(set
            .get_fibonacci_retracement(&DrawingId("f1".to_string()))
            .is_some());
    }

    // ---- DrawingSet: FibonacciExtension ----

    #[test]
    fn drawing_set_add_fibonacci_extension() {
        let mut set = DrawingSet::new();
        set.add_fibonacci_extension(FibonacciExtension::new(
            "fe1",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 200.0),
            ChartPoint::new(2, 150.0),
        ));
        assert_eq!(set.len(), 1);
        assert!(set
            .get_fibonacci_extension(&DrawingId("fe1".to_string()))
            .is_some());
    }

    #[test]
    fn drawing_set_remove_fibonacci_extension() {
        let mut set = DrawingSet::new();
        set.add_fibonacci_extension(FibonacciExtension::new(
            "fe1",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 200.0),
            ChartPoint::new(2, 150.0),
        ));
        assert!(set.remove(&DrawingId("fe1".to_string())));
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn drawing_set_all_fibonacci_extensions() {
        let mut set = DrawingSet::new();
        set.add_fibonacci_extension(FibonacciExtension::new(
            "fe1",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 200.0),
            ChartPoint::new(2, 150.0),
        ));
        set.add_fibonacci_extension(FibonacciExtension::new(
            "fe2",
            ChartPoint::new(3, 50.0),
            ChartPoint::new(4, 100.0),
            ChartPoint::new(5, 80.0),
        ));
        assert_eq!(set.all_fibonacci_extensions().len(), 2);
    }

    // ---- DrawingSet: Pitchfork ----

    #[test]
    fn drawing_set_add_pitchfork() {
        let mut set = DrawingSet::new();
        set.add_pitchfork(Pitchfork::new(
            "pf1",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 200.0),
            ChartPoint::new(2, 120.0),
        ));
        assert_eq!(set.len(), 1);
        assert!(set
            .get_pitchfork(&DrawingId("pf1".to_string()))
            .is_some());
    }

    #[test]
    fn drawing_set_remove_pitchfork() {
        let mut set = DrawingSet::new();
        set.add_pitchfork(Pitchfork::new(
            "pf1",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 200.0),
            ChartPoint::new(2, 120.0),
        ));
        assert!(set.remove(&DrawingId("pf1".to_string())));
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn drawing_set_all_pitchforks() {
        let mut set = DrawingSet::new();
        set.add_pitchfork(Pitchfork::new(
            "pf1",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 200.0),
            ChartPoint::new(2, 120.0),
        ));
        set.add_pitchfork(Pitchfork::new(
            "pf2",
            ChartPoint::new(3, 50.0),
            ChartPoint::new(4, 100.0),
            ChartPoint::new(5, 60.0),
        ));
        assert_eq!(set.all_pitchforks().len(), 2);
    }

    #[test]
    fn drawing_set_mixed_all_types() {
        let mut set = DrawingSet::new();
        set.add_trend_line(TrendLine::new(
            "t1",
            ChartPoint::new(0, 0.0),
            ChartPoint::new(1, 1.0),
        ));
        set.add_fibonacci_extension(FibonacciExtension::new(
            "fe1",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 200.0),
            ChartPoint::new(2, 150.0),
        ));
        set.add_pitchfork(Pitchfork::new(
            "pf1",
            ChartPoint::new(0, 100.0),
            ChartPoint::new(1, 200.0),
            ChartPoint::new(2, 120.0),
        ));
        assert_eq!(set.len(), 3);

        assert!(set.remove(&DrawingId("fe1".to_string())));
        assert_eq!(set.len(), 2);
        assert!(set.get_trend_line(&DrawingId("t1".to_string())).is_some());
        assert!(set
            .get_pitchfork(&DrawingId("pf1".to_string()))
            .is_some());
    }
}
