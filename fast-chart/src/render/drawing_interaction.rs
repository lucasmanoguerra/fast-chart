// ---------------------------------------------------------------------------
// DrawingInteraction — interaction state machine for drawing tools
// ---------------------------------------------------------------------------

use fc_types::drawing::{ChartPoint, DrawingId};

use crate::render::drawing_manager::DrawingManager;

/// The current drawing interaction mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingMode {
    /// No drawing interaction — clicks select drawings.
    Select,
    /// Drawing a trend line (click start, click end).
    TrendLine,
    /// Drawing an arrow.
    Arrow,
    /// Drawing a ray (click start, click direction).
    Ray,
    /// Drawing a segment.
    Segment,
    /// Drawing a rectangle (click corner, drag to opposite corner).
    Rectangle,
    /// Drawing an ellipse (click center, drag radius).
    Ellipse,
    /// Drawing a horizontal line (click to place).
    HorizontalLine,
    /// Drawing a vertical line (click to place).
    VerticalLine,
    /// Drawing Fibonacci retracement (click start, click end).
    FibonacciRetracement,
    /// Drawing Fibonacci extension (click A, click B, click C).
    FibonacciExtension,
    /// Drawing a pitchfork (click A, click B, click C).
    Pitchfork,
    /// Drawing a path/polygon (click to add points, double-click or press Escape to finish).
    Path,
}

/// State of an in-progress multi-click drawing operation.
#[derive(Debug, Clone)]
pub struct DrawingPlacement {
    /// The type of drawing being placed.
    pub mode: DrawingMode,
    /// Points collected so far (for multi-click tools like Fibonacci: A, B, C).
    pub points: Vec<ChartPoint>,
    /// Optional current mouse/drag position for preview.
    pub cursor: Option<ChartPoint>,
}

/// Handles mouse/touch interactions for drawing creation and selection.
///
/// Manages the state machine: mode selection → click → place → completion.
pub struct DrawingInteraction {
    /// Current drawing mode.
    pub mode: DrawingMode,
    /// Active placement (Some when a multi-click drawing is in progress).
    pub placement: Option<DrawingPlacement>,
    /// Hit tolerance in screen pixels.
    pub tolerance: f32,
    /// Whether the mouse is currently dragging.
    pub dragging: bool,
    /// Drag start position (in chart coords).
    pub drag_start: Option<ChartPoint>,
}

impl DrawingInteraction {
    /// Create a new interaction handler in Select mode.
    pub fn new() -> Self {
        Self {
            mode: DrawingMode::Select,
            placement: None,
            tolerance: 5.0,
            dragging: false,
            drag_start: None,
        }
    }

    /// Switch to a different drawing mode, cancelling any in-progress placement.
    pub fn set_mode(&mut self, mode: DrawingMode) {
        self.mode = mode;
        self.placement = None;
        self.dragging = false;
        self.drag_start = None;
    }

    /// Whether a placement is currently in progress.
    pub fn is_placing(&self) -> bool {
        self.placement.is_some()
    }

    /// Whether the current mode requires N clicks to complete.
    fn required_clicks(mode: DrawingMode) -> Option<usize> {
        match mode {
            DrawingMode::Select => None,
            DrawingMode::TrendLine => Some(2),
            DrawingMode::Arrow => Some(2),
            DrawingMode::Ray => Some(2),
            DrawingMode::Segment => Some(2),
            DrawingMode::Rectangle => Some(2),
            DrawingMode::Ellipse => Some(2),
            DrawingMode::HorizontalLine => Some(1),
            DrawingMode::VerticalLine => Some(1),
            DrawingMode::FibonacciRetracement => Some(2),
            DrawingMode::FibonacciExtension => Some(3),
            DrawingMode::Pitchfork => Some(3),
            DrawingMode::Path => None, // variable, finished externally
        }
    }

    /// Handle a click at a chart position. Returns a `DrawingAction` if the
    /// click completes a drawing, or None if more clicks are needed.
    pub fn on_click(&mut self, point: ChartPoint, manager: &mut DrawingManager) -> Option<DrawingAction> {
        match self.mode {
            DrawingMode::Select => {
                // Hit-test and select
                manager.hit_test_and_select(point, self.tolerance);
                None
            }
            _ => {
                let required = Self::required_clicks(self.mode);

                // Start new placement or add point
                let placement = self.placement.get_or_insert_with(|| DrawingPlacement {
                    mode: self.mode,
                    points: Vec::new(),
                    cursor: None,
                });

                placement.points.push(point);

                match required {
                    Some(n) if placement.points.len() >= n => {
                        // Complete!
                        let completed = placement.points.clone();
                        let mode = placement.mode;
                        self.placement = None;
                        Some(DrawingAction::Complete { mode, points: completed })
                    }
                    _ => {
                        // More clicks needed
                        Some(DrawingAction::Intermediate {
                            points: placement.points.clone(),
                        })
                    }
                }
            }
        }
    }

    /// Handle mouse move (for drag preview or cursor tracking).
    pub fn on_move(&mut self, point: ChartPoint) {
        if let Some(ref mut placement) = self.placement {
            placement.cursor = Some(point);
        }
    }

