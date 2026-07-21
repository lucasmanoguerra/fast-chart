// ---------------------------------------------------------------------------
// DrawingManager — unified CRUD + hit-test + rendering for all drawing types
// ---------------------------------------------------------------------------

use fc_domain::drawing::{
    ChartPoint, DrawingId, DrawingSet, TrendLine, Arrow, Ray, Segment, TextDrawing,
    ImageDrawing, LabelDrawing, HorizontalLine, VerticalLine, Rectangle,
    FibonacciRetracement, FibonacciExtension, Pitchfork, Ellipse, Path,
};

use crate::commands::DrawCommand;
use crate::context::RenderContext;
use crate::renderable_drawing::RenderableDrawing;
use fc_drawing::{Drawing, DrawingBounds, HitResult};

/// Invoke `$macro` with all 15 concrete drawing types.
macro_rules! all_drawing_types {
    ($macro:ident) => {
        $macro!(
            TrendLine, Arrow, Ray, Segment, TextDrawing,
            ImageDrawing, LabelDrawing, HorizontalLine, VerticalLine,
            Rectangle, FibonacciRetracement, FibonacciExtension,
            Pitchfork, Ellipse, Path,
        );
    };
}

// ---------------------------------------------------------------------------
// DrawingManager
// ---------------------------------------------------------------------------

/// A unified manager for all drawing types in a pane.
///
/// `DrawingManager` wraps a `DrawingSet` and provides:
/// - Unified hit testing across all types
/// - Selection management (single select / deselect all)
/// - Unified rendering to `Vec<DrawCommand>`
/// - Move operations
/// - Bounds computation for all drawings
pub struct DrawingManager {
    drawings: DrawingSet,
    selected_id: Option<DrawingId>,
}

impl DrawingManager {
    /// Create a new empty drawing manager.
    pub fn new() -> Self {
        Self {
            drawings: DrawingSet::new(),
            selected_id: None,
        }
    }

    /// Create from an existing `DrawingSet`.
    pub fn from_set(drawings: DrawingSet) -> Self {
        Self {
            drawings,
            selected_id: None,
        }
    }

    /// Get a reference to the underlying `DrawingSet`.
    pub fn set(&self) -> &DrawingSet {
        &self.drawings
    }

    /// Get a mutable reference to the underlying `DrawingSet`.
    pub fn set_mut(&mut self) -> &mut DrawingSet {
        &mut self.drawings
    }

    /// Total number of drawings across all types.
    pub fn len(&self) -> usize {
        self.drawings.len()
    }

    /// Whether the manager has no drawings.
    pub fn is_empty(&self) -> bool {
        self.drawings.is_empty()
    }

    // -- Selection --

    /// Get the currently selected drawing ID, if any.
    pub fn selected_id(&self) -> Option<&DrawingId> {
        self.selected_id.as_ref()
    }

    /// Deselect the current drawing.
    pub fn deselect_all(&mut self) {
        self.selected_id = None;
    }

    /// Remove a drawing by ID (any type). Returns true if found and removed.
    pub fn remove(&mut self, id: &DrawingId) -> bool {
        if self.selected_id.as_ref() == Some(id) {
            self.selected_id = None;
        }
        self.drawings.remove(id)
    }

    /// Move the selected drawing by a delta. Returns true if moved.
    pub fn move_selected(&mut self, delta: ChartPoint) -> bool {
        if let Some(ref id) = self.selected_id {
            self.drawings.move_drawing(id, delta);
            true
        } else {
            false
        }
    }

    // -- Unified hit test --

    /// Hit-test all drawings at a point. Returns (id, HitResult) for the first hit.
    pub fn hit_test(&self, point: ChartPoint, tolerance: f32) -> Option<(DrawingId, HitResult)> {
        for item in self.drawings.all_raw() {
            macro_rules! try_hit {
                ($($ty:ty),+ $(,)?) => {
                    $(
                        if let Some(t) = item.as_any().downcast_ref::<$ty>() {
                            let result = Drawing::hit_test(t, point, tolerance);
                            if result != HitResult::Miss {
                                return Some((Drawing::id(t).clone(), result));
                            }
                        }
                    )+
                };
            }
            all_drawing_types!(try_hit);
        }
        None
    }

