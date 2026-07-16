// ---------------------------------------------------------------------------
// Drawing — unified trait for all chart drawing tools
// ---------------------------------------------------------------------------

use crate::render::commands::DrawCommand;
use crate::render::context::RenderContext;
use crate::render::coordinates::WorldPoint;
use crate::render::series_renderer::Rect;
use fast_chart_domain::drawing::{ChartPoint, DrawingId};

/// Result of a hit-test against a drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitResult {
    /// No hit — the point is not near this drawing.
    Miss,
    /// Hit on the drawing body (line, shape interior, etc.).
    Body,
    /// Hit on a control point (endpoint, handle, vertex).
    ControlPoint(usize),
}

/// A chart drawing that can be hit-tested, moved, bounded, and rendered.
///
/// Every drawing type (TrendLine, Rectangle, Arrow, etc.) implements this
/// trait so the `DrawingManager` can handle them polymorphically.
pub trait Drawing: Send + Sync {
    /// Unique identifier for this drawing.
    fn id(&self) -> &DrawingId;

    /// Test whether a chart point hits this drawing.
    ///
    /// `tolerance` is the maximum distance (in pixels) for a hit.
    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult;

    /// Move this drawing by the given delta (in chart coordinates).
    fn move_by(&mut self, delta: ChartPoint);

    /// Bounding rectangle in chart coordinates (timestamp, price).
    fn bounds(&self) -> DrawingBounds;

    /// Whether this drawing is currently selected.
    fn is_selected(&self) -> bool;

    /// Set the selection state.
    fn set_selected(&mut self, selected: bool);

    /// Generate render commands for this drawing using the given context.
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand>;
}

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
        Self { time_start, time_end, price_min, price_max }
    }

    /// Create bounds from a single point (zero-size).
    pub fn from_point(p: ChartPoint) -> Self {
        Self { time_start: p.timestamp, time_end: p.timestamp, price_min: p.price, price_max: p.price }
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
}

// ---------------------------------------------------------------------------
// Drawing impl for Ellipse (Circle)
// ---------------------------------------------------------------------------

impl Drawing for fast_chart_domain::drawing::Ellipse {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        // Check if point is inside the ellipse (normalized distance)
        let dx = (point.timestamp as f64 - self.center.timestamp as f64) / self.radius_x;
        let dy = (point.price - self.center.price) / self.radius_y;
        let dist = dx * dx + dy * dy;
        let tol = 1.0 + tolerance as f64 / self.radius_x.max(self.radius_y).max(1.0);
        if dist <= tol * tol {
            HitResult::Body
        } else {
            HitResult::Miss
        }
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.center.timestamp = self.center.timestamp.saturating_add(delta.timestamp);
        self.center.price += delta.price;
    }

    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::new(
            self.center.timestamp.saturating_sub(self.radius_x as u64),
            self.center.timestamp + self.radius_x as u64,
            self.center.price - self.radius_y,
            self.center.price + self.radius_y,
        )
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let center_screen = pipeline.world_to_screen(WorldPoint::new(self.center.timestamp as f64, self.center.price));

        // Approximate radius in screen space (average of x and y)
        let right = pipeline.world_to_screen(WorldPoint::new(self.center.timestamp as f64 + self.radius_x, self.center.price));
        let bottom = pipeline.world_to_screen(WorldPoint::new(self.center.timestamp as f64, self.center.price + self.radius_y));
        let rx = (right.x - center_screen.x).abs();
        let ry = (bottom.y - center_screen.y).abs();
        let radius = (rx + ry) / 2.0;

        let mut cmds = Vec::with_capacity(2);

        if let Some(fill) = self.fill_color {
            cmds.push(DrawCommand::DrawCircle {
                cx: center_screen.x,
                cy: center_screen.y,
                radius,
                fill: Some(fill),
                stroke: None,
                stroke_width: 0.0,
                z_index: 5,
            });
        }

        cmds.push(DrawCommand::DrawCircle {
            cx: center_screen.x,
            cy: center_screen.y,
            radius,
            fill: None,
            stroke: Some(self.color),
            stroke_width: self.width,
            z_index: 6,
        });

        cmds
    }
}

// ---------------------------------------------------------------------------
// Drawing impl for Rectangle (Box)
// ---------------------------------------------------------------------------

