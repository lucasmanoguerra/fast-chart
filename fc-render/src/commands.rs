// ---------------------------------------------------------------------------
// DrawCommand — universal render queue primitives
// ---------------------------------------------------------------------------

use std::fmt;

// LineStyle canonical definition lives in fc-primitives.
pub use fc_primitives::LineStyle;

/// A single draw primitive that any renderer backend can execute.
///
/// `DrawCommand` is the universal output of the rendering pipeline.
/// Series renderers produce `Vec<DrawCommand>` and the backend executes them.
///
/// All positions use screen-space pixels with origin at top-left (0, 0).
/// Colors are `[r, g, b, a]` in linear float (0.0–1.0).
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// A line segment between two screen-space points.
    DrawLine {
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color: [f32; 4],
        width: f32,
        style: LineStyle,
        z_index: i32,
    },

    /// A filled or stroked rectangle.
    DrawRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        fill: Option<[f32; 4]>,
        stroke: Option<[f32; 4]>,
        stroke_width: f32,
        z_index: i32,
    },

    /// A circle (or ellipse via non-uniform scale) centered at (cx, cy).
    DrawCircle {
        cx: f32,
        cy: f32,
        radius: f32,
        fill: Option<[f32; 4]>,
        stroke: Option<[f32; 4]>,
        stroke_width: f32,
        z_index: i32,
    },

    /// A filled or stroked triangle.
    DrawTriangle {
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        fill: Option<[f32; 4]>,
        stroke: Option<[f32; 4]>,
        stroke_width: f32,
        z_index: i32,
    },

    /// A polyline or polygon.
    ///
    /// When `closed` is true, the last vertex connects back to the first.
    /// When `filled`, the interior is filled with the given color.
    DrawPath {
        points: Vec<[f32; 2]>,
        color: [f32; 4],
        width: f32,
        style: LineStyle,
        closed: bool,
        fill: Option<[f32; 4]>,
        z_index: i32,
    },

    /// A text label at a screen-space position.
    DrawText {
        x: f32,
        y: f32,
        text: String,
        color: [f32; 4],
        font_size: f32,
        /// Horizontal alignment: 0.0 = left, 0.5 = center, 1.0 = right
        align_x: f32,
        /// Vertical alignment: 0.0 = top, 0.5 = center, 1.0 = bottom
        align_y: f32,
        z_index: i32,
    },

    /// An image placed at screen-space (x, y) with given dimensions.
    DrawImage {
        x: f32,
        y: f32,
        src: String,
        width: f32,
        height: f32,
        opacity: f32,
        z_index: i32,
    },
}

// ---------------------------------------------------------------------------
// Convenience constructors
// ---------------------------------------------------------------------------

impl DrawCommand {
    /// Draw a solid line between two points.
    pub fn line(
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color: [f32; 4],
        width: f32,
        z_index: i32,
    ) -> Self {
        Self::DrawLine {
            x0,
            y0,
            x1,
            y1,
            color,
            width,
            style: LineStyle::Solid,
            z_index,
        }
    }

    /// Draw a dashed line between two points.
    pub fn dashed_line(
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color: [f32; 4],
        width: f32,
        z_index: i32,
    ) -> Self {
        Self::DrawLine {
            x0,
            y0,
            x1,
            y1,
            color,
            width,
            style: LineStyle::Dashed,
            z_index,
        }
    }

    /// Draw a filled rectangle with no stroke.
    pub fn filled_rect(x: f32, y: f32, w: f32, h: f32, color: [f32; 4], z_index: i32) -> Self {
        Self::DrawRect {
            x,
            y,
            width: w,
            height: h,
            fill: Some(color),
            stroke: None,
            stroke_width: 0.0,
            z_index,
        }
    }

    /// Draw a stroked rectangle with no fill.
    pub fn stroked_rect(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        stroke_width: f32,
        z_index: i32,
    ) -> Self {
        Self::DrawRect {
            x,
            y,
            width: w,
            height: h,
            fill: None,
            stroke: Some(color),
            stroke_width,
            z_index,
        }
    }

