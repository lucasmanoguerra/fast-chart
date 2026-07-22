// ---------------------------------------------------------------------------
// Drawing — trait, DrawingSet, and re-exports
// ---------------------------------------------------------------------------

mod common;
mod fibonacci;
mod labels;
mod lines;
mod shapes;

use std::fmt;

// Re-exports — preserves exact same public API.
pub use common::{ChartPoint, DrawingId};
pub use fibonacci::{FibonacciExtension, FibonacciRetracement, Pitchfork};
pub use labels::{ImageDrawing, LabelDrawing, TextDrawing};
pub use lines::{Arrow, HorizontalLine, Path, Ray, Segment, TrendLine, VerticalLine};
pub use shapes::{Ellipse, Rectangle};

/// A drawing tool that can be identified and moved.
///
/// Every concrete drawing type (TrendLine, Rectangle, Arrow, etc.) implements
/// this trait so `DrawingSet` can handle them polymorphically.
pub trait Drawing: fmt::Debug + Send + Sync {
    /// Unique identifier for this drawing.
    fn id(&self) -> &DrawingId;

    /// Move this drawing by the given delta (in chart coordinates).
    fn move_by(&mut self, delta: ChartPoint);

    /// Upcast to `&dyn Any` for type-safe downcasting.
    fn as_any(&self) -> &dyn std::any::Any;

    /// Upcast to `&mut dyn Any` for type-safe downcasting.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

// ---------------------------------------------------------------------------
// DrawingSet
// ---------------------------------------------------------------------------

/// A collection of drawing tools for a chart pane.
pub struct DrawingSet {
    drawings: Vec<Box<dyn Drawing>>,
}

impl DrawingSet {
    /// Create an empty drawing set.
    pub fn new() -> Self {
        Self {
            drawings: Vec::new(),
        }
    }

    // -- Typed add methods --------------------------------------------------

    /// Add a trend line.
    pub fn add_trend_line(&mut self, line: TrendLine) {
        self.drawings.push(Box::new(line));
    }

    /// Add an arrow.
    pub fn add_arrow(&mut self, arrow: Arrow) {
        self.drawings.push(Box::new(arrow));
    }

    /// Add a ray.
    pub fn add_ray(&mut self, ray: Ray) {
        self.drawings.push(Box::new(ray));
    }

    /// Add a segment.
    pub fn add_segment(&mut self, segment: Segment) {
        self.drawings.push(Box::new(segment));
    }

    /// Add a text drawing.
    pub fn add_text_drawing(&mut self, text: TextDrawing) {
        self.drawings.push(Box::new(text));
    }

    /// Add an image drawing.
    pub fn add_image_drawing(&mut self, img: ImageDrawing) {
        self.drawings.push(Box::new(img));
    }

    /// Add a label drawing.
    pub fn add_label_drawing(&mut self, label: LabelDrawing) {
        self.drawings.push(Box::new(label));
    }

    /// Add a horizontal line.
    pub fn add_horizontal_line(&mut self, line: HorizontalLine) {
        self.drawings.push(Box::new(line));
    }

    /// Add a vertical line.
    pub fn add_vertical_line(&mut self, line: VerticalLine) {
        self.drawings.push(Box::new(line));
    }

    /// Add a rectangle.
    pub fn add_rectangle(&mut self, rect: Rectangle) {
        self.drawings.push(Box::new(rect));
    }

    /// Add a Fibonacci retracement.
    pub fn add_fibonacci_retracement(&mut self, fib: FibonacciRetracement) {
        self.drawings.push(Box::new(fib));
    }

    /// Add a Fibonacci extension.
    pub fn add_fibonacci_extension(&mut self, ext: FibonacciExtension) {
        self.drawings.push(Box::new(ext));
    }

    /// Add a pitchfork.
    pub fn add_pitchfork(&mut self, pf: Pitchfork) {
        self.drawings.push(Box::new(pf));
    }

    /// Add an ellipse.
    pub fn add_ellipse(&mut self, ellipse: Ellipse) {
        self.drawings.push(Box::new(ellipse));
    }

    /// Add a path.
    pub fn add_path(&mut self, path: Path) {
        self.drawings.push(Box::new(path));
    }

    // -- Typed get methods (by ID, downcast) -------------------------------

    /// Get a trend line by ID.
    pub fn get_trend_line(&self, id: &DrawingId) -> Option<&TrendLine> {
        self.drawings
            .iter()
            .find_map(|d| d.as_any().downcast_ref::<TrendLine>())
            .filter(|t| t.id == *id)
    }

