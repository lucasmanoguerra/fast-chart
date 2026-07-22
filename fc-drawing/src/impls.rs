// ---------------------------------------------------------------------------
// Drawing trait implementations for all fc_domain drawing types
// ---------------------------------------------------------------------------

use crate::bounds::DrawingBounds;
use crate::hit::{HitResult, default_aabb_hit_test};
use crate::trait_def::Drawing;
use fc_domain::drawing::{
    ChartPoint, DrawingId,
    TrendLine, Arrow, Ray, Segment, TextDrawing, ImageDrawing, LabelDrawing,
    HorizontalLine, VerticalLine, Rectangle, FibonacciRetracement, FibonacciExtension,
    Pitchfork, Ellipse, Path,
};

// ---------------------------------------------------------------------------
// Helper: compute bounds from min/max of three points
// ---------------------------------------------------------------------------

fn bounds_from_points(points: &[ChartPoint]) -> DrawingBounds {
    let mut time_start = u64::MAX;
    let mut time_end = u64::MIN;
    let mut price_min = f64::MAX;
    let mut price_max = f64::MIN;
    for p in points {
        time_start = time_start.min(p.timestamp);
        time_end = time_end.max(p.timestamp);
        price_min = price_min.min(p.price);
        price_max = price_max.max(p.price);
    }
    DrawingBounds::new(time_start, time_end, price_min, price_max)
}

// ---------------------------------------------------------------------------
// TrendLine
// ---------------------------------------------------------------------------

impl Drawing for TrendLine {
    fn id(&self) -> &DrawingId { &self.id }
    fn move_by(&mut self, delta: ChartPoint) {
        self.start.timestamp = self.start.timestamp.saturating_add(delta.timestamp);
        self.start.price += delta.price;
        self.end.timestamp = self.end.timestamp.saturating_add(delta.timestamp);
        self.end.price += delta.price;
    }
    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::from_points(self.start, self.end)
    }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// ---------------------------------------------------------------------------
// Arrow
// ---------------------------------------------------------------------------

impl Drawing for Arrow {
    fn id(&self) -> &DrawingId { &self.id }
    fn move_by(&mut self, delta: ChartPoint) {
        self.start.timestamp = self.start.timestamp.saturating_add(delta.timestamp);
        self.start.price += delta.price;
        self.end.timestamp = self.end.timestamp.saturating_add(delta.timestamp);
        self.end.price += delta.price;
    }
    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::from_points(self.start, self.end)
    }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// ---------------------------------------------------------------------------
// Ray
// ---------------------------------------------------------------------------

impl Drawing for Ray {
    fn id(&self) -> &DrawingId { &self.id }
    fn move_by(&mut self, delta: ChartPoint) {
        self.start.timestamp = self.start.timestamp.saturating_add(delta.timestamp);
        self.start.price += delta.price;
    }
    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::from_points(self.start, self.direction)
    }
    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        // Project point onto ray line, check distance
        let dx = self.direction.timestamp as f64 - self.start.timestamp as f64;
        let dy = self.direction.price - self.start.price;
        let len_sq = dx * dx + dy * dy;
        if len_sq < f64::EPSILON {
            // Degenerate ray — fall back to point check
            let d = ((point.timestamp as f64 - self.start.timestamp as f64).powi(2)
                + (point.price - self.start.price).powi(2))
            .sqrt();
            return if d <= tolerance as f64 {
                HitResult::Body
            } else {
                HitResult::Miss
            };
        }
        let t = ((point.timestamp as f64 - self.start.timestamp as f64) * dx
            + (point.price - self.start.price) * dy)
            / len_sq;
        // Ray extends from t=0 to t=infinity
        let t_clamped = t.max(0.0);
        let proj_x = self.start.timestamp as f64 + t_clamped * dx;
        let proj_y = self.start.price + t_clamped * dy;
        let dist = ((point.timestamp as f64 - proj_x).powi(2)
            + (point.price - proj_y).powi(2))
        .sqrt();
        if dist <= tolerance as f64 {
            HitResult::Body
        } else {
            HitResult::Miss
        }
    }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// ---------------------------------------------------------------------------
// Segment
// ---------------------------------------------------------------------------

impl Drawing for Segment {
    fn id(&self) -> &DrawingId { &self.id }
    fn move_by(&mut self, delta: ChartPoint) {
        self.start.timestamp = self.start.timestamp.saturating_add(delta.timestamp);
        self.start.price += delta.price;
        self.end.timestamp = self.end.timestamp.saturating_add(delta.timestamp);
        self.end.price += delta.price;
    }
    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::from_points(self.start, self.end)
    }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// ---------------------------------------------------------------------------