impl Drawing for fast_chart_domain::drawing::Rectangle {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        let min_t = self.top_left.timestamp.min(self.bottom_right.timestamp);
        let max_t = self.top_left.timestamp.max(self.bottom_right.timestamp);
        let min_p = self.top_left.price.min(self.bottom_right.price);
        let max_p = self.top_left.price.max(self.bottom_right.price);

        let tol = tolerance as f64;

        // Check if point is inside the rectangle (with tolerance padding)
        if point.timestamp as f64 >= min_t as f64 - tol
            && point.timestamp as f64 <= max_t as f64 + tol
            && point.price >= min_p - tol
            && point.price <= max_p + tol
        {
            HitResult::Body
        } else {
            HitResult::Miss
        }
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.top_left.timestamp = self.top_left.timestamp.saturating_add(delta.timestamp);
        self.top_left.price += delta.price;
        self.bottom_right.timestamp = self.bottom_right.timestamp.saturating_add(delta.timestamp);
        self.bottom_right.price += delta.price;
    }

    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::from_points(self.top_left, self.bottom_right)
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let tl = pipeline.world_to_screen(WorldPoint::new(self.top_left.timestamp as f64, self.top_left.price));
        let br = pipeline.world_to_screen(WorldPoint::new(self.bottom_right.timestamp as f64, self.bottom_right.price));

        let x = tl.x.min(br.x);
        let y = tl.y.min(br.y);
        let w = (br.x - tl.x).abs();
        let h = (br.y - tl.y).abs();

        let mut cmds = Vec::with_capacity(2);

        // Fill
        if let Some(fill) = self.fill_color {
            cmds.push(DrawCommand::DrawRect {
                x,
                y,
                width: w,
                height: h,
                fill: Some(fill),
                stroke: None,
                stroke_width: 0.0,
                z_index: 5,
            });
        }

        // Stroke
        let style = match self.style {
            fast_chart_domain::price_line::LineStyle::Solid => crate::render::commands::LineStyle::Solid,
            fast_chart_domain::price_line::LineStyle::Dashed => crate::render::commands::LineStyle::Dashed,
            fast_chart_domain::price_line::LineStyle::Dotted => crate::render::commands::LineStyle::Dotted,
        };
        cmds.push(DrawCommand::DrawRect {
            x,
            y,
            width: w,
            height: h,
            fill: None,
            stroke: Some(self.color),
            stroke_width: self.width,
            z_index: 6,
        });

        cmds
    }
}

// ---------------------------------------------------------------------------
// Drawing impl for Segment
// ---------------------------------------------------------------------------

impl Drawing for fast_chart_domain::drawing::Segment {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        // Same distance-to-segment as Arrow (finite line)
        let dx = self.end.timestamp as f64 - self.start.timestamp as f64;
        let dy = self.end.price - self.start.price;
        let len_sq = dx * dx + dy * dy;

        if len_sq == 0.0 {
            let px = point.timestamp as f64 - self.start.timestamp as f64;
            let py = point.price - self.start.price;
            let tol = tolerance as f64;
            return if px * px + py * py <= tol * tol {
                HitResult::Body
            } else {
                HitResult::Miss
            };
        }

        let t = ((point.timestamp as f64 - self.start.timestamp as f64) * dx
            + (point.price - self.start.price) * dy)
            / len_sq;
        let t = t.clamp(0.0, 1.0);

        let proj_x = self.start.timestamp as f64 + t * dx;
        let proj_y = self.start.price + t * dy;

        let dist_x = point.timestamp as f64 - proj_x;
        let dist_y = point.price - proj_y;
        let dist = (dist_x * dist_x + dist_y * dist_y).sqrt();

        let tol = tolerance as f64;
        if dist <= tol {
            HitResult::Body
        } else {
            HitResult::Miss
        }
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.start.timestamp = self.start.timestamp.saturating_add(delta.timestamp);
        self.start.price += delta.price;
        self.end.timestamp = self.end.timestamp.saturating_add(delta.timestamp);
        self.end.price += delta.price;
    }

    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::from_points(self.start, self.end)
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let start_screen = pipeline.world_to_screen(WorldPoint::new(self.start.timestamp as f64, self.start.price));
        let end_screen = pipeline.world_to_screen(WorldPoint::new(self.end.timestamp as f64, self.end.price));

        let style = match self.style {
            fast_chart_domain::price_line::LineStyle::Solid => crate::render::commands::LineStyle::Solid,
            fast_chart_domain::price_line::LineStyle::Dashed => crate::render::commands::LineStyle::Dashed,
            fast_chart_domain::price_line::LineStyle::Dotted => crate::render::commands::LineStyle::Dotted,
        };

        vec![DrawCommand::DrawLine {
            x0: start_screen.x,
            y0: start_screen.y,
            x1: end_screen.x,
            y1: end_screen.y,
            color: self.color,
            width: self.width,
            style,
            z_index: 10,
        }]
    }
}