    /// Draw a filled circle with no stroke.
    pub fn filled_circle(cx: f32, cy: f32, radius: f32, color: [f32; 4], z_index: i32) -> Self {
        Self::DrawCircle {
            cx,
            cy,
            radius,
            fill: Some(color),
            stroke: None,
            stroke_width: 0.0,
            z_index,
        }
    }

    /// Draw a filled triangle with no stroke.
    #[allow(clippy::too_many_arguments)]
    pub fn filled_triangle(
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: [f32; 4],
        z_index: i32,
    ) -> Self {
        Self::DrawTriangle {
            x0,
            y0,
            x1,
            y1,
            x2,
            y2,
            fill: Some(color),
            stroke: None,
            stroke_width: 0.0,
            z_index,
        }
    }

    /// Draw a solid polyline (open path, no fill).
    pub fn polyline(points: Vec<[f32; 2]>, color: [f32; 4], width: f32, z_index: i32) -> Self {
        Self::DrawPath {
            points,
            color,
            width,
            style: LineStyle::Solid,
            closed: false,
            fill: None,
            z_index,
        }
    }

    /// Draw a filled polygon (closed path with fill).
    pub fn filled_polygon(points: Vec<[f32; 2]>, fill: [f32; 4], z_index: i32) -> Self {
        Self::DrawPath {
            points,
            color: [0.0, 0.0, 0.0, 0.0],
            width: 0.0,
            style: LineStyle::Solid,
            closed: true,
            fill: Some(fill),
            z_index,
        }
    }

    /// Draw a text label.
    pub fn text(
        x: f32,
        y: f32,
        text: impl Into<String>,
        color: [f32; 4],
        font_size: f32,
        z_index: i32,
    ) -> Self {
        Self::DrawText {
            x,
            y,
            text: text.into(),
            color,
            font_size,
            align_x: 0.0,
            align_y: 0.5,
            z_index,
        }
    }

    /// Get the z-index of this draw command.
    pub fn z_index(&self) -> i32 {
        match self {
            Self::DrawLine { z_index, .. }
            | Self::DrawRect { z_index, .. }
            | Self::DrawCircle { z_index, .. }
            | Self::DrawTriangle { z_index, .. }
            | Self::DrawPath { z_index, .. }
            | Self::DrawText { z_index, .. }
            | Self::DrawImage { z_index, .. } => *z_index,
        }
    }
}