    /// Handle mouse down (start drag).
    pub fn on_mouse_down(&mut self, point: ChartPoint) {
        self.dragging = true;
        self.drag_start = Some(point);
    }

    /// Handle mouse up (end drag — may complete a drag-to-place drawing).
    pub fn on_mouse_up(&mut self, point: ChartPoint) -> Option<DrawingAction> {
        if self.dragging {
            self.dragging = false;

            if let Some(start) = self.drag_start.take() {
                // If drag was substantial, it's a placement
                let dx = (point.timestamp as f64 - start.timestamp as f64).abs();
                let dy = (point.price - start.price).abs();
                if dx > 1.0 || dy > 1.0 {
                    // For rectangle/ellipse, two-point completion via drag
                    match self.mode {
                        DrawingMode::Rectangle | DrawingMode::Ellipse => {
                            let required = Self::required_clicks(self.mode);
                            let placement = self.placement.get_or_insert_with(|| DrawingPlacement {
                                mode: self.mode,
                                points: Vec::new(),
                                cursor: None,
                            });

                            if placement.points.is_empty() {
                                placement.points.push(start);
                            }
                            placement.points.push(point);

                            if let Some(n) = required {
                                if placement.points.len() >= n {
                                    let completed = placement.points.clone();
                                    let mode = placement.mode;
                                    self.placement = None;
                                    return Some(DrawingAction::Complete { mode, points: completed });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }

    /// Cancel the current placement and return to idle.
    pub fn cancel(&mut self) {
        self.placement = None;
        self.dragging = false;
        self.drag_start = None;
    }
}

impl Default for DrawingInteraction {
    fn default() -> Self {
        Self::new()
    }
}

/// An action produced by the interaction handler.
#[derive(Debug, Clone)]
pub enum DrawingAction {
    /// A drawing was completed with the given mode and points.
    Complete {
        mode: DrawingMode,
        points: Vec<ChartPoint>,
    },
    /// An intermediate click was registered (partial placement).
    Intermediate {
        points: Vec<ChartPoint>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_mode_no_placement() {
        let mut interaction = DrawingInteraction::new();
        let mut mgr = DrawingManager::new();

        let action = interaction.on_click(ChartPoint::new(1000, 100.0), &mut mgr);
        assert!(action.is_none());
        assert!(!interaction.is_placing());
    }

    #[test]
    fn trend_line_needs_two_clicks() {
        let mut interaction = DrawingInteraction::new();
        interaction.set_mode(DrawingMode::TrendLine);
        let mut mgr = DrawingManager::new();

        // First click — intermediate
        let action = interaction.on_click(ChartPoint::new(1000, 100.0), &mut mgr);
        assert!(matches!(action, Some(DrawingAction::Intermediate { .. })));
        assert!(interaction.is_placing());

        // Second click — complete
        let action = interaction.on_click(ChartPoint::new(2000, 200.0), &mut mgr);
        assert!(matches!(action, Some(DrawingAction::Complete { mode: DrawingMode::TrendLine, .. })));
        assert!(!interaction.is_placing());
    }

    #[test]
    fn horizontal_line_completes_in_one_click() {
        let mut interaction = DrawingInteraction::new();
        interaction.set_mode(DrawingMode::HorizontalLine);
        let mut mgr = DrawingManager::new();

        let action = interaction.on_click(ChartPoint::new(1000, 150.0), &mut mgr);
        assert!(matches!(action, Some(DrawingAction::Complete { mode: DrawingMode::HorizontalLine, points }) if points.len() == 1));
    }

    #[test]
    fn fibonacci_extension_needs_three_clicks() {
        let mut interaction = DrawingInteraction::new();
        interaction.set_mode(DrawingMode::FibonacciExtension);
        let mut mgr = DrawingManager::new();

        interaction.on_click(ChartPoint::new(1000, 100.0), &mut mgr);
        interaction.on_click(ChartPoint::new(2000, 200.0), &mut mgr);
        let action = interaction.on_click(ChartPoint::new(1500, 150.0), &mut mgr);
        assert!(matches!(action, Some(DrawingAction::Complete { mode: DrawingMode::FibonacciExtension, points }) if points.len() == 3));
    }

    #[test]
    fn cancel_resets_placement() {
        let mut interaction = DrawingInteraction::new();
        interaction.set_mode(DrawingMode::TrendLine);
        let mut mgr = DrawingManager::new();

        interaction.on_click(ChartPoint::new(1000, 100.0), &mut mgr);
        assert!(interaction.is_placing());

        interaction.cancel();
        assert!(!interaction.is_placing());
    }

    #[test]
    fn switch_mode_cancels_placement() {
        let mut interaction = DrawingInteraction::new();
        interaction.set_mode(DrawingMode::TrendLine);
        let mut mgr = DrawingManager::new();

        interaction.on_click(ChartPoint::new(1000, 100.0), &mut mgr);
        interaction.set_mode(DrawingMode::Rectangle);
        assert!(!interaction.is_placing());
        assert_eq!(interaction.mode, DrawingMode::Rectangle);
    }
}
