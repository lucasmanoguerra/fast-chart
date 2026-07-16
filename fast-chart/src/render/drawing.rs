// ---------------------------------------------------------------------------
// Drawing — unified trait for all chart drawing tools
// ---------------------------------------------------------------------------

use crate::render::commands::DrawCommand;
use crate::render::series_renderer::Rect;
use fast_chart_domain::drawing::{ChartPoint, DrawingId};

/// Result of a hit-test against a drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitResult {
    /// No hit — the point is not near this drawing.
    Miss,
    /// Hit on the drawing body (line, shape interior, etc.).
    Body,
    /// Hit on a control point (endpoint, handle, vertex).
    ControlPoint(usize),
}

/// A chart drawing that can be hit-tested, moved, bounded, and rendered.
///
/// Every drawing type (TrendLine, Rectangle, Arrow, etc.) implements this
/// trait so the `DrawingManager` can handle them polymorphically.
pub trait Drawing: Send + Sync {
    /// Unique identifier for this drawing.
    fn id(&self) -> &DrawingId;

    /// Test whether a chart point hits this drawing.
    ///
    /// `tolerance` is the maximum distance (in pixels) for a hit.
    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult;

    /// Move this drawing by the given delta (in chart coordinates).
    fn move_by(&mut self, delta: ChartPoint);

    /// Bounding rectangle in chart coordinates (timestamp, price).
    fn bounds(&self) -> DrawingBounds;

    /// Whether this drawing is currently selected.
    fn is_selected(&self) -> bool;

    /// Set the selection state.
    fn set_selected(&mut self, selected: bool);

    /// Generate render commands for this drawing.
    fn to_commands(&self) -> Vec<DrawCommand>;
}

/// Bounding box in chart coordinates (timestamp range + price range).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawingBounds {
    pub time_start: u64,
    pub time_end: u64,
    pub price_min: f64,
    pub price_max: f64,
}

impl DrawingBounds {
    pub fn new(time_start: u64, time_end: u64, price_min: f64, price_max: f64) -> Self {
        Self { time_start, time_end, price_min, price_max }
    }

    /// Create bounds from a single point (zero-size).
    pub fn from_point(p: ChartPoint) -> Self {
        Self { time_start: p.timestamp, time_end: p.timestamp, price_min: p.price, price_max: p.price }
    }

    /// Create bounds from two points.
    pub fn from_points(a: ChartPoint, b: ChartPoint) -> Self {
        Self {
            time_start: a.timestamp.min(b.timestamp),
            time_end: a.timestamp.max(b.timestamp),
            price_min: a.price.min(b.price),
            price_max: a.price.max(b.price),
        }
    }

    /// Width in timestamp units.
    pub fn time_width(&self) -> u64 {
        self.time_end.saturating_sub(self.time_start)
    }

    /// Height in price units.
    pub fn price_height(&self) -> f64 {
        self.price_max - self.price_min
    }

    /// Check if a point is inside these bounds.
    pub fn contains(&self, p: ChartPoint) -> bool {
        p.timestamp >= self.time_start
            && p.timestamp <= self.time_end
            && p.price >= self.price_min
            && p.price <= self.price_max
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawing_bounds_from_point() {
        let b = DrawingBounds::from_point(ChartPoint::new(1000, 50.0));
        assert_eq!(b.time_start, 1000);
        assert_eq!(b.time_end, 1000);
        assert!((b.price_min - 50.0).abs() < f64::EPSILON);
        assert!((b.price_max - 50.0).abs() < f64::EPSILON);
        assert_eq!(b.time_width(), 0);
        assert!((b.price_height()).abs() < f64::EPSILON);
    }

    #[test]
    fn drawing_bounds_from_points() {
        let a = ChartPoint::new(2000, 100.0);
        let b = ChartPoint::new(1000, 50.0);
        let bounds = DrawingBounds::from_points(a, b);
        assert_eq!(bounds.time_start, 1000);
        assert_eq!(bounds.time_end, 2000);
        assert!((bounds.price_min - 50.0).abs() < f64::EPSILON);
        assert!((bounds.price_max - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn drawing_bounds_contains() {
        let b = DrawingBounds::new(1000, 2000, 50.0, 100.0);
        assert!(b.contains(ChartPoint::new(1500, 75.0)));
        assert!(!b.contains(ChartPoint::new(500, 75.0)));
        assert!(!b.contains(ChartPoint::new(1500, 120.0)));
    }

    #[test]
    fn drawing_bounds_width_height() {
        let b = DrawingBounds::new(1000, 3000, 40.0, 80.0);
        assert_eq!(b.time_width(), 2000);
        assert!((b.price_height() - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hit_result_equality() {
        assert_eq!(HitResult::Miss, HitResult::Miss);
        assert_eq!(HitResult::Body, HitResult::Body);
        assert_eq!(HitResult::ControlPoint(0), HitResult::ControlPoint(0));
        assert_ne!(HitResult::ControlPoint(0), HitResult::ControlPoint(1));
        assert_ne!(HitResult::Miss, HitResult::Body);
    }

    #[test]
    fn drawing_trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        // We can't instantiate a trait object here, but we can verify
        // that the trait bounds are correct at the type level.
        assert_send_sync::<Box<dyn Drawing>>();
    }
}
