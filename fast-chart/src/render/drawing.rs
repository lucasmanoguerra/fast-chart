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

    /// Combine two bounding boxes into one that contains both.
    pub fn combine(&self, other: &DrawingBounds) -> DrawingBounds {
        DrawingBounds {
            time_start: self.time_start.min(other.time_start),
            time_end: self.time_end.max(other.time_end),
            price_min: self.price_min.min(other.price_min),
            price_max: self.price_max.max(other.price_max),
        }
    }
}

// ---------------------------------------------------------------------------
// Drawing impl for ImageDrawing
// ---------------------------------------------------------------------------

impl Drawing for fast_chart_domain::drawing::ImageDrawing {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        let dx = (point.timestamp as f64 - self.position.timestamp as f64).abs();
        let dy = (point.price - self.position.price).abs();
        let tol = tolerance as f64;

        if dx <= self.width as f64 / 2.0 + tol && dy <= self.height as f64 / 2.0 + tol {
            HitResult::Body
        } else {
            HitResult::Miss
        }
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.position.timestamp = self.position.timestamp.saturating_add(delta.timestamp);
        self.position.price += delta.price;
    }

    fn bounds(&self) -> DrawingBounds {
        let half_w = self.width as f64 / 2.0;
        let half_h = self.height as f64 / 2.0;
        DrawingBounds::new(
            self.position.timestamp.saturating_sub(half_w as u64),
            self.position.timestamp + half_w as u64,
            self.position.price - half_h,
            self.position.price + half_h,
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
        let screen = pipeline.world_to_screen(WorldPoint::new(self.position.timestamp as f64, self.position.price));

        vec![DrawCommand::DrawImage {
            x: screen.x - self.width / 2.0,
            y: screen.y - self.height / 2.0,
            src: self.src.clone(),
            width: self.width,
            height: self.height,
            opacity: self.opacity,
            z_index: 15,
        }]
    }
}

// ---------------------------------------------------------------------------
// Drawing impl for LabelDrawing
// ---------------------------------------------------------------------------

impl Drawing for fast_chart_domain::drawing::LabelDrawing {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        let char_width = self.font_size * 0.6;
        let text_w = self.text.len() as f64 * char_width as f64 + self.padding as f64 * 2.0;
        let text_h = self.font_size as f64 + self.padding as f64 * 2.0;
        let dx = (point.timestamp as f64 - self.position.timestamp as f64).abs();
        let dy = (point.price - self.position.price).abs();
        let tol = tolerance as f64;

        if dx <= text_w / 2.0 + tol && dy <= text_h / 2.0 + tol {
            HitResult::Body
        } else {
            HitResult::Miss
        }
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.position.timestamp = self.position.timestamp.saturating_add(delta.timestamp);
        self.position.price += delta.price;
    }

    fn bounds(&self) -> DrawingBounds {
        let char_width = self.font_size * 0.6;
        let half_w = (self.text.len() as f64 * char_width as f64) / 2.0 + self.padding as f64;
        let half_h = self.font_size as f64 / 2.0 + self.padding as f64;
        DrawingBounds::new(
            self.position.timestamp.saturating_sub(half_w as u64),
            self.position.timestamp + half_w as u64,
            self.position.price - half_h,
            self.position.price + half_h,
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
        let screen = pipeline.world_to_screen(WorldPoint::new(self.position.timestamp as f64, self.position.price));

        let char_width = self.font_size * 0.6;
        let text_w = self.text.len() as f32 * char_width + self.padding * 2.0;
        let text_h = self.font_size + self.padding * 2.0;

        let bg_x = screen.x - text_w / 2.0;
        let bg_y = screen.y - text_h / 2.0;

        let mut cmds = vec![
            // Background rect
            DrawCommand::DrawRect {
                x: bg_x,
                y: bg_y,
                width: text_w,
                height: text_h,
                fill: Some(self.bg_color),
                stroke: Some(self.border_color),
                stroke_width: 1.0,
                z_index: 18,
            },
            // Text
            DrawCommand::DrawText {
                x: bg_x + self.padding,
                y: bg_y + self.padding,
                text: self.text.clone(),
                color: self.text_color,
                font_size: self.font_size,
                align_x: 0.0,
                align_y: 0.0,
                z_index: 19,
            },
        ];

        // Selection indicator
        if self.selected {
            cmds.push(DrawCommand::DrawRect {
                x: bg_x - 2.0,
                y: bg_y - 2.0,
                width: text_w + 4.0,
                height: text_h + 4.0,
                fill: None,
                stroke: Some([0.3, 0.6, 1.0, 0.8]),
                stroke_width: 2.0,
                z_index: 25,
            });
        }

        cmds
    }
}