impl fmt::Display for DrawCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DrawLine { .. } => write!(f, "Line"),
            Self::DrawRect { .. } => write!(f, "Rect"),
            Self::DrawCircle { .. } => write!(f, "Circle"),
            Self::DrawTriangle { .. } => write!(f, "Triangle"),
            Self::DrawPath { points, .. } => write!(f, "Path({} pts)", points.len()),
            Self::DrawText { text, .. } => write!(f, "Text(\"{}\")", text),
            Self::DrawImage { src, .. } => write!(f, "Image(\"{}\")", src),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;


    // Clasificación: determinística — verifica line_creates_draw_line
    #[test]
    fn line_creates_draw_line() {
        let cmd = DrawCommand::line(0.0, 0.0, 100.0, 50.0, [1.0; 4], 2.0, 5);
        match cmd {
            DrawCommand::DrawLine {
                x0,
                y0,
                x1,
                y1,
                width,
                z_index,
                style,
                ..
            } => {
                assert_eq!(x0, 0.0);
                assert_eq!(y0, 0.0);
                assert_eq!(x1, 100.0);
                assert_eq!(y1, 50.0);
                assert_eq!(width, 2.0);
                assert_eq!(z_index, 5);
                assert_eq!(style, LineStyle::Solid);
            }
            _ => panic!("expected DrawLine"),
        }
    }

    // Clasificación: determinística — verifica dashed_line_uses_dashed_style
    #[test]
    fn dashed_line_uses_dashed_style() {
        let cmd = DrawCommand::dashed_line(0.0, 0.0, 10.0, 10.0, [1.0; 4], 1.0, 0);
        match cmd {
            DrawCommand::DrawLine { style, .. } => assert_eq!(style, LineStyle::Dashed),
            _ => panic!("expected DrawLine"),
        }
    }

    // Clasificación: determinística — verifica filled_rect_has_fill_no_stroke
    #[test]
    fn filled_rect_has_fill_no_stroke() {
        let cmd = DrawCommand::filled_rect(0.0, 0.0, 50.0, 30.0, [1.0, 0.0, 0.0, 1.0], 3);
        match cmd {
            DrawCommand::DrawRect {
                fill,
                stroke,
                z_index,
                ..
            } => {
                assert_eq!(fill, Some([1.0, 0.0, 0.0, 1.0]));
                assert_eq!(stroke, None);
                assert_eq!(z_index, 3);
            }
            _ => panic!("expected DrawRect"),
        }
    }

    // Clasificación: determinística — verifica stroked_rect_has_stroke_no_fill
    #[test]
    fn stroked_rect_has_stroke_no_fill() {
        let cmd = DrawCommand::stroked_rect(0.0, 0.0, 50.0, 30.0, [0.0; 4], 1.5, 2);
        match cmd {
            DrawCommand::DrawRect {
                fill,
                stroke,
                stroke_width,
                ..
            } => {
                assert_eq!(fill, None);
                assert_eq!(stroke, Some([0.0; 4]));
                assert_eq!(stroke_width, 1.5);
            }
            _ => panic!("expected DrawRect"),
        }
    }

    // Clasificación: determinística — verifica filled_circle_has_fill_no_stroke
    #[test]
    fn filled_circle_has_fill_no_stroke() {
        let cmd = DrawCommand::filled_circle(100.0, 100.0, 5.0, [0.0, 1.0, 0.0, 1.0], 10);
        match cmd {
            DrawCommand::DrawCircle {
                fill, stroke, ..
            } => {
                assert_eq!(fill, Some([0.0, 1.0, 0.0, 1.0]));
                assert_eq!(stroke, None);
            }
            _ => panic!("expected DrawCircle"),
        }
    }

    // Clasificación: determinística — verifica filled_triangle_has_fill_no_stroke
    #[test]
    fn filled_triangle_has_fill_no_stroke() {
        let cmd = DrawCommand::filled_triangle(0.0, 0.0, 10.0, 0.0, 5.0, 10.0, [1.0; 4], 7);
        match cmd {
            DrawCommand::DrawTriangle {
                fill, stroke, z_index, ..
            } => {
                assert_eq!(fill, Some([1.0; 4]));
                assert_eq!(stroke, None);
                assert_eq!(z_index, 7);
            }
            _ => panic!("expected DrawTriangle"),
        }
    }

    // Clasificación: determinística — verifica polyline_is_open_no_fill
    #[test]
    fn polyline_is_open_no_fill() {
        let pts = vec![[0.0, 0.0], [10.0, 5.0], [20.0, 0.0]];
        let cmd = DrawCommand::polyline(pts.clone(), [1.0; 4], 1.0, 0);
        match cmd {
            DrawCommand::DrawPath {
                points,
                closed,
                fill,
                ..
            } => {
                assert_eq!(points, pts);
                assert!(!closed);
                assert!(fill.is_none());
            }
            _ => panic!("expected DrawPath"),
        }
    }

    // Clasificación: determinística — verifica filled_polygon_is_closed_with_fill
    #[test]
    fn filled_polygon_is_closed_with_fill() {
        let pts = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]];
        let cmd = DrawCommand::filled_polygon(pts.clone(), [0.5, 0.5, 0.5, 0.3], 4);
        match cmd {
            DrawCommand::DrawPath {
                points,
                closed,
                fill,
                z_index,
                ..
            } => {
                assert_eq!(points, pts);
                assert!(closed);
                assert_eq!(fill, Some([0.5, 0.5, 0.5, 0.3]));
                assert_eq!(z_index, 4);
            }
            _ => panic!("expected DrawPath"),
        }
    }

    // Clasificación: determinística — verifica text_label
    #[test]
    fn text_label() {
        let cmd = DrawCommand::text(10.0, 20.0, "BTC", [1.0; 4], 14.0, 20);
        match cmd {
            DrawCommand::DrawText {
                x,
                y,
                text,
                font_size,
                align_x,
                align_y,
                z_index,
                ..
            } => {
                assert_eq!(x, 10.0);
                assert_eq!(y, 20.0);
                assert_eq!(text, "BTC");
                assert_eq!(font_size, 14.0);
                assert_eq!(align_x, 0.0); // left by default
                assert_eq!(align_y, 0.5); // center-y by default
                assert_eq!(z_index, 20);
            }
            _ => panic!("expected DrawText"),
        }
    }


    // Clasificación: determinística — verifica z_index_accessor
    #[test]
    fn z_index_accessor() {
        let cmds = [DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 3),
            DrawCommand::filled_rect(0.0, 0.0, 1.0, 1.0, [1.0; 4], 7),
            DrawCommand::filled_circle(0.0, 0.0, 1.0, [1.0; 4], 11),
            DrawCommand::filled_triangle(0.0, 0.0, 1.0, 0.0, 0.5, 1.0, [1.0; 4], 5),
            DrawCommand::polyline(vec![[0.0, 0.0]], [1.0; 4], 1.0, 9),
            DrawCommand::text(0.0, 0.0, "t", [1.0; 4], 12.0, 15)];
        let expected = [3, 7, 11, 5, 9, 15];
        for (cmd, &exp) in cmds.iter().zip(&expected) {
            assert_eq!(cmd.z_index(), exp, "z_index mismatch for {cmd}");
        }
    }


    // Clasificación: determinística — verifica line_style_default_is_solid
    #[test]
    fn line_style_default_is_solid() {
        assert_eq!(LineStyle::default(), LineStyle::Solid);
    }

    // Clasificación: determinística — verifica que variantes de LineStyle son distinguibles
    #[test]
    fn line_style_equality() {
        assert_eq!(LineStyle::Solid, LineStyle::Solid);
        assert_ne!(LineStyle::Solid, LineStyle::Dashed);
        assert_ne!(LineStyle::Dashed, LineStyle::Dotted);
    }


    // Clasificación: determinística — verifica display_format
    #[test]
    fn display_format() {
        let line = DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 0);
        assert_eq!(line.to_string(), "Line");

        let rect = DrawCommand::filled_rect(0.0, 0.0, 1.0, 1.0, [1.0; 4], 0);
        assert_eq!(rect.to_string(), "Rect");

        let circle = DrawCommand::filled_circle(0.0, 0.0, 1.0, [1.0; 4], 0);
        assert_eq!(circle.to_string(), "Circle");

        let tri = DrawCommand::filled_triangle(0.0, 0.0, 1.0, 0.0, 0.5, 1.0, [1.0; 4], 0);
        assert_eq!(tri.to_string(), "Triangle");

        let path = DrawCommand::polyline(vec![[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]], [1.0; 4], 1.0, 0);
        assert_eq!(path.to_string(), "Path(3 pts)");

        let txt = DrawCommand::text(0.0, 0.0, "hello", [1.0; 4], 12.0, 0);
        assert_eq!(txt.to_string(), "Text(\"hello\")");
    }


    // Clasificación: determinística — verifica draw_command_clone
    #[test]
    fn draw_command_clone() {
        let cmd = DrawCommand::line(0.0, 0.0, 100.0, 50.0, [1.0; 4], 2.0, 5);
        let cloned = cmd.clone();
        assert_eq!(cloned.z_index(), 5);
    }

    // Clasificación: determinística — verifica path_clone_preserves_points
    #[test]
    fn path_clone_preserves_points() {
        let pts = vec![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
        let cmd = DrawCommand::polyline(pts.clone(), [1.0; 4], 1.0, 0);
        let cloned = cmd.clone();
        match cloned {
            DrawCommand::DrawPath { points, .. } => assert_eq!(points, pts),
            _ => panic!("expected DrawPath"),
        }
    }


    // Clasificación: determinística — verifica draw_command_debug
    #[test]
    fn draw_command_debug() {
        let cmd = DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 0);
        let dbg = format!("{:?}", cmd);
        assert!(dbg.contains("DrawLine"));
    }
}
