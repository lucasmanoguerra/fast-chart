// ---------------------------------------------------------------------------
// Common primitives shared across all drawing types
// ---------------------------------------------------------------------------

/// Unique identifier for a drawing tool.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DrawingId(pub String);

/// A point on the chart defined by timestamp and price.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
