// ---------------------------------------------------------------------------
// RenderableDrawing — extension trait adding to_commands to Drawing
// ---------------------------------------------------------------------------

use crate::commands::DrawCommand;
use crate::context::RenderContext;
use crate::coordinates::WorldPoint;
use fc_drawing::Drawing;
use fc_domain::price_line::LineStyle;

/// Extension trait that adds rendering capability to `Drawing`.
///
/// This trait is separate from `Drawing` because the rendering layer (`fc-render`)
/// cannot be a dependency of the domain layer (`fc-drawing`). Instead, this trait
/// bridges the gap by adding `to_commands` to any type that implements `Drawing`.
pub trait RenderableDrawing: Drawing {
    /// Generate render commands for this drawing using the given context.
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand>;
}

// ---------------------------------------------------------------------------
// Simple line-based types
// ---------------------------------------------------------------------------

impl RenderableDrawing for fc_domain::drawing::TrendLine {
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let s = pipeline.world_to_screen(WorldPoint::new(self.start.timestamp as f64, self.start.price));
        let e = pipeline.world_to_screen(WorldPoint::new(self.end.timestamp as f64, self.end.price));
        vec![DrawCommand::DrawLine { x0: s.x, y0: s.y, x1: e.x, y1: e.y, color: self.color, width: self.width, style: self.style, z_index: 10 }]
    }
}

impl RenderableDrawing for fc_domain::drawing::Arrow {
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let s = pipeline.world_to_screen(WorldPoint::new(self.start.timestamp as f64, self.start.price));
        let e = pipeline.world_to_screen(WorldPoint::new(self.end.timestamp as f64, self.end.price));
        let mut cmds = vec![DrawCommand::DrawLine { x0: s.x, y0: s.y, x1: e.x, y1: e.y, color: self.color, width: self.width, style: self.style, z_index: 10 }];
        let dx = e.x - s.x; let dy = e.y - s.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.01 {
            let ux = dx / len; let uy = dy / len;
            let px = -uy; let py = ux;
            let tip = self.arrowhead_size; let half_base = tip * 0.4;
            cmds.push(DrawCommand::DrawTriangle {
                x0: e.x, y0: e.y,
                x1: e.x - ux * tip + px * half_base, y1: e.y - uy * tip + py * half_base,
                x2: e.x - ux * tip - px * half_base, y2: e.y - uy * tip - py * half_base,
                fill: Some(self.color), stroke: None, stroke_width: 0.0, z_index: 11,
            });
        }
        cmds
    }
}

impl RenderableDrawing for fc_domain::drawing::Segment {
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let s = pipeline.world_to_screen(WorldPoint::new(self.start.timestamp as f64, self.start.price));
        let e = pipeline.world_to_screen(WorldPoint::new(self.end.timestamp as f64, self.end.price));
        vec![DrawCommand::DrawLine { x0: s.x, y0: s.y, x1: e.x, y1: e.y, color: self.color, width: self.width, style: self.style, z_index: 10 }]
    }
}

impl RenderableDrawing for fc_domain::drawing::Ray {
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let s = pipeline.world_to_screen(WorldPoint::new(self.start.timestamp as f64, self.start.price));
        let d = pipeline.world_to_screen(WorldPoint::new(self.direction.timestamp as f64, self.direction.price));
        let dx = d.x - s.x; let dy = d.y - s.y;
        let len = (dx * dx + dy * dy).sqrt();
        let (ex, ey) = if len > 0.01 { let ux = dx/len; let uy = dy/len; let ext = ctx.clip_rect.width + ctx.clip_rect.height; (s.x + ux*ext, s.y + uy*ext) } else { (s.x, s.y) };
        vec![DrawCommand::DrawLine { x0: s.x, y0: s.y, x1: ex, y1: ey, color: self.color, width: self.width, style: self.style, z_index: 10 }]
    }
}

// ---------------------------------------------------------------------------
// Span lines (horizontal/vertical)
// ---------------------------------------------------------------------------

impl RenderableDrawing for fc_domain::drawing::HorizontalLine {
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let s1 = pipeline.world_to_screen(WorldPoint::new(ctx.time_range.0, self.price));
        let s2 = pipeline.world_to_screen(WorldPoint::new(ctx.time_range.1, self.price));
        vec![DrawCommand::DrawLine { x0: s1.x, y0: s1.y, x1: s2.x, y1: s2.y, color: self.color, width: self.width, style: self.style, z_index: 8 }]
    }
}

