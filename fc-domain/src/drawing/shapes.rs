use crate::price_line::LineStyle;

use super::{ChartPoint, Drawing, DrawingId};

// ---------------------------------------------------------------------------
// Rectangle
// ---------------------------------------------------------------------------

/// A rectangle defined by two corner points.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Whether this drawing is currently selected.
    pub selected: bool,
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
            selected: false,
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

impl Drawing for Rectangle {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.top_left.timestamp = self.top_left.timestamp.saturating_add(delta.timestamp);
        self.top_left.price += delta.price;
        self.bottom_right.timestamp = self.bottom_right.timestamp.saturating_add(delta.timestamp);
        self.bottom_right.price += delta.price;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// Ellipse
// ---------------------------------------------------------------------------

/// An ellipse defined by center point and horizontal/vertical radii.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ellipse {
    /// Unique identifier.
    pub id: DrawingId,
    /// Center point of the ellipse.
    pub center: ChartPoint,
    /// Horizontal radius in timestamp units.
    pub radius_x: f64,
    /// Vertical radius in price units.
    pub radius_y: f64,
    /// Border color [r, g, b, a].
    pub color: [f32; 4],
    /// Border width in pixels.
    pub width: f32,
    /// Border line style.
    pub style: LineStyle,
    /// Optional fill color [r, g, b, a].
    pub fill_color: Option<[f32; 4]>,
    /// Whether this drawing is currently selected.
    pub selected: bool,
}

impl Ellipse {
    /// Create a new ellipse with default styling and no fill.
    pub fn new(id: impl Into<String>, center: ChartPoint, radius_x: f64, radius_y: f64) -> Self {
        Self {
            id: DrawingId(id.into()),
            center,
            radius_x,
            radius_y,
            color: [1.0, 1.0, 1.0, 1.0],
            width: 1.0,
            style: LineStyle::Solid,
            fill_color: None,
            selected: false,
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

    /// Check if a point is inside the ellipse.
    ///
    /// Uses the standard ellipse equation: `((x-cx)/rx)^2 + ((y-cy)/ry)^2 <= 1`.
    /// Returns `true` when the point is inside or exactly on the boundary.
    pub fn contains(&self, point: ChartPoint) -> bool {
        let dx = (point.timestamp as f64 - self.center.timestamp as f64) / self.radius_x;
        let dy = (point.price - self.center.price) / self.radius_y;
        dx * dx + dy * dy <= 1.0
    }

    /// Get the bounding box as `(min_point, max_point)`.
    ///
    /// Returns the bottom-left and top-right corners of the axis-aligned
    /// bounding rectangle that fully encloses the ellipse.
    pub fn bounding_box(&self) -> (ChartPoint, ChartPoint) {
        let min = ChartPoint::new(
            self.center.timestamp.saturating_sub(self.radius_x as u64),
            self.center.price - self.radius_y,
        );
        let max = ChartPoint::new(
            self.center.timestamp + self.radius_x as u64,
            self.center.price + self.radius_y,
        );
        (min, max)
    }
}

impl Drawing for Ellipse {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.center.timestamp = self.center.timestamp.saturating_add(delta.timestamp);
        self.center.price += delta.price;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