    /// Get an arrow by ID.
    pub fn get_arrow(&self, id: &DrawingId) -> Option<&Arrow> {
        self.drawings
            .iter()
            .find_map(|d| d.as_any().downcast_ref::<Arrow>())
            .filter(|a| a.id == *id)
    }

    /// Get a ray by ID.
    pub fn get_ray(&self, id: &DrawingId) -> Option<&Ray> {
        self.drawings
            .iter()
            .find_map(|d| d.as_any().downcast_ref::<Ray>())
            .filter(|r| r.id == *id)
    }

    /// Get a segment by ID.
    pub fn get_segment(&self, id: &DrawingId) -> Option<&Segment> {
        self.drawings
            .iter()
            .find_map(|d| d.as_any().downcast_ref::<Segment>())
            .filter(|s| s.id == *id)
    }

    /// Get a text drawing by ID.
    pub fn get_text_drawing(&self, id: &DrawingId) -> Option<&TextDrawing> {
        self.drawings
            .iter()
            .find_map(|d| d.as_any().downcast_ref::<TextDrawing>())
            .filter(|t| t.id == *id)
    }

    /// Get an image drawing by ID.
    pub fn get_image_drawing(&self, id: &DrawingId) -> Option<&ImageDrawing> {
        self.drawings
            .iter()
            .find_map(|d| d.as_any().downcast_ref::<ImageDrawing>())
            .filter(|i| i.id == *id)
    }

    /// Get a label drawing by ID.
    pub fn get_label_drawing(&self, id: &DrawingId) -> Option<&LabelDrawing> {
        self.drawings
            .iter()
            .find_map(|d| d.as_any().downcast_ref::<LabelDrawing>())
            .filter(|l| l.id == *id)
    }

    /// Get a horizontal line by ID.
    pub fn get_horizontal_line(&self, id: &DrawingId) -> Option<&HorizontalLine> {
        self.drawings
            .iter()
            .find_map(|d| d.as_any().downcast_ref::<HorizontalLine>())
            .filter(|h| h.id == *id)
    }

    /// Get a vertical line by ID.
    pub fn get_vertical_line(&self, id: &DrawingId) -> Option<&VerticalLine> {
        self.drawings
            .iter()
            .find_map(|d| d.as_any().downcast_ref::<VerticalLine>())
            .filter(|v| v.id == *id)
    }

    /// Get a rectangle by ID.
    pub fn get_rectangle(&self, id: &DrawingId) -> Option<&Rectangle> {
        self.drawings
            .iter()
            .find_map(|d| d.as_any().downcast_ref::<Rectangle>())
            .filter(|r| r.id == *id)
    }

    /// Get a Fibonacci retracement by ID.
    pub fn get_fibonacci_retracement(&self, id: &DrawingId) -> Option<&FibonacciRetracement> {
        self.drawings
            .iter()
            .find_map(|d| d.as_any().downcast_ref::<FibonacciRetracement>())
            .filter(|f| f.id == *id)
    }

    /// Get a Fibonacci extension by ID.
    pub fn get_fibonacci_extension(&self, id: &DrawingId) -> Option<&FibonacciExtension> {
        self.drawings
            .iter()
            .find_map(|d| d.as_any().downcast_ref::<FibonacciExtension>())
            .filter(|e| e.id == *id)
    }

    /// Get a pitchfork by ID.
    pub fn get_pitchfork(&self, id: &DrawingId) -> Option<&Pitchfork> {
        self.drawings
            .iter()
            .find_map(|d| d.as_any().downcast_ref::<Pitchfork>())
            .filter(|p| p.id == *id)
    }

    /// Get an ellipse by ID.
    pub fn get_ellipse(&self, id: &DrawingId) -> Option<&Ellipse> {
        self.drawings
            .iter()
            .find_map(|d| d.as_any().downcast_ref::<Ellipse>())
            .filter(|e| e.id == *id)
    }

    /// Get a path by ID.
    pub fn get_path(&self, id: &DrawingId) -> Option<&Path> {
        self.drawings
            .iter()
            .find_map(|d| d.as_any().downcast_ref::<Path>())
            .filter(|p| p.id == *id)
    }

    // -- Typed all methods (collect via downcast) ---------------------------

    /// Get all trend lines.
    pub fn all_trend_lines(&self) -> Vec<&TrendLine> {
        self.drawings
            .iter()
            .filter_map(|d| d.as_any().downcast_ref::<TrendLine>())
            .collect()
    }

    /// Get all arrows.
    pub fn all_arrows(&self) -> Vec<&Arrow> {
        self.drawings
            .iter()
            .filter_map(|d| d.as_any().downcast_ref::<Arrow>())
            .collect()
    }

