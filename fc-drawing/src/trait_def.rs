// ---------------------------------------------------------------------------
// Drawing — unified trait for all chart drawing tools
// ---------------------------------------------------------------------------

use crate::bounds::DrawingBounds;
use crate::hit::{HitResult, default_aabb_hit_test};
use fc_domain::drawing::{ChartPoint, DrawingId};

/// A chart drawing that can be hit-tested, moved, bounded, and rendered.
///
/// Every drawing type (TrendLine, Rectangle, Arrow, etc.) implements this
/// trait so the `DrawingManager` can handle them polymorphically.
///
/// Default implementations:
/// - `hit_test`: AABB bounds check (override for complex geometry)
/// - `is_selected`: returns false
/// - `set_selected`: no-op
/// - `type_name`: std::any::type_name
///
/// Note: The `to_commands` method is NOT part of this trait because it requires
/// the render layer (`fc-render`). Instead, use `RenderableDrawing` from
/// `fc-render` to generate render commands.
pub trait Drawing: Send + Sync {
    /// Unique identifier for this drawing.
    fn id(&self) -> &DrawingId;

    /// Move this drawing by the given delta (in chart coordinates).
    fn move_by(&mut self, delta: ChartPoint);

    /// Bounding rectangle in chart coordinates (timestamp, price).
    fn bounds(&self) -> DrawingBounds;

    /// Test whether a chart point hits this drawing.
    ///
    /// Default implementation uses AABB bounds check with tolerance.
    /// Override for types with complex geometry (Ray, Path, Pitchfork).
    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        default_aabb_hit_test(&self.bounds(), point, tolerance)
    }

    /// Whether this drawing is currently selected.
    fn is_selected(&self) -> bool {
        false
    }

    /// Set the selection state.
    fn set_selected(&mut self, _selected: bool) {}

    /// Upcast to `&dyn Any` for type-safe downcasting.
    fn as_any(&self) -> &dyn std::any::Any;

    /// Upcast to `&mut dyn Any` for type-safe downcasting.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Type name for debugging.
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