    /// Hit-test and select the first hit drawing. Returns the hit ID if found.
    pub fn hit_test_and_select(&mut self, point: ChartPoint, tolerance: f32) -> Option<DrawingId> {
        if let Some((id, _)) = self.hit_test(point, tolerance) {
            self.selected_id = Some(id.clone());
            Some(id)
        } else {
            self.selected_id = None;
            None
        }
    }

    // -- Unified bounds --

    /// Compute combined bounds of all drawings. Returns None if empty.
    pub fn bounds(&self) -> Option<DrawingBounds> {
        let mut result: Option<DrawingBounds> = None;

        for item in self.drawings.all_raw() {
            macro_rules! try_bounds {
                ($($ty:ty),+ $(,)?) => {
                    $(
                        if let Some(t) = item.as_any().downcast_ref::<$ty>() {
                            let b = Drawing::bounds(t);
                            result = Some(match result {
                                Some(r) => r.combine(&b),
                                None => b,
                            });
                        }
                    )+
                };
            }
            all_drawing_types!(try_bounds);
        }

        result
    }

    // -- Unified render --

    /// Render all drawings to a sorted list of draw commands.
    pub fn render(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let mut cmds = Vec::new();

        for item in self.drawings.all_raw() {
            macro_rules! try_render {
                ($($ty:ty),+ $(,)?) => {
                    $(
                        if let Some(t) = item.as_any().downcast_ref::<$ty>() {
                            cmds.extend(RenderableDrawing::to_commands(t, ctx));
                        }
                    )+
                };
            }
            all_drawing_types!(try_render);
        }

        // Sort by z_index
        cmds.sort_by_key(|c| c.z_index());
        cmds
    }
}