// ---------------------------------------------------------------------------
// Drawing impl for TextDrawing
// ---------------------------------------------------------------------------

impl Drawing for fast_chart_domain::drawing::TextDrawing {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        // Text hit-test: approximate bounding box based on font size and text length
        let char_width = self.font_size * 0.6; // rough estimate
        let text_width_chars = self.text.len() as f64;
        let half_w = (text_width_chars * char_width as f64) / 2.0;
        let half_h = self.font_size as f64;

        let dx = (point.timestamp as f64 - self.position.timestamp as f64).abs();
        let dy = (point.price - self.position.price).abs();
        let tol = tolerance as f64;

        if dx <= half_w + tol && dy <= half_h + tol {
            HitResult::Body
        } else {
            HitResult::Miss
        }
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.position.timestamp = self.position.timestamp.saturating_add(delta.timestamp);
        self.position.price += delta.price;
    }

    fn bounds(&self) -> DrawingBounds {
        let char_width = self.font_size * 0.6;
        let half_w = (self.text.len() as f64 * char_width as f64) / 2.0;
        let half_h = self.font_size as f64;
        DrawingBounds::new(
            self.position.timestamp.saturating_sub(half_w as u64),
            self.position.timestamp + half_w as u64,
            self.position.price - half_h,
            self.position.price + half_h,
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
        let screen = pipeline.world_to_screen(WorldPoint::new(self.position.timestamp as f64, self.position.price));

        vec![DrawCommand::DrawText {
            x: screen.x,
            y: screen.y,
            text: self.text.clone(),
            color: self.color,
            font_size: self.font_size,
            align_x: self.align_x,
            align_y: self.align_y,
            z_index: 20,
        }]
    }
}

// ---------------------------------------------------------------------------
// Drawing impl for Path (Polygon / Polyline)
// ---------------------------------------------------------------------------

impl Drawing for fast_chart_domain::drawing::Path {
    fn id(&self) -> &DrawingId {
        &self.id
    }

    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        if self.points.is_empty() {
            return HitResult::Miss;
        }

        // Check each segment for proximity
        let tol = tolerance as f64;
        for window in self.points.windows(2) {
            let a = window[0];
            let b = window[1];
            let dx = b.timestamp as f64 - a.timestamp as f64;
            let dy = b.price - a.price;
            let len_sq = dx * dx + dy * dy;
            if len_sq == 0.0 {
                let px = point.timestamp as f64 - a.timestamp as f64;
                let py = point.price - a.price;
                if px * px + py * py <= tol * tol {
                    return HitResult::Body;
                }
                continue;
            }
            let t = ((point.timestamp as f64 - a.timestamp as f64) * dx
                + (point.price - a.price) * dy)
                / len_sq;
            let t = t.clamp(0.0, 1.0);
            let proj_x = a.timestamp as f64 + t * dx;
            let proj_y = a.price + t * dy;
            let dist_x = point.timestamp as f64 - proj_x;
            let dist_y = point.price - proj_y;
            if dist_x * dist_x + dist_y * dist_y <= tol * tol {
                return HitResult::Body;
            }
        }

