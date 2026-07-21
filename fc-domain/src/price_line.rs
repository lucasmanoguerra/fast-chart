use crate::price_scale::PriceScaleId;

// LineStyle canonical definition lives in fc-primitives.
pub use fc_primitives::LineStyle;

/// A horizontal price line at a specific price level.
///
/// # Examples
///
/// ```
/// use fc_domain::{PriceLine, LineStyle};
/// use fc_domain::PriceScaleId;
///
/// let line = PriceLine::new("entry", 150.0)
///     .with_color([1.0, 0.0, 0.0, 1.0])
///     .with_style(LineStyle::Dashed)
///     .with_label("Entry");
///
/// assert_eq!(line.price, 150.0);
/// assert_eq!(line.style, LineStyle::Dashed);
/// ```
#[derive(Debug, Clone)]
pub struct PriceLine {
    /// Unique identifier for this price line.
    pub id: PriceLineId,
    /// The price level.
    pub price: f64,
    /// The price scale this line belongs to.
    pub scale_id: PriceScaleId,
    /// Line color [r, g, b, a].
    pub color: [f32; 4],
    /// Line width in pixels.
    pub width: f32,
    /// Line style.
    pub style: LineStyle,
    /// Optional label text.
    pub label: Option<String>,
    /// Label position.
    pub label_position: LabelPosition,
}

/// Unique identifier for a price line.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PriceLineId(pub String);

/// Position for the price line label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelPosition {
    Left,
    Right,
    Center,
}

impl Default for LabelPosition {
    fn default() -> Self {
        Self::Right
    }
}

impl PriceLine {
    /// Create a new price line at the given price.
    pub fn new(id: impl Into<String>, price: f64) -> Self {
        Self {
            id: PriceLineId(id.into()),
            price,
            scale_id: PriceScaleId::Right,
            color: [0.5, 0.5, 0.5, 0.8],
            width: 1.0,
            style: LineStyle::Solid,
            label: None,
            label_position: LabelPosition::Right,
        }
    }

    /// Set the price scale.
    pub fn with_scale(mut self, scale_id: PriceScaleId) -> Self {
        self.scale_id = scale_id;
        self
    }

    /// Set the color.
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

    /// Set a label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set label position.
    pub fn with_label_position(mut self, position: LabelPosition) -> Self {
        self.label_position = position;
        self
    }
}

/// A collection of price lines for a pane.
#[derive(Debug, Default)]
pub struct PriceLineSet {
    lines: Vec<PriceLine>,
}

impl PriceLineSet {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    /// Add a price line.
    pub fn add(&mut self, line: PriceLine) {
        self.lines.push(line);
    }

    /// Remove a price line by ID.
    pub fn remove(&mut self, id: &PriceLineId) -> bool {
        if let Some(pos) = self.lines.iter().position(|l| l.id == *id) {
            self.lines.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get a price line by ID.
    pub fn get(&self, id: &PriceLineId) -> Option<&PriceLine> {
        self.lines.iter().find(|l| l.id == *id)
    }

    /// Get all price lines.
    pub fn all(&self) -> &[PriceLine] {
        &self.lines
    }

    /// Get all price lines for a specific scale.
    pub fn for_scale(&self, scale_id: &PriceScaleId) -> Vec<&PriceLine> {
        self.lines
            .iter()
            .filter(|l| l.scale_id == *scale_id)
            .collect()
    }

    /// Number of price lines.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_line_new() {
        let line = PriceLine::new("support", 100.0);
        assert_eq!(line.id, PriceLineId("support".to_string()));
        assert_eq!(line.price, 100.0);
        assert_eq!(line.scale_id, PriceScaleId::Right);
    }

    #[test]
    fn price_line_builder() {
        let line = PriceLine::new("resistance", 200.0)
            .with_scale(PriceScaleId::Left)
            .with_color([1.0, 0.0, 0.0, 1.0])
            .with_width(2.0)
            .with_style(LineStyle::Dashed)
            .with_label("R1")
            .with_label_position(LabelPosition::Left);

        assert_eq!(line.scale_id, PriceScaleId::Left);
        assert_eq!(line.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(line.width, 2.0);
        assert_eq!(line.style, LineStyle::Dashed);
        assert_eq!(line.label, Some("R1".to_string()));
        assert_eq!(line.label_position, LabelPosition::Left);
    }

    #[test]
    fn price_line_set_add_remove() {
        let mut set = PriceLineSet::new();
        assert!(set.is_empty());

        set.add(PriceLine::new("a", 100.0));
        set.add(PriceLine::new("b", 200.0));
        assert_eq!(set.len(), 2);

        assert!(set.remove(&PriceLineId("a".to_string())));
        assert_eq!(set.len(), 1);
        assert!(set.get(&PriceLineId("a".to_string())).is_none());
    }

    #[test]
    fn price_line_set_for_scale() {
        let mut set = PriceLineSet::new();
        set.add(PriceLine::new("left1", 100.0).with_scale(PriceScaleId::Left));
        set.add(PriceLine::new("right1", 200.0).with_scale(PriceScaleId::Right));
        set.add(PriceLine::new("left2", 150.0).with_scale(PriceScaleId::Left));

        let left_lines = set.for_scale(&PriceScaleId::Left);
        assert_eq!(left_lines.len(), 2);

        let right_lines = set.for_scale(&PriceScaleId::Right);
        assert_eq!(right_lines.len(), 1);
    }
}
