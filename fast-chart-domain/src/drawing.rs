use crate::price_line::LineStyle;

/// Unique identifier for a drawing tool.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DrawingId(pub String);

/// A point on the chart defined by timestamp and price.
#[derive(Debug, Clone, Copy)]
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
// DrawingSet
// ---------------------------------------------------------------------------

/// A collection of drawing tools for a chart pane.
#[derive(Debug, Default)]
pub struct DrawingSet {
    trend_lines: Vec<TrendLine>,
    horizontal_lines: Vec<HorizontalLine>,
    vertical_lines: Vec<VerticalLine>,
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

    /// Total number of drawings across all types.
    pub fn len(&self) -> usize {
        self.trend_lines.len() + self.horizontal_lines.len() + self.vertical_lines.len()
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
}