        HitResult::Miss
    }

    fn move_by(&mut self, delta: ChartPoint) {
        for p in &mut self.points {
            p.timestamp = p.timestamp.saturating_add(delta.timestamp);
            p.price += delta.price;
        }
    }

    fn bounds(&self) -> DrawingBounds {
        if self.points.is_empty() {
            return DrawingBounds::new(0, 0, 0.0, 0.0);
        }
        let mut min_t = u64::MAX;
        let mut max_t = 0u64;
        let mut min_p = f64::MAX;
        let mut max_p = f64::MIN;
        for p in &self.points {
            min_t = min_t.min(p.timestamp);
            max_t = max_t.max(p.timestamp);
            min_p = min_p.min(p.price);
            max_p = max_p.max(p.price);
        }
        DrawingBounds::new(min_t, max_t, min_p, max_p)
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        if self.points.is_empty() {
            return Vec::new();
        }

        let pipeline = &ctx.pipeline;
        let screen_points: Vec<[f32; 2]> = self
            .points
            .iter()
            .map(|p| {
                let sp = pipeline.world_to_screen(WorldPoint::new(p.timestamp as f64, p.price));
                [sp.x, sp.y]
            })
            .collect();

        let mut cmds = Vec::with_capacity(2);

        // Fill for closed paths
        if self.closed {
            if let Some(fill) = self.fill_color {
                cmds.push(DrawCommand::DrawPath {
                    points: screen_points.clone(),
                    color: [0.0; 4],
                    width: 0.0,
                    style: crate::render::commands::LineStyle::Solid,
                    closed: true,
                    fill: Some(fill),
                    z_index: 5,
                });
            }
        }

        // Stroke
        let style = match self.style {
            fast_chart_domain::price_line::LineStyle::Solid => crate::render::commands::LineStyle::Solid,
            fast_chart_domain::price_line::LineStyle::Dashed => crate::render::commands::LineStyle::Dashed,
            fast_chart_domain::price_line::LineStyle::Dotted => crate::render::commands::LineStyle::Dotted,
        };
        cmds.push(DrawCommand::DrawPath {
            points: screen_points,
            color: self.color,
            width: self.width,
            style,
            closed: self.closed,
            fill: None,
            z_index: 10,
        });

        cmds
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
// Drawing impl for TrendLine
// ---------------------------------------------------------------------------

impl Drawing for fast_chart_domain::drawing::TrendLine {
    fn id(&self) -> &DrawingId { &self.id }

    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        let dx = self.end.timestamp as f64 - self.start.timestamp as f64;
        let dy = self.end.price - self.start.price;
        let len_sq = dx * dx + dy * dy;
        if len_sq == 0.0 {
            let px = point.timestamp as f64 - self.start.timestamp as f64;
            let py = point.price - self.start.price;
            return if px * px + py * py <= tolerance as f64 * tolerance as f64 { HitResult::Body } else { HitResult::Miss };
        }
        let t = (((point.timestamp as f64 - self.start.timestamp as f64) * dx + (point.price - self.start.price) * dy) / len_sq).clamp(0.0, 1.0);
        let proj_x = self.start.timestamp as f64 + t * dx;
        let proj_y = self.start.price + t * dy;
        let dist = ((point.timestamp as f64 - proj_x).powi(2) + (point.price - proj_y).powi(2)).sqrt();
        if dist <= tolerance as f64 { HitResult::Body } else { HitResult::Miss }
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.start.timestamp = self.start.timestamp.saturating_add(delta.timestamp);
        self.start.price += delta.price;
        self.end.timestamp = self.end.timestamp.saturating_add(delta.timestamp);
        self.end.price += delta.price;
    }

    fn bounds(&self) -> DrawingBounds { DrawingBounds::from_points(self.start, self.end) }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }

    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let start_screen = ctx.pipeline.world_to_screen(WorldPoint::new(self.start.timestamp as f64, self.start.price));
        let end_screen = ctx.pipeline.world_to_screen(WorldPoint::new(self.end.timestamp as f64, self.end.price));
        let style = match self.style {
            fast_chart_domain::price_line::LineStyle::Solid => crate::render::commands::LineStyle::Solid,
            fast_chart_domain::price_line::LineStyle::Dashed => crate::render::commands::LineStyle::Dashed,
            fast_chart_domain::price_line::LineStyle::Dotted => crate::render::commands::LineStyle::Dotted,
        };
        vec![DrawCommand::DrawLine { x0: start_screen.x, y0: start_screen.y, x1: end_screen.x, y1: end_screen.y, color: self.color, width: self.width, style, z_index: 10 }]
    }
}

// ---------------------------------------------------------------------------
// Drawing impl for HorizontalLine
// ---------------------------------------------------------------------------

impl Drawing for fast_chart_domain::drawing::HorizontalLine {
    fn id(&self) -> &DrawingId { &self.id }

    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        let dist = (point.price - self.price).abs();
        if dist <= tolerance as f64 { HitResult::Body } else { HitResult::Miss }
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.price += delta.price;
    }

    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::new(0, u64::MAX, self.price, self.price)
    }

    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }

    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let screen = ctx.pipeline.world_to_screen(WorldPoint::new(ctx.time_range.0, self.price));
        let screen2 = ctx.pipeline.world_to_screen(WorldPoint::new(ctx.time_range.1, self.price));
        let style = match self.style {
            fast_chart_domain::price_line::LineStyle::Solid => crate::render::commands::LineStyle::Solid,
            fast_chart_domain::price_line::LineStyle::Dashed => crate::render::commands::LineStyle::Dashed,
            fast_chart_domain::price_line::LineStyle::Dotted => crate::render::commands::LineStyle::Dotted,
        };
        vec![DrawCommand::DrawLine { x0: screen.x, y0: screen.y, x1: screen2.x, y1: screen2.y, color: self.color, width: self.width, style, z_index: 8 }]
    }
}

