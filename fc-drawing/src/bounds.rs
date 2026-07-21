// ---------------------------------------------------------------------------
// DrawingBounds — bounding box in chart coordinates
// ---------------------------------------------------------------------------

use fc_domain::drawing::ChartPoint;

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
        Self {
            time_start,
            time_end,
            price_min,
            price_max,
        }
    }

    /// Create bounds from a single point (zero-size).
    pub fn from_point(p: ChartPoint) -> Self {
        Self {
            time_start: p.timestamp,
            time_end: p.timestamp,
            price_min: p.price,
            price_max: p.price,
        }
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

    /// Combine two bounding boxes into one that contains both.
    pub fn combine(&self, other: &DrawingBounds) -> DrawingBounds {
        DrawingBounds {
            time_start: self.time_start.min(other.time_start),
            time_end: self.time_end.max(other.time_end),
            price_min: self.price_min.min(other.price_min),
            price_max: self.price_max.max(other.price_max),
        }
    }
}