// ---------------------------------------------------------------------------
// Drawing impl for Ray
// ---------------------------------------------------------------------------

impl Drawing for fast_chart_domain::drawing::Ray {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        // Ray extends from start through direction (infinite in that direction)
        let dx = self.direction.timestamp as f64 - self.start.timestamp as f64;
        let dy = self.direction.price - self.start.price;
        let len_sq = dx * dx + dy * dy;

        if len_sq == 0.0 {
            let px = point.timestamp as f64 - self.start.timestamp as f64;
            let py = point.price - self.start.price;
            let tol = tolerance as f64;
            return if px * px + py * py <= tol * tol {
                HitResult::Body
            } else {
                HitResult::Miss
            };
        }

        // Project point onto the ray direction (t >= 0 for ray)
        let t = ((point.timestamp as f64 - self.start.timestamp as f64) * dx
            + (point.price - self.start.price) * dy)
            / len_sq;

        if t < 0.0 {
            // Behind the start point
            return HitResult::Miss;
        }

        // Projected point on the ray
        let proj_x = self.start.timestamp as f64 + t * dx;
        let proj_y = self.start.price + t * dy;

        let dist_x = point.timestamp as f64 - proj_x;
        let dist_y = point.price - proj_y;
        let dist = (dist_x * dist_x + dist_y * dist_y).sqrt();

        let tol = tolerance as f64;
        if dist <= tol {
            HitResult::Body
        } else {
            HitResult::Miss
        }
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.start.timestamp = self.start.timestamp.saturating_add(delta.timestamp);
        self.start.price += delta.price;
        self.direction.timestamp = self.direction.timestamp.saturating_add(delta.timestamp);
        self.direction.price += delta.price;
    }

    fn bounds(&self) -> DrawingBounds {
        // Ray bounds use start as min; max is open-ended (use start + direction vector * large factor)
        let dx = self.direction.timestamp as f64 - self.start.timestamp as f64;
        let dy = self.direction.price - self.start.price;
        // Extend to a large timestamp for the bounding box
        let large_factor = 1000.0;
        let far_time = self.start.timestamp as f64 + dx * large_factor;
        let far_price = self.start.price + dy * large_factor;
        DrawingBounds::new(
            self.start.timestamp.min(far_time as u64),
            self.start.timestamp.max(far_time as u64),
            self.start.price.min(far_price),
            self.start.price.max(far_price),
        )
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let start_screen = pipeline.world_to_screen(WorldPoint::new(self.start.timestamp as f64, self.start.price));
        let dir_screen = pipeline.world_to_screen(WorldPoint::new(self.direction.timestamp as f64, self.direction.price));

        // Compute the direction vector in screen space and extend to pane edge
        let dx = dir_screen.x - start_screen.x;
        let dy = dir_screen.y - start_screen.y;
        let len = (dx * dx + dy * dy).sqrt();

        let (end_x, end_y) = if len > 0.01 {
            let ux = dx / len;
            let uy = dy / len;
            // Extend to cover the full pane width
            let extent = ctx.clip_rect.width + ctx.clip_rect.height;
            (start_screen.x + ux * extent, start_screen.y + uy * extent)
        } else {
            (start_screen.x, start_screen.y)
        };

        let style = match self.style {
            fast_chart_domain::price_line::LineStyle::Solid => crate::render::commands::LineStyle::Solid,
            fast_chart_domain::price_line::LineStyle::Dashed => crate::render::commands::LineStyle::Dashed,
            fast_chart_domain::price_line::LineStyle::Dotted => crate::render::commands::LineStyle::Dotted,
        };

        vec![DrawCommand::DrawLine {
            x0: start_screen.x,
            y0: start_screen.y,
            x1: end_x,
            y1: end_y,
            color: self.color,
            width: self.width,
            style,
            z_index: 10,
        }]
    }
}