// ---------------------------------------------------------------------------
// Drawing impl for VerticalLine
// ---------------------------------------------------------------------------

impl Drawing for fast_chart_domain::drawing::VerticalLine {
    fn id(&self) -> &DrawingId { &self.id }

    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        let dist = (point.timestamp as f64 - self.timestamp as f64).abs();
        if dist <= tolerance as f64 { HitResult::Body } else { HitResult::Miss }
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.timestamp = self.timestamp.saturating_add(delta.timestamp);
    }

    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::new(self.timestamp, self.timestamp, f64::MIN, f64::MAX)
    }

    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }

    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let screen1 = ctx.pipeline.world_to_screen(WorldPoint::new(self.timestamp as f64, ctx.price_range.0));
        let screen2 = ctx.pipeline.world_to_screen(WorldPoint::new(self.timestamp as f64, ctx.price_range.1));
        let style = match self.style {
            fast_chart_domain::price_line::LineStyle::Solid => crate::render::commands::LineStyle::Solid,
            fast_chart_domain::price_line::LineStyle::Dashed => crate::render::commands::LineStyle::Dashed,
            fast_chart_domain::price_line::LineStyle::Dotted => crate::render::commands::LineStyle::Dotted,
        };
        vec![DrawCommand::DrawLine { x0: screen1.x, y0: screen1.y, x1: screen2.x, y1: screen2.y, color: self.color, width: self.width, style, z_index: 8 }]
    }
}

// ---------------------------------------------------------------------------
// Drawing impl for FibonacciRetracement
// ---------------------------------------------------------------------------

impl Drawing for fast_chart_domain::drawing::FibonacciRetracement {
    fn id(&self) -> &DrawingId { &self.id }

    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        // Check if near any level line
        let tol = tolerance as f64;
        for &level in &self.levels {
            let price = self.price_at_level(level);
            if (point.price - price).abs() <= tol { return HitResult::Body; }
        }
        HitResult::Miss
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.start.timestamp = self.start.timestamp.saturating_add(delta.timestamp);
        self.start.price += delta.price;
        self.end.timestamp = self.end.timestamp.saturating_add(delta.timestamp);
        self.end.price += delta.price;
    }

    fn bounds(&self) -> DrawingBounds { DrawingBounds::from_points(self.start, self.end) }
    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }

    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let mut cmds = Vec::new();
        let style = match self.style {
            fast_chart_domain::price_line::LineStyle::Solid => crate::render::commands::LineStyle::Solid,
            fast_chart_domain::price_line::LineStyle::Dashed => crate::render::commands::LineStyle::Dashed,
            fast_chart_domain::price_line::LineStyle::Dotted => crate::render::commands::LineStyle::Dotted,
        };
        let left = ctx.time_range.0;
        let right = ctx.time_range.1;
        for &level in &self.levels {
            let price = self.price_at_level(level);
            let s1 = ctx.pipeline.world_to_screen(WorldPoint::new(left as f64, price));
            let s2 = ctx.pipeline.world_to_screen(WorldPoint::new(right as f64, price));
            cmds.push(DrawCommand::DrawLine { x0: s1.x, y0: s1.y, x1: s2.x, y1: s2.y, color: self.color, width: self.width, style, z_index: 9 });
        }
        cmds
    }
}

// ---------------------------------------------------------------------------
// Drawing impl for FibonacciExtension
// ---------------------------------------------------------------------------