impl RenderableDrawing for fc_domain::drawing::VerticalLine {
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let s1 = pipeline.world_to_screen(WorldPoint::new(self.timestamp as f64, ctx.price_range.0));
        let s2 = pipeline.world_to_screen(WorldPoint::new(self.timestamp as f64, ctx.price_range.1));
        vec![DrawCommand::DrawLine { x0: s1.x, y0: s1.y, x1: s2.x, y1: s2.y, color: self.color, width: self.width, style: self.style, z_index: 8 }]
    }
}

// ---------------------------------------------------------------------------
// Shape types
// ---------------------------------------------------------------------------

impl RenderableDrawing for fc_domain::drawing::Rectangle {
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let tl = pipeline.world_to_screen(WorldPoint::new(self.top_left.timestamp as f64, self.top_left.price));
        let br = pipeline.world_to_screen(WorldPoint::new(self.bottom_right.timestamp as f64, self.bottom_right.price));
        let x = tl.x.min(br.x); let y = tl.y.min(br.y);
        let w = (br.x - tl.x).abs(); let h = (br.y - tl.y).abs();
        let mut cmds = Vec::with_capacity(2);
        if let Some(fill) = self.fill_color {
            cmds.push(DrawCommand::DrawRect { x, y, width: w, height: h, fill: Some(fill), stroke: None, stroke_width: 0.0, z_index: 5 });
        }
        cmds.push(DrawCommand::DrawRect { x, y, width: w, height: h, fill: None, stroke: Some(self.color), stroke_width: self.width, z_index: 6 });
        cmds
    }
}

impl RenderableDrawing for fc_domain::drawing::Ellipse {
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let center_screen = pipeline.world_to_screen(WorldPoint::new(self.center.timestamp as f64, self.center.price));
        let right = pipeline.world_to_screen(WorldPoint::new(self.center.timestamp as f64 + self.radius_x, self.center.price));
        let bottom = pipeline.world_to_screen(WorldPoint::new(self.center.timestamp as f64, self.center.price + self.radius_y));
        let rx = (right.x - center_screen.x).abs();
        let ry = (bottom.y - center_screen.y).abs();
        let radius = (rx + ry) / 2.0;
        let mut cmds = Vec::with_capacity(2);
        if let Some(fill) = self.fill_color {
            cmds.push(DrawCommand::DrawCircle { cx: center_screen.x, cy: center_screen.y, radius, fill: Some(fill), stroke: None, stroke_width: 0.0, z_index: 5 });
        }
        cmds.push(DrawCommand::DrawCircle { cx: center_screen.x, cy: center_screen.y, radius, fill: None, stroke: Some(self.color), stroke_width: self.width, z_index: 6 });
        cmds
    }
}

// ---------------------------------------------------------------------------
// Fibonacci types
// ---------------------------------------------------------------------------

impl RenderableDrawing for fc_domain::drawing::FibonacciRetracement {
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let mut cmds = Vec::new();
        let left = ctx.time_range.0;
        let right = ctx.time_range.1;
        for &level in &self.levels {
            let price = self.price_at_level(level);
            let s1 = ctx.pipeline.world_to_screen(WorldPoint::new(left, price));
            let s2 = ctx.pipeline.world_to_screen(WorldPoint::new(right, price));
            cmds.push(DrawCommand::DrawLine { x0: s1.x, y0: s1.y, x1: s2.x, y1: s2.y, color: self.color, width: self.width, style: self.style, z_index: 9 });
        }
        cmds
    }
}

impl RenderableDrawing for fc_domain::drawing::FibonacciExtension {
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let mut cmds = Vec::new();
        let left = ctx.time_range.0;
        let right = ctx.time_range.1;
        for &level in &self.levels {
            let price = self.price_at_level(level);
            let s1 = ctx.pipeline.world_to_screen(WorldPoint::new(left, price));
            let s2 = ctx.pipeline.world_to_screen(WorldPoint::new(right, price));
            cmds.push(DrawCommand::DrawLine { x0: s1.x, y0: s1.y, x1: s2.x, y1: s2.y, color: self.color, width: self.width, style: self.style, z_index: 9 });
        }
        cmds
    }
}

// ---------------------------------------------------------------------------
// Pitchfork — three-point tool with median + parallels
// ---------------------------------------------------------------------------

impl RenderableDrawing for fc_domain::drawing::Pitchfork {
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let sa = pipeline.world_to_screen(WorldPoint::new(self.point_a.timestamp as f64, self.point_a.price));
        let sb = pipeline.world_to_screen(WorldPoint::new(self.point_b.timestamp as f64, self.point_b.price));
        let sc = pipeline.world_to_screen(WorldPoint::new(self.point_c.timestamp as f64, self.point_c.price));

        // Median line: point_a to midpoint of B and C
        let mid_x = (sb.x + sc.x) / 2.0;
        let mid_y = (sb.y + sc.y) / 2.0;