impl Default for DrawingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fc_domain::drawing::{
        TrendLine, HorizontalLine, VerticalLine, FibonacciRetracement,
        FibonacciExtension, Pitchfork, Arrow,
    };

    #[test]
    fn new_manager_is_empty() {
        let mgr = DrawingManager::new();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
        assert!(mgr.selected_id().is_none());
    }

    #[test]
    fn add_and_remove_trend_line() {
        let mut mgr = DrawingManager::new();
        mgr.set_mut().add_trend_line(TrendLine::new(
            "tl1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        ));
        assert_eq!(mgr.len(), 1);

        let removed = mgr.remove(&DrawingId("tl1".to_string()));
        assert!(removed);
        assert!(mgr.is_empty());
    }

    #[test]
    fn hit_test_finds_trend_line() {
        let mut mgr = DrawingManager::new();
        mgr.set_mut().add_trend_line(TrendLine::new(
            "tl1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        ));

        let hit = mgr.hit_test(ChartPoint::new(1500, 150.0), 50.0);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().0, DrawingId("tl1".to_string()));
    }

    #[test]
    fn hit_test_returns_none_on_empty() {
        let mgr = DrawingManager::new();
        assert!(mgr.hit_test(ChartPoint::new(1500, 150.0), 50.0).is_none());
    }

    #[test]
    fn select_and_deselect() {
        let mut mgr = DrawingManager::new();
        mgr.set_mut().add_trend_line(TrendLine::new(
            "tl1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        ));

        let id = mgr.hit_test_and_select(ChartPoint::new(1500, 150.0), 50.0);
        assert!(id.is_some());
        assert!(mgr.selected_id().is_some());

        mgr.deselect_all();
        assert!(mgr.selected_id().is_none());
    }

    #[test]
    fn render_produces_commands() {
        let mut mgr = DrawingManager::new();
        mgr.set_mut().add_trend_line(TrendLine::new(
            "tl1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        ));

        use crate::context::RenderContext;
        use crate::coordinates::CoordinatePipeline;

        let pipeline =
            CoordinatePipeline::new((0.0, 3000.0), (50.0, 200.0), 0.0, 0.0, 800.0, 400.0, 1.0);
        let ctx = RenderContext::from_pipeline(
            pipeline,
            fc_primitives::Rect::new(0.0, 0.0, 800.0, 400.0),
            0,
        );

        let cmds = mgr.render(&ctx);
        assert_eq!(cmds.len(), 1);
    }

    #[test]
    fn combined_bounds() {
        let mut mgr = DrawingManager::new();
        mgr.set_mut().add_trend_line(TrendLine::new(
            "tl1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        ));
        mgr.set_mut().add_arrow(Arrow::new(
            "a1",
            ChartPoint::new(3000, 300.0),
            ChartPoint::new(4000, 400.0),
        ));

        let bounds = mgr.bounds();
        assert!(bounds.is_some());
        let b = bounds.unwrap();
        assert_eq!(b.time_start, 1000);
        assert_eq!(b.time_end, 4000);
    }

    #[test]
    fn move_selected_drawing() {
        let mut mgr = DrawingManager::new();
        mgr.set_mut().add_trend_line(TrendLine::new(
            "tl1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        ));

        mgr.hit_test_and_select(ChartPoint::new(1500, 150.0), 50.0);
        let moved = mgr.move_selected(ChartPoint::new(100, 10.0));
        assert!(moved);

        let tl = mgr
            .set()
            .get_trend_line(&DrawingId("tl1".to_string()))
            .unwrap();
        assert_eq!(tl.start.timestamp, 1100);
        assert!((tl.start.price - 110.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hit_test_miss() {
        let mgr = DrawingManager::new();
        assert!(mgr.hit_test(ChartPoint::new(1500, 150.0), 50.0).is_none());
    }

    #[test]
    fn select_deselect_cycle() {
        let mut mgr = DrawingManager::new();
        mgr.set_mut().add_trend_line(TrendLine::new(
            "tl1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        ));

        let id = mgr.hit_test_and_select(ChartPoint::new(1500, 150.0), 50.0);
        assert!(id.is_some());
        assert_eq!(mgr.selected_id(), Some(&id.unwrap()));

        mgr.deselect_all();
        assert!(mgr.selected_id().is_none());
    }

    #[test]
    fn move_selected() {
        let mut mgr = DrawingManager::new();
        mgr.set_mut().add_trend_line(TrendLine::new(
            "tl1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        ));
        mgr.set_mut().add_arrow(Arrow::new(
            "a1",
            ChartPoint::new(3000, 300.0),
            ChartPoint::new(4000, 400.0),
        ));

        mgr.hit_test_and_select(ChartPoint::new(1500, 150.0), 50.0);
        let moved = mgr.move_selected(ChartPoint::new(100, 10.0));
        assert!(moved);

        let tl = mgr
            .set()
            .get_trend_line(&DrawingId("tl1".to_string()))
            .unwrap();
        assert_eq!(tl.start.timestamp, 1100);

        let ar = mgr.set().get_arrow(&DrawingId("a1".to_string())).unwrap();
        assert_eq!(ar.start.timestamp, 3000);
    }

    #[test]
    fn bounds_single() {
        let mut mgr = DrawingManager::new();
        mgr.set_mut().add_trend_line(TrendLine::new(
            "tl1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        ));

        let bounds = mgr.bounds();
        assert!(bounds.is_some());
        let b = bounds.unwrap();
        assert_eq!(b.time_start, 1000);
        assert_eq!(b.time_end, 2000);
    }

    #[test]
    fn render_single() {
        let mut mgr = DrawingManager::new();
        mgr.set_mut().add_trend_line(TrendLine::new(
            "tl1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        ));

        use crate::context::RenderContext;
        use crate::coordinates::CoordinatePipeline;

        let pipeline =
            CoordinatePipeline::new((0.0, 3000.0), (50.0, 250.0), 0.0, 0.0, 800.0, 400.0, 1.0);
        let ctx = RenderContext::from_pipeline(
            pipeline,
            fc_primitives::Rect::new(0.0, 0.0, 800.0, 400.0),
            0,
        );

        let cmds = mgr.render(&ctx);
        assert_eq!(cmds.len(), 1);
    }
}
