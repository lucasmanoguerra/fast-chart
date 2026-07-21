use super::{ChartPoint, Drawing, DrawingId};

// ---------------------------------------------------------------------------
// TextDrawing
// ---------------------------------------------------------------------------

/// A text label anchored to a chart point.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextDrawing {
    /// Unique identifier.
    pub id: DrawingId,
    /// Anchor point in chart coordinates.
    pub position: ChartPoint,
    /// Text content.
    pub text: String,
    /// Text color [r, g, b, a].
    pub color: [f32; 4],
    /// Font size in pixels.
    pub font_size: f32,
    /// Horizontal alignment: 0.0 = left, 0.5 = center, 1.0 = right.
    pub align_x: f32,
    /// Vertical alignment: 0.0 = top, 0.5 = center, 1.0 = bottom.
    pub align_y: f32,
    /// Whether this drawing is currently selected.
    pub selected: bool,
}

impl TextDrawing {
    /// Create a new text label at the given position.
    pub fn new(id: impl Into<String>, position: ChartPoint, text: impl Into<String>) -> Self {
        Self {
            id: DrawingId(id.into()),
            position,
            text: text.into(),
            color: [1.0, 1.0, 1.0, 1.0],
            font_size: 14.0,
            align_x: 0.0,
            align_y: 0.5,
            selected: false,
        }
    }

    /// Set the text color.
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Set the font size.
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set horizontal alignment.
    pub fn with_align_x(mut self, align: f32) -> Self {
        self.align_x = align;
        self
    }

    /// Set vertical alignment.
    pub fn with_align_y(mut self, align: f32) -> Self {
        self.align_y = align;
        self
    }
}

impl Drawing for TextDrawing {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.position.timestamp = self.position.timestamp.saturating_add(delta.timestamp);
        self.position.price += delta.price;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// ImageDrawing
// ---------------------------------------------------------------------------

/// An image annotation anchored to a chart point, with width and height in pixels.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImageDrawing {
    /// Unique identifier.
    pub id: DrawingId,
    /// Anchor position in chart coordinates.
    pub position: ChartPoint,
    /// Image data URI or file path.
    pub src: String,
    /// Width in screen pixels.
    pub width: f32,
    /// Height in screen pixels.
    pub height: f32,
    /// Opacity 0.0..1.0.
    pub opacity: f32,
    /// Whether this drawing is currently selected.
    pub selected: bool,
}

impl ImageDrawing {
    /// Create a new image annotation at the given position.
    pub fn new(id: impl Into<String>, position: ChartPoint, src: impl Into<String>) -> Self {
        Self {
            id: DrawingId(id.into()),
            position,
            src: src.into(),
            width: 100.0,
            height: 100.0,
            opacity: 1.0,
            selected: false,
        }
    }

    /// Set width in pixels.
    pub fn with_width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// Set height in pixels.
    pub fn with_height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    /// Set opacity.
    pub fn with_opacity(mut self, o: f32) -> Self {
        self.opacity = o;
        self
    }
}

impl Drawing for ImageDrawing {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.position.timestamp = self.position.timestamp.saturating_add(delta.timestamp);
        self.position.price += delta.price;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// LabelDrawing
// ---------------------------------------------------------------------------

/// A label with a background rectangle anchored to a chart point.
///
/// This is a higher-level convenience over TextDrawing + Rectangle: it renders
/// a text string inside a filled/stroked background box.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LabelDrawing {
    /// Unique identifier.
    pub id: DrawingId,
    /// Anchor position in chart coordinates.
    pub position: ChartPoint,
    /// Label text.
    pub text: String,
    /// Text color [r, g, b, a].
    pub text_color: [f32; 4],
    /// Background fill color [r, g, b, a].
    pub bg_color: [f32; 4],
    /// Border color [r, g, b, a].
    pub border_color: [f32; 4],
    /// Font size in pixels.
    pub font_size: f32,
    /// Padding in pixels.
    pub padding: f32,
    /// Whether this drawing is currently selected.
    pub selected: bool,
}

impl LabelDrawing {
    /// Create a new label at the given position.
    pub fn new(id: impl Into<String>, position: ChartPoint, text: impl Into<String>) -> Self {
        Self {
            id: DrawingId(id.into()),
            position,
            text: text.into(),
            text_color: [1.0, 1.0, 1.0, 1.0],
            bg_color: [0.15, 0.15, 0.15, 0.9],
            border_color: [0.4, 0.4, 0.4, 1.0],
            font_size: 12.0,
            padding: 4.0,
            selected: false,
        }
    }

    /// Set text color.
    pub fn with_text_color(mut self, c: [f32; 4]) -> Self {
        self.text_color = c;
        self
    }

    /// Set background color.
    pub fn with_bg_color(mut self, c: [f32; 4]) -> Self {
        self.bg_color = c;
        self
    }

    /// Set border color.
    pub fn with_border_color(mut self, c: [f32; 4]) -> Self {
        self.border_color = c;
        self
    }

    /// Set font size.
    pub fn with_font_size(mut self, s: f32) -> Self {
        self.font_size = s;
        self
    }
}

impl Drawing for LabelDrawing {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.position.timestamp = self.position.timestamp.saturating_add(delta.timestamp);
        self.position.price += delta.price;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
