// ---------------------------------------------------------------------------
// HitResult — result of a hit-test against a drawing
// ---------------------------------------------------------------------------

use fc_domain::drawing::ChartPoint;
use crate::bounds::DrawingBounds;

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

/// Default AABB hit test: checks if point is within tolerance of bounds.
pub fn default_aabb_hit_test(
    bounds: &DrawingBounds,
    point: ChartPoint,
    tolerance: f32,
) -> HitResult {
    let tol_time = tolerance as u64;
    let tol_price = tolerance as f64;

    if point.timestamp >= bounds.time_start.saturating_sub(tol_time)
        && point.timestamp <= bounds.time_end.saturating_add(tol_time)
        && point.price >= bounds.price_min - tol_price
        && point.price <= bounds.price_max + tol_price
    {
        HitResult::Body
    } else {
        HitResult::Miss
    }
}