    /// Get all rays.
    pub fn all_rays(&self) -> Vec<&Ray> {
        self.drawings
            .iter()
            .filter_map(|d| d.as_any().downcast_ref::<Ray>())
            .collect()
    }

    /// Get all segments.
    pub fn all_segments(&self) -> Vec<&Segment> {
        self.drawings
            .iter()
            .filter_map(|d| d.as_any().downcast_ref::<Segment>())
            .collect()
    }

    /// Get all text drawings.
    pub fn all_text_drawings(&self) -> Vec<&TextDrawing> {
        self.drawings
            .iter()
            .filter_map(|d| d.as_any().downcast_ref::<TextDrawing>())
            .collect()
    }

    /// Get all image drawings.
    pub fn all_image_drawings(&self) -> Vec<&ImageDrawing> {
        self.drawings
            .iter()
            .filter_map(|d| d.as_any().downcast_ref::<ImageDrawing>())
            .collect()
    }

    /// Get all label drawings.
    pub fn all_label_drawings(&self) -> Vec<&LabelDrawing> {
        self.drawings
            .iter()
            .filter_map(|d| d.as_any().downcast_ref::<LabelDrawing>())
            .collect()
    }

    /// Get all horizontal lines.
    pub fn all_horizontal_lines(&self) -> Vec<&HorizontalLine> {
        self.drawings
            .iter()
            .filter_map(|d| d.as_any().downcast_ref::<HorizontalLine>())
            .collect()
    }

    /// Get all vertical lines.
    pub fn all_vertical_lines(&self) -> Vec<&VerticalLine> {
        self.drawings
            .iter()
            .filter_map(|d| d.as_any().downcast_ref::<VerticalLine>())
            .collect()
    }

    /// Get all rectangles.
    pub fn all_rectangles(&self) -> Vec<&Rectangle> {
        self.drawings
            .iter()
            .filter_map(|d| d.as_any().downcast_ref::<Rectangle>())
            .collect()
    }

    /// Get all Fibonacci retracements.
    pub fn all_fibonacci_retracements(&self) -> Vec<&FibonacciRetracement> {
        self.drawings
            .iter()
            .filter_map(|d| d.as_any().downcast_ref::<FibonacciRetracement>())
            .collect()
    }

    /// Get all Fibonacci extensions.
    pub fn all_fibonacci_extensions(&self) -> Vec<&FibonacciExtension> {
        self.drawings
            .iter()
            .filter_map(|d| d.as_any().downcast_ref::<FibonacciExtension>())
            .collect()
    }

    /// Get all pitchforks.
    pub fn all_pitchforks(&self) -> Vec<&Pitchfork> {
        self.drawings
            .iter()
            .filter_map(|d| d.as_any().downcast_ref::<Pitchfork>())
            .collect()
    }

    /// Get all ellipses.
    pub fn all_ellipses(&self) -> Vec<&Ellipse> {
        self.drawings
            .iter()
            .filter_map(|d| d.as_any().downcast_ref::<Ellipse>())
            .collect()
    }

    /// Get all paths.
    pub fn all_paths(&self) -> Vec<&Path> {
        self.drawings
            .iter()
            .filter_map(|d| d.as_any().downcast_ref::<Path>())
            .collect()
    }

    // -- Raw access ---------------------------------------------------------

    /// Get a slice of all raw drawing trait objects.
    ///
    /// Use this for single-pass iteration over all drawings regardless of type.
    pub fn all_raw(&self) -> &[Box<dyn Drawing>] {
        &self.drawings
    }

    // -- Generic operations -------------------------------------------------

    /// Remove a drawing by ID. Returns `true` if found and removed.
    pub fn remove(&mut self, id: &DrawingId) -> bool {
        if let Some(pos) = self.drawings.iter().position(|d| d.id() == id) {
            self.drawings.remove(pos);
            true
        } else {
            false
        }
    }

    /// Move a drawing by delta. Returns `true` if the drawing was found.
    pub fn move_drawing(&mut self, id: &DrawingId, delta: ChartPoint) -> bool {
        if let Some(d) = self.drawings.iter_mut().find(|d| d.id() == id) {
            d.move_by(delta);
            true
        } else {
            false
        }
    }

    /// Total number of drawings.
    pub fn len(&self) -> usize {
        self.drawings.len()
    }

    /// Check if the set contains no drawings.
    pub fn is_empty(&self) -> bool {
        self.drawings.is_empty()
    }
}

impl Default for DrawingSet {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for DrawingSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DrawingSet")
            .field("len", &self.drawings.len())
            .finish()
    }
}

#[cfg(test)]
mod tests;