// ---------------------------------------------------------------------------
// Drawing impl for Arrow
// ---------------------------------------------------------------------------

impl Drawing for fast_chart_domain::drawing::Arrow {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        // Distance from point to line segment (start → end)
        let dx = self.end.timestamp as f64 - self.start.timestamp as f64;
        let dy = self.end.price - self.start.price;
        let len_sq = dx * dx + dy * dy;

        if len_sq == 0.0 {
            // Degenerate: start == end — treat as a point hit
            let px = point.timestamp as f64 - self.start.timestamp as f64;
            let py = point.price - self.start.price;
            let tol = tolerance as f64;
            return if px * px + py * py <= tol * tol {
                HitResult::Body
            } else {
                HitResult::Miss
            };
        }

        let t = ((point.timestamp as f64 - self.start.timestamp as f64) * dx
            + (point.price - self.start.price) * dy)
            / len_sq;
        let t = t.clamp(0.0, 1.0);

        let proj_x = self.start.timestamp as f64 + t * dx;
        let proj_y = self.start.price + t * dy;

        let dist_x = point.timestamp as f64 - proj_x;
        let dist_y = point.price - proj_y;
        let dist = (dist_x * dist_x + dist_y * dist_y).sqrt();

        let tol = tolerance as f64;
        if dist <= tol {
            HitResult::Body
        } else {
            HitResult::Miss
        }
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.start.timestamp = self.start.timestamp.saturating_add(delta.timestamp);
        self.start.price += delta.price;
        self.end.timestamp = self.end.timestamp.saturating_add(delta.timestamp);
        self.end.price += delta.price;
    }

    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::from_points(self.start, self.end)
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let start_screen = pipeline.world_to_screen(WorldPoint::new(self.start.timestamp as f64, self.start.price));
        let end_screen = pipeline.world_to_screen(WorldPoint::new(self.end.timestamp as f64, self.end.price));

        let mut cmds = Vec::with_capacity(3);

        // 1. Line segment
        cmds.push(DrawCommand::DrawLine {
            x0: start_screen.x,
            y0: start_screen.y,
            x1: end_screen.x,
            y1: end_screen.y,
            color: self.color,
            width: self.width,
            style: match self.style {
                fast_chart_domain::price_line::LineStyle::Solid => {
                    crate::render::commands::LineStyle::Solid
                }
                fast_chart_domain::price_line::LineStyle::Dashed => {
                    crate::render::commands::LineStyle::Dashed
                }
                fast_chart_domain::price_line::LineStyle::Dotted => {
                    crate::render::commands::LineStyle::Dotted
                }
            },
            z_index: 10,
        });

        // 2. Arrowhead triangle at end point
        let dx = end_screen.x - start_screen.x;
        let dy = end_screen.y - start_screen.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.01 {
            let ux = dx / len;
            let uy = dy / len;
            // Perpendicular
            let px = -uy;
            let py = ux;

            let tip = self.arrowhead_size as f32;
            let half_base = tip * 0.4;

            let p1x = end_screen.x - ux * tip + px * half_base;
            let p1y = end_screen.y - uy * tip + py * half_base;
            let p2x = end_screen.x - ux * tip - px * half_base;
            let p2y = end_screen.y - uy * tip - py * half_base;

            cmds.push(DrawCommand::DrawTriangle {
                x0: end_screen.x,
                y0: end_screen.y,
                x1: p1x,
                y1: p1y,
                x2: p2x,
                y2: p2y,
                fill: Some(self.color),
                stroke: None,
                stroke_width: 0.0,
                z_index: 11,
            });
        }

        cmds
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawing_bounds_from_point() {
        let b = DrawingBounds::from_point(ChartPoint::new(1000, 50.0));
        assert_eq!(b.time_start, 1000);
        assert_eq!(b.time_end, 1000);
        assert!((b.price_min - 50.0).abs() < f64::EPSILON);
        assert!((b.price_max - 50.0).abs() < f64::EPSILON);
        assert_eq!(b.time_width(), 0);
        assert!((b.price_height()).abs() < f64::EPSILON);
    }

    #[test]
    fn drawing_bounds_from_points() {
        let a = ChartPoint::new(2000, 100.0);
        let b = ChartPoint::new(1000, 50.0);
        let bounds = DrawingBounds::from_points(a, b);
        assert_eq!(bounds.time_start, 1000);
        assert_eq!(bounds.time_end, 2000);
        assert!((bounds.price_min - 50.0).abs() < f64::EPSILON);
        assert!((bounds.price_max - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn drawing_bounds_contains() {
        let b = DrawingBounds::new(1000, 2000, 50.0, 100.0);
        assert!(b.contains(ChartPoint::new(1500, 75.0)));
        assert!(!b.contains(ChartPoint::new(500, 75.0)));
        assert!(!b.contains(ChartPoint::new(1500, 120.0)));
    }

    #[test]
    fn drawing_bounds_width_height() {
        let b = DrawingBounds::new(1000, 3000, 40.0, 80.0);
        assert_eq!(b.time_width(), 2000);
        assert!((b.price_height() - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hit_result_equality() {
        assert_eq!(HitResult::Miss, HitResult::Miss);
        assert_eq!(HitResult::Body, HitResult::Body);
        assert_eq!(HitResult::ControlPoint(0), HitResult::ControlPoint(0));
        assert_ne!(HitResult::ControlPoint(0), HitResult::ControlPoint(1));
        assert_ne!(HitResult::Miss, HitResult::Body);
    }

    #[test]
    fn drawing_trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        // We can't instantiate a trait object here, but we can verify
        // that the trait bounds are correct at the type level.
        assert_send_sync::<Box<dyn Drawing>>();
    }

    // ---- Arrow Drawing impl ----

    fn test_arrow() -> fast_chart_domain::drawing::Arrow {
        fast_chart_domain::drawing::Arrow::new(
            "test-arrow",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 150.0),
        )
    }

    #[test]
    fn arrow_id() {
        let arrow = test_arrow();
        assert_eq!(arrow.id().0, "test-arrow");
    }

    #[test]
    fn arrow_hit_test_body() {
        let arrow = test_arrow();
        // Point on the line segment
        let hit = arrow.hit_test(ChartPoint::new(1500, 125.0), 50.0);
        assert_eq!(hit, HitResult::Body);
    }

    #[test]
    fn arrow_hit_test_miss() {
        let arrow = test_arrow();
        // Point far from the line
        let hit = arrow.hit_test(ChartPoint::new(1500, 500.0), 5.0);
        assert_eq!(hit, HitResult::Miss);
    }

    #[test]
    fn arrow_move_by() {
        let mut arrow = test_arrow();
        arrow.move_by(ChartPoint::new(500, 10.0));
        assert_eq!(arrow.start.timestamp, 1500);
        assert!((arrow.start.price - 110.0).abs() < f64::EPSILON);
        assert_eq!(arrow.end.timestamp, 2500);
        assert!((arrow.end.price - 160.0).abs() < f64::EPSILON);
    }

    #[test]
    fn arrow_bounds() {
        let arrow = test_arrow();
        let bounds = arrow.bounds();
        assert_eq!(bounds.time_start, 1000);
        assert_eq!(bounds.time_end, 2000);
        assert!((bounds.price_min - 100.0).abs() < f64::EPSILON);
        assert!((bounds.price_max - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn arrow_selection_state() {
        let mut arrow = test_arrow();
        assert!(!arrow.is_selected());
        arrow.set_selected(true);
        assert!(arrow.is_selected());
        arrow.set_selected(false);
        assert!(!arrow.is_selected());
    }

    #[test]
    fn arrow_to_commands_produces_line_and_triangle() {
        use crate::render::context::RenderContext;
        use crate::render::coordinates::CoordinatePipeline;

        let arrow = fast_chart_domain::drawing::Arrow::new(
            "cmd-arrow",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 150.0),
        );

        let pipeline = CoordinatePipeline::new(
            (0.0, 3000.0),
            (50.0, 200.0),
            0.0, 0.0, 800.0, 400.0, 1.0,
        );
        let ctx = RenderContext::from_pipeline(pipeline, crate::render::series_renderer::Rect::new(0.0, 0.0, 800.0, 400.0), 0);

        let cmds = arrow.to_commands(&ctx);
        // Should produce: DrawLine + DrawTriangle (arrowhead)
        assert!(cmds.len() >= 2, "expected at least 2 commands, got {}", cmds.len());

        // First command should be the line
        match &cmds[0] {
            DrawCommand::DrawLine { x0, y0, x1, y1, .. } => {
                assert!(*x0 >= 0.0 && *x0 <= 800.0);
                assert!(*x1 >= 0.0 && *x1 <= 800.0);
            }
            other => panic!("expected DrawLine, got {:?}", other),
        }

        // Second command should be the arrowhead triangle
        match &cmds[1] {
            DrawCommand::DrawTriangle { fill, .. } => {
                assert!(fill.is_some(), "arrowhead should be filled");
            }
            other => panic!("expected DrawTriangle, got {:?}", other),
        }
    }

    #[test]
    fn arrow_degenerate_hit_test() {
        // Arrow with zero length
        let arrow = fast_chart_domain::drawing::Arrow::new(
            "degen",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(1000, 100.0),
        );
        assert_eq!(arrow.hit_test(ChartPoint::new(1000, 100.0), 5.0), HitResult::Body);
        assert_eq!(arrow.hit_test(ChartPoint::new(2000, 200.0), 5.0), HitResult::Miss);
    }

    // ---- Ray Drawing impl ----

    fn test_ray() -> fast_chart_domain::drawing::Ray {
        fast_chart_domain::drawing::Ray::new(
            "test-ray",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 150.0),
        )
    }

    #[test]
    fn ray_id() {
        let ray = test_ray();
        assert_eq!(ray.id().0, "test-ray");
    }

    #[test]
    fn ray_hit_test_body() {
        let ray = test_ray();
        // Point on the ray direction
        let hit = ray.hit_test(ChartPoint::new(1500, 125.0), 50.0);
        assert_eq!(hit, HitResult::Body);
    }

    #[test]
    fn ray_hit_test_miss_behind() {
        let ray = test_ray();
        // Point behind the start (negative t)
        let hit = ray.hit_test(ChartPoint::new(500, 75.0), 50.0);
        assert_eq!(hit, HitResult::Miss);
    }

    #[test]
    fn ray_hit_test_miss_far() {
        let ray = test_ray();
        // Point far perpendicular
        let hit = ray.hit_test(ChartPoint::new(1500, 500.0), 5.0);
        assert_eq!(hit, HitResult::Miss);
    }

    #[test]
    fn ray_move_by() {
        let mut ray = test_ray();
        ray.move_by(ChartPoint::new(500, 10.0));
        assert_eq!(ray.start.timestamp, 1500);
        assert!((ray.start.price - 110.0).abs() < f64::EPSILON);
        assert_eq!(ray.direction.timestamp, 2500);
        assert!((ray.direction.price - 160.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ray_bounds() {
        let ray = test_ray();
        let bounds = ray.bounds();
        // Bounds should include start
        assert!(bounds.contains(ChartPoint::new(1000, 100.0)));
    }

    #[test]
    fn ray_selection_state() {
        let mut ray = test_ray();
        assert!(!ray.is_selected());
        ray.set_selected(true);
        assert!(ray.is_selected());
        ray.set_selected(false);
        assert!(!ray.is_selected());
    }

    #[test]
    fn ray_to_commands_produces_line() {
        use crate::render::context::RenderContext;
        use crate::render::coordinates::CoordinatePipeline;

        let ray = fast_chart_domain::drawing::Ray::new(
            "cmd-ray",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 150.0),
        );

        let pipeline = CoordinatePipeline::new(
            (0.0, 3000.0),
            (50.0, 200.0),
            0.0, 0.0, 800.0, 400.0, 1.0,
        );
        let ctx = RenderContext::from_pipeline(pipeline, crate::render::series_renderer::Rect::new(0.0, 0.0, 800.0, 400.0), 0);

        let cmds = ray.to_commands(&ctx);
        assert_eq!(cmds.len(), 1, "ray should produce exactly 1 DrawLine");

        match &cmds[0] {
            DrawCommand::DrawLine { x0, y0, x1, y1, z_index, .. } => {
                // Start should be in screen bounds
                assert!(*x0 >= 0.0 && *x0 <= 800.0);
                assert!(*y0 >= 0.0 && *y0 <= 400.0);
                // End should extend far
                assert!(*x1 > *x0 || *y1 > *y0, "ray should extend beyond start");
                assert_eq!(*z_index, 10);
            }
            other => panic!("expected DrawLine, got {:?}", other),
        }
    }

    // ---- Segment Drawing impl ----

    #[test]
    fn segment_hit_test_body() {
        let seg = fast_chart_domain::drawing::Segment::new(
            "seg1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 150.0),
        );
        assert_eq!(seg.hit_test(ChartPoint::new(1500, 125.0), 50.0), HitResult::Body);
    }

    #[test]
    fn segment_hit_test_miss() {
        let seg = fast_chart_domain::drawing::Segment::new(
            "seg1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 150.0),
        );
        assert_eq!(seg.hit_test(ChartPoint::new(1500, 500.0), 5.0), HitResult::Miss);
    }

    #[test]
    fn segment_move_by() {
        let mut seg = fast_chart_domain::drawing::Segment::new(
            "seg1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 150.0),
        );
        seg.move_by(ChartPoint::new(100, 10.0));
        assert_eq!(seg.start.timestamp, 1100);
        assert_eq!(seg.end.timestamp, 2100);
    }

    #[test]
    fn segment_bounds() {
        let seg = fast_chart_domain::drawing::Segment::new(
            "seg1",
            ChartPoint::new(2000, 150.0),
            ChartPoint::new(1000, 100.0),
        );
        let b = seg.bounds();
        assert_eq!(b.time_start, 1000);
        assert_eq!(b.time_end, 2000);
    }

    #[test]
    fn segment_to_commands() {
        use crate::render::context::RenderContext;
        use crate::render::coordinates::CoordinatePipeline;

        let seg = fast_chart_domain::drawing::Segment::new(
            "cmd-seg",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 150.0),
        );

        let pipeline = CoordinatePipeline::new(
            (0.0, 3000.0),
            (50.0, 200.0),
            0.0, 0.0, 800.0, 400.0, 1.0,
        );
        let ctx = RenderContext::from_pipeline(pipeline, crate::render::series_renderer::Rect::new(0.0, 0.0, 800.0, 400.0), 0);

        let cmds = seg.to_commands(&ctx);
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            DrawCommand::DrawLine { z_index, .. } => assert_eq!(*z_index, 10),
            other => panic!("expected DrawLine, got {:?}", other),
        }
    }

    // ---- Rectangle (Box) Drawing impl ----

    #[test]
    fn rectangle_hit_test_inside() {
        let rect = fast_chart_domain::drawing::Rectangle::new(
            "box1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        );
        assert_eq!(rect.hit_test(ChartPoint::new(1500, 150.0), 5.0), HitResult::Body);
    }

    #[test]
    fn rectangle_hit_test_outside() {
        let rect = fast_chart_domain::drawing::Rectangle::new(
            "box1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        );
        assert_eq!(rect.hit_test(ChartPoint::new(3000, 300.0), 5.0), HitResult::Miss);
    }

    #[test]
    fn rectangle_hit_test_edge_tolerance() {
        let rect = fast_chart_domain::drawing::Rectangle::new(
            "box1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        );
        // Just outside the rectangle but within tolerance
        assert_eq!(rect.hit_test(ChartPoint::new(2001, 150.0), 5.0), HitResult::Body);
    }

    #[test]
    fn rectangle_move_by() {
        let mut rect = fast_chart_domain::drawing::Rectangle::new(
            "box1",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        );
        rect.move_by(ChartPoint::new(100, 10.0));
        assert_eq!(rect.top_left.timestamp, 1100);
        assert_eq!(rect.bottom_right.timestamp, 2100);
    }

    #[test]
    fn rectangle_bounds() {
        let rect = fast_chart_domain::drawing::Rectangle::new(
            "box1",
            ChartPoint::new(2000, 200.0),
            ChartPoint::new(1000, 100.0),
        );
        let b = rect.bounds();
        assert_eq!(b.time_start, 1000);
        assert_eq!(b.time_end, 2000);
    }

    #[test]
    fn rectangle_to_commands_with_fill() {
        use crate::render::context::RenderContext;
        use crate::render::coordinates::CoordinatePipeline;

        let rect = fast_chart_domain::drawing::Rectangle::new(
            "box-fill",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        ).with_fill([0.5, 0.5, 0.5, 0.3]);

        let pipeline = CoordinatePipeline::new(
            (0.0, 3000.0),
            (50.0, 250.0),
            0.0, 0.0, 800.0, 400.0, 1.0,
        );
        let ctx = RenderContext::from_pipeline(pipeline, crate::render::series_renderer::Rect::new(0.0, 0.0, 800.0, 400.0), 0);

        let cmds = rect.to_commands(&ctx);
        // Fill + Stroke = 2 commands
        assert_eq!(cmds.len(), 2);

        // First should be fill
        match &cmds[0] {
            DrawCommand::DrawRect { fill, stroke, z_index, .. } => {
                assert!(fill.is_some());
                assert!(stroke.is_none());
                assert_eq!(*z_index, 5);
            }
            other => panic!("expected DrawRect for fill, got {:?}", other),
        }

        // Second should be stroke
        match &cmds[1] {
            DrawCommand::DrawRect { fill, stroke, z_index, .. } => {
                assert!(fill.is_none());
                assert!(stroke.is_some());
                assert_eq!(*z_index, 6);
            }
            other => panic!("expected DrawRect for stroke, got {:?}", other),
        }
    }

    #[test]
    fn rectangle_to_commands_no_fill() {
        use crate::render::context::RenderContext;
        use crate::render::coordinates::CoordinatePipeline;

        let rect = fast_chart_domain::drawing::Rectangle::new(
            "box-stroke",
            ChartPoint::new(1000, 100.0),
            ChartPoint::new(2000, 200.0),
        );

        let pipeline = CoordinatePipeline::new(
            (0.0, 3000.0),
            (50.0, 250.0),
            0.0, 0.0, 800.0, 400.0, 1.0,
        );
        let ctx = RenderContext::from_pipeline(pipeline, crate::render::series_renderer::Rect::new(0.0, 0.0, 800.0, 400.0), 0);

        let cmds = rect.to_commands(&ctx);
        // No fill = only stroke
        assert_eq!(cmds.len(), 1);
    }

    // ---- Ellipse (Circle) Drawing impl ----

    #[test]
    fn ellipse_hit_test_inside() {
        let ell = fast_chart_domain::drawing::Ellipse::new(
            "e1",
            ChartPoint::new(1000, 100.0),
            500.0, 50.0,
        );
        assert_eq!(ell.hit_test(ChartPoint::new(1000, 100.0), 5.0), HitResult::Body);
    }

    #[test]
    fn ellipse_hit_test_outside() {
        let ell = fast_chart_domain::drawing::Ellipse::new(
            "e1",
            ChartPoint::new(1000, 100.0),
            100.0, 10.0,
        );
        assert_eq!(ell.hit_test(ChartPoint::new(5000, 500.0), 5.0), HitResult::Miss);
    }

    #[test]
    fn ellipse_move_by() {
        let mut ell = fast_chart_domain::drawing::Ellipse::new(
            "e1",
            ChartPoint::new(1000, 100.0),
            500.0, 50.0,
        );
        ell.move_by(ChartPoint::new(100, 10.0));
        assert_eq!(ell.center.timestamp, 1100);
        assert!((ell.center.price - 110.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ellipse_bounds() {
        let ell = fast_chart_domain::drawing::Ellipse::new(
            "e1",
            ChartPoint::new(1000, 100.0),
            500.0, 50.0,
        );
        let b = ell.bounds();
        assert_eq!(b.time_start, 500);
        assert_eq!(b.time_end, 1500);
        assert!((b.price_min - 50.0).abs() < f64::EPSILON);
        assert!((b.price_max - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ellipse_to_commands() {
        use crate::render::context::RenderContext;
        use crate::render::coordinates::CoordinatePipeline;

        let ell = fast_chart_domain::drawing::Ellipse::new(
            "cmd-e",
            ChartPoint::new(1000, 100.0),
            500.0, 50.0,
        );

        let pipeline = CoordinatePipeline::new(
            (0.0, 3000.0),
            (50.0, 200.0),
            0.0, 0.0, 800.0, 400.0, 1.0,
        );
        let ctx = RenderContext::from_pipeline(pipeline, crate::render::series_renderer::Rect::new(0.0, 0.0, 800.0, 400.0), 0);

        let cmds = ell.to_commands(&ctx);
        assert_eq!(cmds.len(), 1, "no fill = only stroke");
        match &cmds[0] {
            DrawCommand::DrawCircle { fill, stroke, radius, .. } => {
                assert!(fill.is_none());
                assert!(stroke.is_some());
                assert!(*radius > 0.0);
            }
            other => panic!("expected DrawCircle, got {:?}", other),
        }
    }
}