// TextDrawing
// ---------------------------------------------------------------------------

impl Drawing for TextDrawing {
    fn id(&self) -> &DrawingId { &self.id }
    fn move_by(&mut self, delta: ChartPoint) {
        self.position.timestamp = self.position.timestamp.saturating_add(delta.timestamp);
        self.position.price += delta.price;
    }
    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::from_point(self.position)
    }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// ---------------------------------------------------------------------------
// ImageDrawing
// ---------------------------------------------------------------------------

impl Drawing for ImageDrawing {
    fn id(&self) -> &DrawingId { &self.id }
    fn move_by(&mut self, delta: ChartPoint) {
        self.position.timestamp = self.position.timestamp.saturating_add(delta.timestamp);
        self.position.price += delta.price;
    }
    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::from_point(self.position)
    }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// ---------------------------------------------------------------------------
// LabelDrawing
// ---------------------------------------------------------------------------

impl Drawing for LabelDrawing {
    fn id(&self) -> &DrawingId { &self.id }
    fn move_by(&mut self, delta: ChartPoint) {
        self.position.timestamp = self.position.timestamp.saturating_add(delta.timestamp);
        self.position.price += delta.price;
    }
    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::from_point(self.position)
    }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// ---------------------------------------------------------------------------
// HorizontalLine
// ---------------------------------------------------------------------------