impl Drawing for fast_chart_domain::drawing::FibonacciExtension {
    fn id(&self) -> &DrawingId { &self.id }

    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        let tol = tolerance as f64;
        for &level in &self.levels {
            let price = self.price_at_level(level);
            if (point.price - price).abs() <= tol { return HitResult::Body; }
        }
        HitResult::Miss
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.point_a.timestamp = self.point_a.timestamp.saturating_add(delta.timestamp);
        self.point_a.price += delta.price;
        self.point_b.timestamp = self.point_b.timestamp.saturating_add(delta.timestamp);
        self.point_b.price += delta.price;
        self.point_c.timestamp = self.point_c.timestamp.saturating_add(delta.timestamp);
        self.point_c.price += delta.price;
    }

    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::from_points(self.point_a, self.point_b).combine(&DrawingBounds::from_point(self.point_c))
    }

    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }

    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let mut cmds = Vec::new();
        let style = match self.style {
            fast_chart_domain::price_line::LineStyle::Solid => crate::render::commands::LineStyle::Solid,
            fast_chart_domain::price_line::LineStyle::Dashed => crate::render::commands::LineStyle::Dashed,
            fast_chart_domain::price_line::LineStyle::Dotted => crate::render::commands::LineStyle::Dotted,
        };
        let left = ctx.time_range.0;
        let right = ctx.time_range.1;
        for &level in &self.levels {
            let price = self.price_at_level(level);
            let s1 = ctx.pipeline.world_to_screen(WorldPoint::new(left as f64, price));
            let s2 = ctx.pipeline.world_to_screen(WorldPoint::new(right as f64, price));
            cmds.push(DrawCommand::DrawLine { x0: s1.x, y0: s1.y, x1: s2.x, y1: s2.y, color: self.color, width: self.width, style, z_index: 9 });
        }
        cmds
    }
}

// ---------------------------------------------------------------------------
// Drawing impl for Pitchfork
// ---------------------------------------------------------------------------

impl Drawing for fast_chart_domain::drawing::Pitchfork {
    fn id(&self) -> &DrawingId { &self.id }

    fn hit_test(&self, point: ChartPoint, tolerance: f32) -> HitResult {
        let tol = tolerance as f64;
        // Check distance to three prongs (A→midpoint, A→B, A→C)
        let mid = ChartPoint::new((self.point_b.timestamp + self.point_c.timestamp) / 2, (self.point_b.price + self.point_c.price) / 2.0);
        for &end in &[mid, self.point_b, self.point_c] {
            let dx = end.timestamp as f64 - self.point_a.timestamp as f64;
            let dy = end.price - self.point_a.price;
            let len_sq = dx * dx + dy * dy;
            if len_sq == 0.0 { continue; }
            let t = (((point.timestamp as f64 - self.point_a.timestamp as f64) * dx + (point.price - self.point_a.price) * dy) / len_sq).clamp(0.0, 1.0);
            let proj_x = self.point_a.timestamp as f64 + t * dx;
            let proj_y = self.point_a.price + t * dy;
            let dist = ((point.timestamp as f64 - proj_x).powi(2) + (point.price - proj_y).powi(2)).sqrt();
            if dist <= tol { return HitResult::Body; }
        }
        HitResult::Miss
    }

    fn move_by(&mut self, delta: ChartPoint) {
        self.point_a.timestamp = self.point_a.timestamp.saturating_add(delta.timestamp);
        self.point_a.price += delta.price;
        self.point_b.timestamp = self.point_b.timestamp.saturating_add(delta.timestamp);
        self.point_b.price += delta.price;
        self.point_c.timestamp = self.point_c.timestamp.saturating_add(delta.timestamp);
        self.point_c.price += delta.price;
    }

    fn bounds(&self) -> DrawingBounds {
        DrawingBounds::from_points(self.point_a, self.point_b).combine(&DrawingBounds::from_point(self.point_c))
    }

