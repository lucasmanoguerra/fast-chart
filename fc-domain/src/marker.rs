use crate::price_scale::PriceScaleId;

/// Position of a marker relative to the candle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum MarkerPosition {
    /// Above the candle (typically for sell signals).
    #[default]
    AboveBar,
    /// Below the candle (typically for buy signals).
    BelowBar,
    /// At the candle's close price.
    AtPrice,
}


/// Shape of a marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum MarkerShape {
    #[default]
    Circle,
    Square,
    ArrowUp,
    ArrowDown,
    Triangle,
}


/// A point annotation at a specific timestamp.
///
/// # Examples
///
/// ```
/// use fc_domain::{Marker, MarkerShape, MarkerPosition};
///
/// let marker = Marker::new("buy-1", 1000, 105.0)
///     .with_shape(MarkerShape::ArrowUp)
///     .with_position(MarkerPosition::BelowBar)
///     .with_color([0.0, 1.0, 0.0, 1.0]);
///
/// assert_eq!(marker.timestamp, 1000);
/// assert_eq!(marker.price, 105.0);
/// assert_eq!(marker.shape, MarkerShape::ArrowUp);
/// ```
#[derive(Debug, Clone)]
pub struct Marker {
    /// Unique identifier.
    pub id: MarkerId,
    /// Timestamp of the marker.
    pub timestamp: u64,
    /// Price level (used for AtPrice position).
    pub price: f64,
    /// Position relative to the candle.
    pub position: MarkerPosition,
    /// Shape of the marker.
    pub shape: MarkerShape,
    /// Color [r, g, b, a].
    pub color: [f32; 4],
    /// Size in pixels.
    pub size: f32,
    /// Optional text label.
    pub label: Option<String>,
    /// The price scale this marker belongs to.
    pub scale_id: PriceScaleId,
}

/// Unique identifier for a marker.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MarkerId(pub String);

impl Marker {
    /// Create a new marker at the given timestamp and price.
    pub fn new(id: impl Into<String>, timestamp: u64, price: f64) -> Self {
        Self {
            id: MarkerId(id.into()),
            timestamp,
            price,
            position: MarkerPosition::AboveBar,
            shape: MarkerShape::Circle,
            color: [1.0, 0.0, 0.0, 1.0],
            size: 8.0,
            label: None,
            scale_id: PriceScaleId::Right,
        }
    }

    /// Set position.
    pub fn with_position(mut self, position: MarkerPosition) -> Self {
        self.position = position;
        self
    }

    /// Set shape.
    pub fn with_shape(mut self, shape: MarkerShape) -> Self {
        self.shape = shape;
        self
    }

    /// Set color.
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Set size.
    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set price scale.
    pub fn with_scale(mut self, scale_id: PriceScaleId) -> Self {
        self.scale_id = scale_id;
        self
    }
}

/// A collection of markers for a pane.
#[derive(Debug, Default)]
pub struct MarkerSet {
    markers: Vec<Marker>,
}

impl MarkerSet {
    pub fn new() -> Self {
        Self {
            markers: Vec::new(),
        }
    }

    /// Add a marker.
    pub fn add(&mut self, marker: Marker) {
        self.markers.push(marker);
    }

    /// Remove a marker by ID.
    pub fn remove(&mut self, id: &MarkerId) -> bool {
        if let Some(pos) = self.markers.iter().position(|m| m.id == *id) {
            self.markers.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get a marker by ID.
    pub fn get(&self, id: &MarkerId) -> Option<&Marker> {
        self.markers.iter().find(|m| m.id == *id)
    }

    /// Get all markers.
    pub fn all(&self) -> &[Marker] {
        &self.markers
    }

    /// Get markers for a specific timestamp range.
    pub fn in_range(&self, start: u64, end: u64) -> Vec<&Marker> {
        self.markers
            .iter()
            .filter(|m| m.timestamp >= start && m.timestamp <= end)
            .collect()
    }

    /// Get markers for a specific scale.
    pub fn for_scale(&self, scale_id: &PriceScaleId) -> Vec<&Marker> {
        self.markers
            .iter()
            .filter(|m| m.scale_id == *scale_id)
            .collect()
    }

    /// Number of markers.
    pub fn len(&self) -> usize {
        self.markers.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.markers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Clasificación: determinística — verifica marker_new
    #[test]
    fn marker_new() {
        let m = Marker::new("buy", 1000, 105.0);
        assert_eq!(m.id, MarkerId("buy".to_string()));
        assert_eq!(m.timestamp, 1000);
        assert_eq!(m.price, 105.0);
        assert_eq!(m.position, MarkerPosition::AboveBar);
    }

    // Clasificación: determinística — verifica que build() produce tema completo sin NaN
    #[test]
    fn marker_builder() {
        let m = Marker::new("sell", 2000, 110.0)
            .with_position(MarkerPosition::BelowBar)
            .with_shape(MarkerShape::ArrowDown)
            .with_color([0.0, 1.0, 0.0, 1.0])
            .with_size(12.0)
            .with_label("Sell")
            .with_scale(PriceScaleId::Left);

        assert_eq!(m.position, MarkerPosition::BelowBar);
        assert_eq!(m.shape, MarkerShape::ArrowDown);
        assert_eq!(m.color, [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(m.size, 12.0);
        assert_eq!(m.label, Some("Sell".to_string()));
        assert_eq!(m.scale_id, PriceScaleId::Left);
    }

    // Clasificación: determinística — verifica marker_set_add_remove
    #[test]
    fn marker_set_add_remove() {
        let mut set = MarkerSet::new();
        assert!(set.is_empty());

        set.add(Marker::new("a", 100, 100.0));
        set.add(Marker::new("b", 200, 110.0));
        assert_eq!(set.len(), 2);

        assert!(set.remove(&MarkerId("a".to_string())));
        assert_eq!(set.len(), 1);
        assert!(set.get(&MarkerId("a".to_string())).is_none());
    }

    // Clasificación: determinística — verifica marker_set_in_range
    #[test]
    fn marker_set_in_range() {
        let mut set = MarkerSet::new();
        set.add(Marker::new("a", 100, 100.0));
        set.add(Marker::new("b", 200, 110.0));
        set.add(Marker::new("c", 300, 120.0));

        let range = set.in_range(150, 250);
        assert_eq!(range.len(), 1);
        assert_eq!(range[0].id, MarkerId("b".to_string()));
    }
}