impl Drawing for HorizontalLine {
    fn id(&self) -> &DrawingId { &self.id }
    fn move_by(&mut self, delta: ChartPoint) {
        self.price += delta.price;
    }
    fn bounds(&self) -> DrawingBounds {
        // Full-width line at this price level
        DrawingBounds::new(0, u64::MAX, self.price, self.price)
    }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// ---------------------------------------------------------------------------
// VerticalLine
// ---------------------------------------------------------------------------

impl Drawing for VerticalLine {
    fn id(&self) -> &DrawingId { &self.id }
    fn move_by(&mut self, delta: ChartPoint) {
        self.timestamp = self.timestamp.saturating_add(delta.timestamp);
    }
    fn bounds(&self) -> DrawingBounds {
        // Full-height line at this timestamp
        DrawingBounds::new(self.timestamp, self.timestamp, f64::MIN, f64::MAX)
    }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// ---------------------------------------------------------------------------
// Rectangle
// ---------------------------------------------------------------------------

impl Drawing for Rectangle {
    fn id(&self) -> &DrawingId { &self.id }
    fn move_by(&mut self, delta: ChartPoint) {
        self.top_left.timestamp = self.top_left.timestamp.saturating_add(delta.timestamp);
        self.top_left.price += delta.price;
        self.bottom_right.timestamp = self.bottom_right.timestamp.saturating_add(delta.timestamp);
        self.bottom_right.price += delta.price;
    }
    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::from_points(self.top_left, self.bottom_right)
    }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// ---------------------------------------------------------------------------
// FibonacciRetracement
// ---------------------------------------------------------------------------

impl Drawing for FibonacciRetracement {
    fn id(&self) -> &DrawingId { &self.id }
    fn move_by(&mut self, delta: ChartPoint) {
        self.start.timestamp = self.start.timestamp.saturating_add(delta.timestamp);
        self.start.price += delta.price;
        self.end.timestamp = self.end.timestamp.saturating_add(delta.timestamp);
        self.end.price += delta.price;
    }
    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::from_points(self.start, self.end)
    }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// ---------------------------------------------------------------------------
// FibonacciExtension
// ---------------------------------------------------------------------------

impl Drawing for FibonacciExtension {
    fn id(&self) -> &DrawingId { &self.id }
    fn move_by(&mut self, delta: ChartPoint) {
        self.point_a.timestamp = self.point_a.timestamp.saturating_add(delta.timestamp);
        self.point_a.price += delta.price;
        self.point_b.timestamp = self.point_b.timestamp.saturating_add(delta.timestamp);
        self.point_b.price += delta.price;
        self.point_c.timestamp = self.point_c.timestamp.saturating_add(delta.timestamp);
        self.point_c.price += delta.price;
    }
    fn bounds(&self) -> DrawingBounds {
        bounds_from_points(&[self.point_a, self.point_b, self.point_c])
    }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// ---------------------------------------------------------------------------
// Pitchfork
// ---------------------------------------------------------------------------

impl Drawing for Pitchfork {
    fn id(&self) -> &DrawingId { &self.id }
    fn move_by(&mut self, delta: ChartPoint) {
        self.point_a.timestamp = self.point_a.timestamp.saturating_add(delta.timestamp);
        self.point_a.price += delta.price;
        self.point_b.timestamp = self.point_b.timestamp.saturating_add(delta.timestamp);
        self.point_b.price += delta.price;
        self.point_c.timestamp = self.point_c.timestamp.saturating_add(delta.timestamp);
        self.point_c.price += delta.price;
    }
    fn bounds(&self) -> DrawingBounds {
        bounds_from_points(&[self.point_a, self.point_b, self.point_c])
    }
    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        // Simplified: check distance to median line (A to midpoint of B,C)
        let mid = ChartPoint::new(
            (self.point_b.timestamp + self.point_c.timestamp) / 2,
            (self.point_b.price + self.point_c.price) / 2.0,
        );
        let dx = mid.timestamp as f64 - self.point_a.timestamp as f64;
        let dy = mid.price - self.point_a.price;
        let len_sq = dx * dx + dy * dy;
        if len_sq < f64::EPSILON {
            return default_aabb_hit_test(&self.bounds(), point, tolerance);
        }
        let t = ((point.timestamp as f64 - self.point_a.timestamp as f64) * dx
            + (point.price - self.point_a.price) * dy)
            / len_sq;
        let t_clamped = t.clamp(0.0, 1.0);
        let proj_x = self.point_a.timestamp as f64 + t_clamped * dx;
        let proj_y = self.point_a.price + t_clamped * dy;
        let dist = ((point.timestamp as f64 - proj_x).powi(2)
            + (point.price - proj_y).powi(2))
        .sqrt();
        if dist <= tolerance as f64 {
            HitResult::Body
        } else {
            HitResult::Miss
        }
    }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// ---------------------------------------------------------------------------
// Ellipse
// ---------------------------------------------------------------------------

impl Drawing for Ellipse {
    fn id(&self) -> &DrawingId { &self.id }
    fn move_by(&mut self, delta: ChartPoint) {
        self.center.timestamp = self.center.timestamp.saturating_add(delta.timestamp);
        self.center.price += delta.price;
    }
    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::new(
            self.center.timestamp.saturating_sub(self.radius_x as u64),
            self.center.timestamp + self.radius_x as u64,
            self.center.price - self.radius_y,
            self.center.price + self.radius_y,
        )
    }
    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        // Check ellipse equation with tolerance
        let dx = (point.timestamp as f64 - self.center.timestamp as f64) / self.radius_x;
        let dy = (point.price - self.center.price) / self.radius_y;
        let d = dx * dx + dy * dy;
        // Allow hit within the ellipse boundary + tolerance
        if d <= (1.0 + tolerance as f64 / self.radius_x.max(self.radius_y)).powi(2) {
            HitResult::Body
        } else {
            HitResult::Miss
        }
    }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// ---------------------------------------------------------------------------
// Path
// ---------------------------------------------------------------------------

impl Drawing for Path {
    fn id(&self) -> &DrawingId { &self.id }
    fn move_by(&mut self, delta: ChartPoint) {
        for pt in &mut self.points {
            pt.timestamp = pt.timestamp.saturating_add(delta.timestamp);
            pt.price += delta.price;
        }
    }
    fn bounds(&self) -> DrawingBounds {
        if self.points.is_empty() {
            return DrawingBounds::new(0, 0, 0.0, 0.0);
        }
        bounds_from_points(&self.points)
    }
    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        // Check distance to each line segment
        let tol = tolerance as f64;
        for window in self.points.windows(2) {
            let dist = point_to_segment_dist(point, window[0], window[1]);
            if dist <= tol {
                return HitResult::Body;
            }
        }
        HitResult::Miss
    }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// ---------------------------------------------------------------------------
// Geometry helper
// ---------------------------------------------------------------------------

/// Distance from a point to a line segment in (timestamp, price) space.
fn point_to_segment_dist(p: ChartPoint, a: ChartPoint, b: ChartPoint) -> f64 {
    let dx = b.timestamp as f64 - a.timestamp as f64;
    let dy = b.price - a.price;
    let len_sq = dx * dx + dy * dy;
    if len_sq < f64::EPSILON {
        // Degenerate segment — return distance to endpoint
        let dpx = p.timestamp as f64 - a.timestamp as f64;
        let dpy = p.price - a.price;
        return (dpx * dpx + dpy * dpy).sqrt();
    }
    let t = ((p.timestamp as f64 - a.timestamp as f64) * dx
        + (p.price - a.price) * dy)
        / len_sq;
    let t_clamped = t.clamp(0.0, 1.0);
    let proj_x = a.timestamp as f64 + t_clamped * dx;
    let proj_y = a.price + t_clamped * dy;
    let dpx = p.timestamp as f64 - proj_x;
    let dpy = p.price - proj_y;
    (dpx * dpx + dpy * dpy).sqrt()
}