    fn is_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected: bool) { self.selected = selected; }

    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let style = match self.style {
            fast_chart_domain::price_line::LineStyle::Solid => crate::render::commands::LineStyle::Solid,
            fast_chart_domain::price_line::LineStyle::Dashed => crate::render::commands::LineStyle::Dashed,
            fast_chart_domain::price_line::LineStyle::Dotted => crate::render::commands::LineStyle::Dotted,
        };
        let sa = ctx.pipeline.world_to_screen(WorldPoint::new(self.point_a.timestamp as f64, self.point_a.price));
        let sb = ctx.pipeline.world_to_screen(WorldPoint::new(self.point_b.timestamp as f64, self.point_b.price));
        let sc = ctx.pipeline.world_to_screen(WorldPoint::new(self.point_c.timestamp as f64, self.point_c.price));
        vec![
            DrawCommand::DrawLine { x0: sa.x, y0: sa.y, x1: sb.x, y1: sb.y, color: self.color, width: self.width, style, z_index: 10 },
            DrawCommand::DrawLine { x0: sa.x, y0: sa.y, x1: sc.x, y1: sc.y, color: self.color, width: self.width, style, z_index: 10 },
        ]
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

    // ---- Path (Polygon) Drawing impl ----

    #[test]
    fn path_hit_test_on_segment() {
        let path = fast_chart_domain::drawing::Path::new(
            "poly1",
            vec![
                ChartPoint::new(1000, 100.0),
                ChartPoint::new(2000, 200.0),
                ChartPoint::new(3000, 100.0),
            ],
        );
        // Near the middle segment
        assert_eq!(path.hit_test(ChartPoint::new(1500, 150.0), 50.0), HitResult::Body);
    }

    #[test]
    fn path_hit_test_miss() {
        let path = fast_chart_domain::drawing::Path::new(
            "poly1",
            vec![
                ChartPoint::new(1000, 100.0),
                ChartPoint::new(2000, 200.0),
            ],
        );
        assert_eq!(path.hit_test(ChartPoint::new(1500, 500.0), 5.0), HitResult::Miss);
    }

    #[test]
    fn path_move_by() {
        let mut path = fast_chart_domain::drawing::Path::new(
            "poly1",
            vec![
                ChartPoint::new(1000, 100.0),
                ChartPoint::new(2000, 200.0),
            ],
        );
        path.move_by(ChartPoint::new(100, 10.0));
        assert_eq!(path.points[0].timestamp, 1100);
        assert_eq!(path.points[1].timestamp, 2100);
    }

    #[test]
    fn path_bounds() {
        let path = fast_chart_domain::drawing::Path::new(
            "poly1",
            vec![
                ChartPoint::new(3000, 200.0),
                ChartPoint::new(1000, 100.0),
                ChartPoint::new(2000, 300.0),
            ],
        );
        let b = path.bounds();
        assert_eq!(b.time_start, 1000);
        assert_eq!(b.time_end, 3000);
        assert!((b.price_min - 100.0).abs() < f64::EPSILON);
        assert!((b.price_max - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn path_to_commands_closed_polygon() {
        use crate::render::context::RenderContext;
        use crate::render::coordinates::CoordinatePipeline;

        let path = fast_chart_domain::drawing::Path::new(
            "cmd-poly",
            vec![
                ChartPoint::new(1000, 100.0),
                ChartPoint::new(2000, 200.0),
                ChartPoint::new(3000, 100.0),
            ],
        )
        .with_closed(true)
        .with_fill([0.5, 0.5, 0.5, 0.3]);

        let pipeline = CoordinatePipeline::new(
            (0.0, 4000.0),
            (50.0, 250.0),
            0.0, 0.0, 800.0, 400.0, 1.0,
        );
        let ctx = RenderContext::from_pipeline(pipeline, crate::render::series_renderer::Rect::new(0.0, 0.0, 800.0, 400.0), 0);

        let cmds = path.to_commands(&ctx);
        // Fill + Stroke = 2
        assert_eq!(cmds.len(), 2);

        match &cmds[0] {
            DrawCommand::DrawPath { closed, fill, z_index, .. } => {
                assert!(*closed);
                assert!(fill.is_some());
                assert_eq!(*z_index, 5);
            }
            other => panic!("expected DrawPath fill, got {:?}", other),
        }
    }

    // ---- TextDrawing Drawing impl tests ----

    #[test]
    fn text_drawing_hit_test() {
        let td = fast_chart_domain::drawing::TextDrawing::new(
            "txt1",
            ChartPoint::new(2000, 150.0),
            "hello world",
        );
        // At the anchor point
        assert_eq!(td.hit_test(ChartPoint::new(2000, 150.0), 5.0), HitResult::Body);
        // Far away
        assert_eq!(td.hit_test(ChartPoint::new(5000, 500.0), 5.0), HitResult::Miss);
    }

    #[test]
    fn text_drawing_move_by() {
        let mut td = fast_chart_domain::drawing::TextDrawing::new(
            "txt1",
            ChartPoint::new(1000, 100.0),
            "label",
        );
        td.move_by(ChartPoint::new(500, 25.0));
        assert_eq!(td.position.timestamp, 1500);
        assert!((td.position.price - 125.0).abs() < f64::EPSILON);
    }

    #[test]
    fn text_drawing_bounds() {
        let td = fast_chart_domain::drawing::TextDrawing::new(
            "txt1",
            ChartPoint::new(2000, 150.0),
            "hello",
        );
        let b = td.bounds();
        assert!(b.time_start < 2000);
        assert!(b.time_end > 2000);
        assert!(b.price_min < 150.0);
        assert!(b.price_max > 150.0);
    }

    #[test]
    fn text_drawing_to_commands() {
        use crate::render::context::RenderContext;
        use crate::render::coordinates::CoordinatePipeline;

        let td = fast_chart_domain::drawing::TextDrawing::new(
            "cmd-t",
            ChartPoint::new(1000, 100.0),
            "Price",
        )
        .with_color([1.0, 0.0, 0.0, 1.0])
        .with_font_size(16.0);

        let pipeline = CoordinatePipeline::new(
            (0.0, 3000.0),
            (50.0, 200.0),
            0.0, 0.0, 800.0, 400.0, 1.0,
        );
        let ctx = RenderContext::from_pipeline(pipeline, crate::render::series_renderer::Rect::new(0.0, 0.0, 800.0, 400.0), 0);

        let cmds = td.to_commands(&ctx);
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            DrawCommand::DrawText { text, color, font_size, z_index, .. } => {
                assert_eq!(text, "Price");
                assert_eq!(color, &[1.0, 0.0, 0.0, 1.0]);
                assert_eq!(*font_size, 16.0);
                assert_eq!(*z_index, 20);
            }
            other => panic!("expected DrawText, got {:?}", other),
        }
    }

    #[test]
    fn text_drawing_selected_toggle() {
        let mut td = fast_chart_domain::drawing::TextDrawing::new(
            "txt1",
            ChartPoint::new(1000, 100.0),
            "sel",
        );
        assert!(!td.is_selected());
        td.set_selected(true);
        assert!(td.is_selected());
        td.set_selected(false);
        assert!(!td.is_selected());
    }

    // ---- ImageDrawing Drawing impl tests ----

    #[test]
    fn image_hit_test() {
        let img = fast_chart_domain::drawing::ImageDrawing::new(
            "img1",
            ChartPoint::new(2000, 150.0),
            "logo.png",
        );
        // At anchor
        assert_eq!(img.hit_test(ChartPoint::new(2000, 150.0), 5.0), HitResult::Body);
        // Inside bounds
        assert_eq!(img.hit_test(ChartPoint::new(2020, 160.0), 5.0), HitResult::Body);
        // Far away
        assert_eq!(img.hit_test(ChartPoint::new(5000, 500.0), 5.0), HitResult::Miss);
    }

    #[test]
    fn image_move_by() {
        let mut img = fast_chart_domain::drawing::ImageDrawing::new(
            "img1",
            ChartPoint::new(1000, 100.0),
            "pic.png",
        );
        img.move_by(ChartPoint::new(500, 25.0));
        assert_eq!(img.position.timestamp, 1500);
        assert!((img.position.price - 125.0).abs() < f64::EPSILON);
    }

    #[test]
    fn image_bounds() {
        let img = fast_chart_domain::drawing::ImageDrawing::new(
            "img1",
            ChartPoint::new(2000, 150.0),
            "pic.png",
        ).with_width(200.0).with_height(100.0);
        let b = img.bounds();
        assert_eq!(b.time_start, 1900);
        assert_eq!(b.time_end, 2100);
        assert!((b.price_min - 100.0).abs() < f64::EPSILON);
        assert!((b.price_max - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn image_to_commands() {
        use crate::render::context::RenderContext;
        use crate::render::coordinates::CoordinatePipeline;

        let img = fast_chart_domain::drawing::ImageDrawing::new(
            "cmd-img",
            ChartPoint::new(1000, 100.0),
            "chart-bg.png",
        )
        .with_width(150.0)
        .with_height(80.0)
        .with_opacity(0.7);

        let pipeline = CoordinatePipeline::new(
            (0.0, 3000.0),
            (50.0, 200.0),
            0.0, 0.0, 800.0, 400.0, 1.0,
        );
        let ctx = RenderContext::from_pipeline(pipeline, crate::render::series_renderer::Rect::new(0.0, 0.0, 800.0, 400.0), 0);

        let cmds = img.to_commands(&ctx);
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            DrawCommand::DrawImage { src, width, height, opacity, z_index, .. } => {
                assert_eq!(src, "chart-bg.png");
                assert_eq!(*width, 150.0);
                assert_eq!(*height, 80.0);
                assert!((opacity - 0.7).abs() < f32::EPSILON);
                assert_eq!(*z_index, 15);
            }
            other => panic!("expected DrawImage, got {:?}", other),
        }
    }

    // ---- LabelDrawing Drawing impl tests ----

    #[test]
    fn label_hit_test() {
        let label = fast_chart_domain::drawing::LabelDrawing::new(
            "lbl1",
            ChartPoint::new(2000, 150.0),
            "BTC",
        );
        assert_eq!(label.hit_test(ChartPoint::new(2000, 150.0), 5.0), HitResult::Body);
        assert_eq!(label.hit_test(ChartPoint::new(5000, 500.0), 5.0), HitResult::Miss);
    }

    #[test]
    fn label_move_by() {
        let mut label = fast_chart_domain::drawing::LabelDrawing::new(
            "lbl1",
            ChartPoint::new(1000, 100.0),
            "ETH",
        );
        label.move_by(ChartPoint::new(500, 25.0));
        assert_eq!(label.position.timestamp, 1500);
        assert!((label.position.price - 125.0).abs() < f64::EPSILON);
    }

    #[test]
    fn label_bounds() {
        let label = fast_chart_domain::drawing::LabelDrawing::new(
            "lbl1",
            ChartPoint::new(2000, 150.0),
            "SOL",
        );
        let b = label.bounds();
        assert!(b.time_start < 2000);
        assert!(b.time_end > 2000);
        assert!(b.price_min < 150.0);
        assert!(b.price_max > 150.0);
    }

    #[test]
    fn label_to_commands() {
        use crate::render::context::RenderContext;
        use crate::render::coordinates::CoordinatePipeline;

        let label = fast_chart_domain::drawing::LabelDrawing::new(
            "cmd-lbl",
            ChartPoint::new(1000, 100.0),
            "Label",
        )
        .with_text_color([1.0, 0.0, 0.0, 1.0])
        .with_bg_color([0.0, 0.0, 0.0, 0.8])
        .with_font_size(14.0);

        let pipeline = CoordinatePipeline::new(
            (0.0, 3000.0),
            (50.0, 200.0),
            0.0, 0.0, 800.0, 400.0, 1.0,
        );
        let ctx = RenderContext::from_pipeline(pipeline, crate::render::series_renderer::Rect::new(0.0, 0.0, 800.0, 400.0), 0);

        let cmds = label.to_commands(&ctx);
        // bg rect + text = 2 (not selected)
        assert_eq!(cmds.len(), 2);

        match &cmds[0] {
            DrawCommand::DrawRect { fill, stroke, z_index, .. } => {
                assert!(fill.is_some());
                assert!(stroke.is_some());
                assert_eq!(*z_index, 18);
            }
            other => panic!("expected DrawRect bg, got {:?}", other),
        }
        match &cmds[1] {
            DrawCommand::DrawText { text, color, font_size, z_index, .. } => {
                assert_eq!(text, "Label");
                assert_eq!(color, &[1.0, 0.0, 0.0, 1.0]);
                assert_eq!(*font_size, 14.0);
                assert_eq!(*z_index, 19);
            }
            other => panic!("expected DrawText, got {:?}", other),
        }
    }

    #[test]
    fn label_selected_adds_outline() {
        use crate::render::context::RenderContext;
        use crate::render::coordinates::CoordinatePipeline;

        let mut label = fast_chart_domain::drawing::LabelDrawing::new(
            "cmd-lbl2",
            ChartPoint::new(1000, 100.0),
            "Hi",
        );
        label.set_selected(true);

        let pipeline = CoordinatePipeline::new(
            (0.0, 3000.0),
            (50.0, 200.0),
            0.0, 0.0, 800.0, 400.0, 1.0,
        );
        let ctx = RenderContext::from_pipeline(pipeline, crate::render::series_renderer::Rect::new(0.0, 0.0, 800.0, 400.0), 0);

        let cmds = label.to_commands(&ctx);
        // bg rect + text + selection outline = 3
        assert_eq!(cmds.len(), 3);
        match &cmds[2] {
            DrawCommand::DrawRect { stroke, z_index, fill, .. } => {
                assert!(fill.is_none());
                assert!(stroke.is_some());
                assert_eq!(*z_index, 25);
            }
            other => panic!("expected DrawRect selection, got {:?}", other),
        }
    }
}
