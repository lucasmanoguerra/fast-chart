use crate::price_line::LineStyle;

use super::{ChartPoint, Drawing, DrawingId};

// ---------------------------------------------------------------------------
// TrendLine
// ---------------------------------------------------------------------------

/// A line segment connecting two points on the chart.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Whether this drawing is currently selected.
    pub selected: bool,
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
}

impl Drawing for TrendLine {
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
// Segment
// ---------------------------------------------------------------------------

/// A finite line segment between two points.
///
/// Simpler than TrendLine -- purely geometric with no drawing-tool semantics.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Segment {
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
    /// Whether this drawing is currently selected.
    pub selected: bool,
}

impl Segment {
    /// Create a new segment between two points with default styling.
    pub fn new(id: impl Into<String>, start: ChartPoint, end: ChartPoint) -> Self {
        Self {
            id: DrawingId(id.into()),
            start,
            end,
            color: [1.0, 1.0, 1.0, 1.0],
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
}

impl Drawing for Segment {
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
// Ray
// ---------------------------------------------------------------------------

/// A ray (half-line) starting at `start` and extending infinitely through `direction`.
///
/// Used for support/resistance lines that extend into the future.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ray {
    /// Unique identifier.
    pub id: DrawingId,
    /// Origin point.
    pub start: ChartPoint,
    /// A second point defining the direction (the ray extends from `start` through and beyond `direction`).
    pub direction: ChartPoint,
    /// Line color [r, g, b, a].
    pub color: [f32; 4],
    /// Line width in pixels.
    pub width: f32,
    /// Line style.
    pub style: LineStyle,
    /// Whether this drawing is currently selected.
    pub selected: bool,
}

impl Ray {
    /// Create a new ray starting at `start` and extending through `direction`.
    pub fn new(id: impl Into<String>, start: ChartPoint, direction: ChartPoint) -> Self {
        Self {
            id: DrawingId(id.into()),
            start,
            direction,
            color: [1.0, 1.0, 1.0, 1.0],
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
}

impl Drawing for Ray {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.start.timestamp = self.start.timestamp.saturating_add(delta.timestamp);
        self.start.price += delta.price;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// Arrow
// ---------------------------------------------------------------------------

/// An arrow with arrowhead at the end, typically used for directional annotations.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Arrow {
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
    /// Arrowhead size in pixels.
    pub arrowhead_size: f32,
    /// Whether this drawing is currently selected.
    pub selected: bool,
}

impl Arrow {
    /// Create a new arrow between two points with default styling.
    pub fn new(id: impl Into<String>, start: ChartPoint, end: ChartPoint) -> Self {
        Self {
            id: DrawingId(id.into()),
            start,
            end,
            color: [1.0, 1.0, 1.0, 1.0],
            width: 1.0,
            style: LineStyle::Solid,
            arrowhead_size: 12.0,
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

    /// Set the arrowhead size.
    pub fn with_arrowhead_size(mut self, size: f32) -> Self {
        self.arrowhead_size = size;
        self
    }
}

impl Drawing for Arrow {
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
// HorizontalLine
// ---------------------------------------------------------------------------

/// A horizontal line at a specific price level spanning across the chart.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Whether this drawing is currently selected.
    pub selected: bool,
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

impl Drawing for HorizontalLine {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.price += delta.price;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// VerticalLine
// ---------------------------------------------------------------------------

/// A vertical line at a specific timestamp spanning across the chart.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Whether this drawing is currently selected.
    pub selected: bool,
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
}

impl Drawing for VerticalLine {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.timestamp = self.timestamp.saturating_add(delta.timestamp);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// Path
// ---------------------------------------------------------------------------

/// A series of connected line segments (polyline).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Path {
    /// Unique identifier.
    pub id: DrawingId,
    /// Ordered list of points forming the path.
    pub points: Vec<ChartPoint>,
    /// Line color [r, g, b, a].
    pub color: [f32; 4],
    /// Line width in pixels.
    pub width: f32,
    /// Line style.
    pub style: LineStyle,
    /// If `true`, the last point connects back to the first.
    pub closed: bool,
    /// Optional fill color for closed paths.
    pub fill_color: Option<[f32; 4]>,
    /// Whether this drawing is currently selected.
    pub selected: bool,
}

impl Path {
    /// Create a new path with default styling, not closed.
    pub fn new(id: impl Into<String>, points: Vec<ChartPoint>) -> Self {
        Self {
            id: DrawingId(id.into()),
            points,
            color: [1.0, 1.0, 1.0, 1.0],
            width: 1.0,
            style: LineStyle::Solid,
            closed: false,
            fill_color: None,
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

    /// Set whether the path is closed.
    pub fn with_closed(mut self, closed: bool) -> Self {
        self.closed = closed;
        self
    }

    /// Set the fill color.
    pub fn with_fill(mut self, fill_color: [f32; 4]) -> Self {
        self.fill_color = Some(fill_color);
        self
    }

    /// Append a point to the end of the path.
    pub fn push(&mut self, point: ChartPoint) {
        self.points.push(point);
    }

    /// Get the number of line segments.
    ///
    /// Returns `0` for fewer than 2 points. For an open path with `n` points
    /// returns `n - 1`; for a closed path returns `n`.
    pub fn segment_count(&self) -> usize {
        if self.points.len() < 2 {
            0
        } else if self.closed {
            self.points.len()
        } else {
            self.points.len() - 1
        }
    }

    /// Calculate total length of the path (sum of Euclidean segment lengths).
    ///
    /// Distance is computed in `(timestamp, price)` space. For a closed path
    /// the closing segment (last point to first) is included.
    pub fn total_length(&self) -> f64 {
        if self.points.len() < 2 {
            return 0.0;
        }

        let mut total = 0.0;
        for window in self.points.windows(2) {
            let dx = window[1].timestamp as f64 - window[0].timestamp as f64;
            let dy = window[1].price - window[0].price;
            total += (dx * dx + dy * dy).sqrt();
        }

        if self.closed {
            let first = &self.points[0];
            let last = &self.points[self.points.len() - 1];
            let dx = last.timestamp as f64 - first.timestamp as f64;
            let dy = last.price - first.price;
            total += (dx * dx + dy * dy).sqrt();
        }

        total
    }

    /// Get a reference to the point at the given index.
    pub fn point(&self, index: usize) -> Option<&ChartPoint> {
        self.points.get(index)
    }
}

impl Drawing for Path {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn move_by(&mut self, delta: ChartPoint) {
        for pt in &mut self.points {
            pt.timestamp = pt.timestamp.saturating_add(delta.timestamp);
            pt.price += delta.price;
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