        let mut cmds = vec![
            DrawCommand::DrawLine { x0: sa.x, y0: sa.y, x1: mid_x, y1: mid_y, color: self.color, width: self.width, style: self.style, z_index: 10 },
            DrawCommand::DrawLine { x0: sa.x, y0: sa.y, x1: sb.x, y1: sb.y, color: self.color, width: self.width, style: self.style, z_index: 10 },
            DrawCommand::DrawLine { x0: sa.x, y0: sa.y, x1: sc.x, y1: sc.y, color: self.color, width: self.width, style: self.style, z_index: 10 },
        ];

        // Upper and lower parallels through B and C
        let dx = mid_x - sa.x; let dy = mid_y - sa.y;
        cmds.push(DrawCommand::DrawLine { x0: sb.x, y0: sb.y, x1: sb.x + dx, y1: sb.y + dy, color: self.color, width: self.width, style: self.style, z_index: 10 });
        cmds.push(DrawCommand::DrawLine { x0: sc.x, y0: sc.y, x1: sc.x + dx, y1: sc.y + dy, color: self.color, width: self.width, style: self.style, z_index: 10 });

        cmds
    }
}

// ---------------------------------------------------------------------------
// Text/Image/Label renderers
// ---------------------------------------------------------------------------

impl RenderableDrawing for fc_domain::drawing::TextDrawing {
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let screen = pipeline.world_to_screen(WorldPoint::new(self.position.timestamp as f64, self.position.price));
        vec![DrawCommand::DrawText {
            x: screen.x, y: screen.y, text: self.text.clone(),
            color: self.color, font_size: self.font_size,
            align_x: self.align_x, align_y: self.align_y, z_index: 20,
        }]
    }
}

impl RenderableDrawing for fc_domain::drawing::ImageDrawing {
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let screen = pipeline.world_to_screen(WorldPoint::new(self.position.timestamp as f64, self.position.price));
        vec![DrawCommand::DrawImage {
            x: screen.x - self.width / 2.0, y: screen.y - self.height / 2.0,
            src: self.src.clone(), width: self.width, height: self.height,
            opacity: self.opacity, z_index: 15,
        }]
    }
}

impl RenderableDrawing for fc_domain::drawing::LabelDrawing {
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        let pipeline = &ctx.pipeline;
        let screen = pipeline.world_to_screen(WorldPoint::new(self.position.timestamp as f64, self.position.price));
        let char_width = self.font_size * 0.6;
        let text_w = self.text.len() as f32 * char_width + self.padding * 2.0;
        let text_h = self.font_size + self.padding * 2.0;
        let bg_x = screen.x - text_w / 2.0;
        let bg_y = screen.y - text_h / 2.0;
        let mut cmds = vec![
            DrawCommand::DrawRect { x: bg_x, y: bg_y, width: text_w, height: text_h, fill: Some(self.bg_color), stroke: Some(self.border_color), stroke_width: 1.0, z_index: 18 },
            DrawCommand::DrawText { x: bg_x + self.padding, y: bg_y + self.padding, text: self.text.clone(), color: self.text_color, font_size: self.font_size, align_x: 0.0, align_y: 0.0, z_index: 19 },
        ];
        if self.selected {
            cmds.push(DrawCommand::DrawRect { x: bg_x - 2.0, y: bg_y - 2.0, width: text_w + 4.0, height: text_h + 4.0, fill: None, stroke: Some([0.3, 0.6, 1.0, 0.8]), stroke_width: 2.0, z_index: 25 });
        }
        cmds
    }
}

// ---------------------------------------------------------------------------
// Path — polyline / polygon with custom hit_test
// ---------------------------------------------------------------------------

impl RenderableDrawing for fc_domain::drawing::Path {
    fn to_commands(&self, ctx: &RenderContext) -> Vec<DrawCommand> {
        if self.points.is_empty() { return Vec::new(); }
        let screen_points: Vec<[f32; 2]> = self.points.iter()
            .map(|p| { let sp = ctx.pipeline.world_to_screen(WorldPoint::new(p.timestamp as f64, p.price)); [sp.x, sp.y] })
            .collect();
        let mut cmds = Vec::with_capacity(2);
        if self.closed {
            if let Some(fill) = self.fill_color {
                cmds.push(DrawCommand::DrawPath { points: screen_points.clone(), color: [0.0; 4], width: 0.0, style: LineStyle::Solid, closed: true, fill: Some(fill), z_index: 5 });
            }
        }
        cmds.push(DrawCommand::DrawPath { points: screen_points, color: self.color, width: self.width, style: self.style, closed: self.closed, fill: None, z_index: 10 });
        cmds
    }
}
