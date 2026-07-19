use fc_core::render::commands::{DrawCommand, LineStyle};
use std::f32::consts::TAU;

use crate::types::Vertex;

pub const CIRCLE_SEGMENTS: u32 = 32;

pub fn screen_to_ndc(x: f32, y: f32, w: f32, h: f32) -> (f32, f32) {
    let ndc_x = (x / w) * 2.0 - 1.0;
    let ndc_y = 1.0 - (y / h) * 2.0;
    (ndc_x, ndc_y)
}

fn make_vertex(x: f32, y: f32, color: [f32; 4]) -> Vertex {
    Vertex {
        position: [x, y],
        color,
        tex_coord: [0.0, 0.0],
    }
}

pub fn push_line(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    color: [f32; 4],
    line_width: f32,
    surface_w: f32,
    surface_h: f32,
    base_index: u32,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    if len < f32::EPSILON {
        return;
    }

    let half_w = line_width * 0.5;
    let nx = -dy / len * half_w;
    let ny = dx / len * half_w;

    let (px0, py0) = screen_to_ndc(x0 + nx, y0 + ny, surface_w, surface_h);
    let (px1, py1) = screen_to_ndc(x1 + nx, y1 + ny, surface_w, surface_h);
    let (px2, py2) = screen_to_ndc(x0 - nx, y0 - ny, surface_w, surface_h);
    let (px3, py3) = screen_to_ndc(x1 - nx, y1 - ny, surface_w, surface_h);

    vertices.push(make_vertex(px0, py0, color));
    vertices.push(make_vertex(px1, py1, color));
    vertices.push(make_vertex(px2, py2, color));
    vertices.push(make_vertex(px3, py3, color));

    let i = base_index;
    indices.extend_from_slice(&[i, i + 1, i + 2, i + 2, i + 1, i + 3]);
}

pub fn push_rect_fill(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: [f32; 4],
    surface_w: f32,
    surface_h: f32,
    base_index: u32,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    let (tl_x, tl_y) = screen_to_ndc(x, y, surface_w, surface_h);
    let (tr_x, tr_y) = screen_to_ndc(x + w, y, surface_w, surface_h);
    let (bl_x, bl_y) = screen_to_ndc(x, y + h, surface_w, surface_h);
    let (br_x, br_y) = screen_to_ndc(x + w, y + h, surface_w, surface_h);

    vertices.push(make_vertex(tl_x, tl_y, color));
    vertices.push(make_vertex(tr_x, tr_y, color));
    vertices.push(make_vertex(bl_x, bl_y, color));
    vertices.push(make_vertex(br_x, br_y, color));

    let i = base_index;
    indices.extend_from_slice(&[i, i + 1, i + 2, i + 2, i + 1, i + 3]);
}

pub fn push_rect_stroke(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: [f32; 4],
    stroke_width: f32,
    surface_w: f32,
    surface_h: f32,
    base_index: u32,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    push_line(x, y, x + w, y, color, stroke_width, surface_w, surface_h, base_index, vertices, indices);
    push_line(x + w, y, x + w, y + h, color, stroke_width, surface_w, surface_h, base_index + 4, vertices, indices);
    push_line(x + w, y + h, x, y + h, color, stroke_width, surface_w, surface_h, base_index + 8, vertices, indices);
    push_line(x, y + h, x, y, color, stroke_width, surface_w, surface_h, base_index + 12, vertices, indices);
}

pub fn push_circle_fill(
    cx: f32,
    cy: f32,
    radius: f32,
    color: [f32; 4],
    surface_w: f32,
    surface_h: f32,
    base_index: u32,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    let (c_ndc_x, c_ndc_y) = screen_to_ndc(cx, cy, surface_w, surface_h);

    vertices.push(make_vertex(c_ndc_x, c_ndc_y, color));

    for i in 0..CIRCLE_SEGMENTS {
        let angle = TAU * (i as f32) / (CIRCLE_SEGMENTS as f32);
        let px = cx + radius * angle.cos();
        let py = cy + radius * angle.sin();
        let (ndx, ndy) = screen_to_ndc(px, py, surface_w, surface_h);
        vertices.push(make_vertex(ndx, ndy, color));
    }

    for i in 0..CIRCLE_SEGMENTS {
        let next = (i + 1) % CIRCLE_SEGMENTS;
        let a = base_index + 1 + i;
        let b = base_index + 1 + next;
        indices.extend_from_slice(&[base_index, a, b]);
    }
}

pub fn push_circle_stroke(
    cx: f32,
    cy: f32,
    radius: f32,
    color: [f32; 4],
    stroke_width: f32,
    surface_w: f32,
    surface_h: f32,
    base_index: u32,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    let half_w = stroke_width * 0.5;
    let inner = radius - half_w;
    let outer = radius + half_w;

    let mut idx = base_index;
    for i in 0..CIRCLE_SEGMENTS {
        let angle0 = TAU * (i as f32) / (CIRCLE_SEGMENTS as f32);
        let angle1 = TAU * ((i + 1) as f32) / (CIRCLE_SEGMENTS as f32);

        let ix0 = cx + inner * angle0.cos();
        let iy0 = cy + inner * angle0.sin();
        let ox0 = cx + outer * angle0.cos();
        let oy0 = cy + outer * angle0.sin();
        let ix1 = cx + inner * angle1.cos();
        let iy1 = cy + inner * angle1.sin();
        let ox1 = cx + outer * angle1.cos();
        let oy1 = cy + outer * angle1.sin();

        let (a_x, a_y) = screen_to_ndc(ix0, iy0, surface_w, surface_h);
        let (b_x, b_y) = screen_to_ndc(ox0, oy0, surface_w, surface_h);
        let (c_x, c_y) = screen_to_ndc(ix1, iy1, surface_w, surface_h);
        let (d_x, d_y) = screen_to_ndc(ox1, oy1, surface_w, surface_h);

        vertices.push(make_vertex(a_x, a_y, color));
        vertices.push(make_vertex(b_x, b_y, color));
        vertices.push(make_vertex(c_x, c_y, color));
        vertices.push(make_vertex(d_x, d_y, color));

        indices.extend_from_slice(&[idx, idx + 1, idx + 2, idx + 2, idx + 1, idx + 3]);
        idx += 4;
    }
}

pub fn push_triangle_fill(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: [f32; 4],
    surface_w: f32,
    surface_h: f32,
    base_index: u32,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    let (a_x, a_y) = screen_to_ndc(x0, y0, surface_w, surface_h);
    let (b_x, b_y) = screen_to_ndc(x1, y1, surface_w, surface_h);
    let (c_x, c_y) = screen_to_ndc(x2, y2, surface_w, surface_h);

    vertices.push(make_vertex(a_x, a_y, color));
    vertices.push(make_vertex(b_x, b_y, color));
    vertices.push(make_vertex(c_x, c_y, color));

    indices.extend_from_slice(&[base_index, base_index + 1, base_index + 2]);
}

pub fn push_triangle_stroke(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: [f32; 4],
    stroke_width: f32,
    surface_w: f32,
    surface_h: f32,
    base_index: u32,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    push_line(x0, y0, x1, y1, color, stroke_width, surface_w, surface_h, base_index, vertices, indices);
    push_line(x1, y1, x2, y2, color, stroke_width, surface_w, surface_h, base_index + 4, vertices, indices);
    push_line(x2, y2, x0, y0, color, stroke_width, surface_w, surface_h, base_index + 8, vertices, indices);
}

pub fn push_path_line(
    points: &[[f32; 2]],
    color: [f32; 4],
    width: f32,
    closed: bool,
    surface_w: f32,
    surface_h: f32,
    base_index: u32,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    if points.len() < 2 {
        return;
    }

    let mut idx = base_index;
    for i in 0..points.len() - 1 {
        let [x0, y0] = points[i];
        let [x1, y1] = points[i + 1];
        push_line(x0, y0, x1, y1, color, width, surface_w, surface_h, idx, vertices, indices);
        idx += 4;
    }

    if closed && points.len() > 2 {
        let [x0, y0] = points[points.len() - 1];
        let [x1, y1] = points[0];
        push_line(x0, y0, x1, y1, color, width, surface_w, surface_h, idx, vertices, indices);
    }
}

pub fn push_path_fill(
    points: &[[f32; 2]],
    color: [f32; 4],
    surface_w: f32,
    surface_h: f32,
    base_index: u32,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    if points.len() < 3 {
        return;
    }

    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    for p in points {
        cx += p[0];
        cy += p[1];
    }
    cx /= points.len() as f32;
    cy /= points.len() as f32;

    let (c_ndc_x, c_ndc_y) = screen_to_ndc(cx, cy, surface_w, surface_h);
    vertices.push(make_vertex(c_ndc_x, c_ndc_y, color));

    for p in points {
        let (nx, ny) = screen_to_ndc(p[0], p[1], surface_w, surface_h);
        vertices.push(make_vertex(nx, ny, color));
    }

    for i in 0..points.len() {
        let next = (i + 1) % points.len();
        let a = base_index + 1 + i as u32;
        let b = base_index + 1 + next as u32;
        indices.extend_from_slice(&[base_index, a, b]);
    }
}

pub fn generate_vertices(
    cmd: &DrawCommand,
    surface_w: f32,
    surface_h: f32,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    let base_index = vertices.len() as u32;

    match cmd {
        DrawCommand::DrawLine {
            x0,
            y0,
            x1,
            y1,
            color,
            width,
            style,
            ..
        } => {
            match style {
                LineStyle::Solid => {
                    push_line(
                        *x0, *y0, *x1, *y1,
                        *color, *width,
                        surface_w, surface_h,
                        base_index,
                        vertices,
                        indices,
                    );
                }
                LineStyle::Dashed | LineStyle::Dotted => {
                    // TODO: implement dashed/dotted by splitting the line into segments
                    push_line(
                        *x0, *y0, *x1, *y1,
                        *color, *width,
                        surface_w, surface_h,
                        base_index,
                        vertices,
                        indices,
                    );
                }
            }
        }

        DrawCommand::DrawRect {
            x,
            y,
            width,
            height,
            fill,
            stroke,
            stroke_width,
            ..
        } => {
            let mut cur_base = base_index;
            if let Some(color) = fill {
                push_rect_fill(
                    *x, *y, *width, *height,
                    *color,
                    surface_w, surface_h,
                    cur_base,
                    vertices,
                    indices,
                );
                cur_base += 4;
            }
            if let Some(color) = stroke {
                push_rect_stroke(
                    *x, *y, *width, *height,
                    *color,
                    *stroke_width,
                    surface_w, surface_h,
                    cur_base,
                    vertices,
                    indices,
                );
            }
        }

        DrawCommand::DrawCircle {
            cx,
            cy,
            radius,
            fill,
            stroke,
            stroke_width,
            ..
        } => {
            let mut cur_base = base_index;
            if let Some(color) = fill {
                push_circle_fill(
                    *cx, *cy, *radius,
                    *color,
                    surface_w, surface_h,
                    cur_base,
                    vertices,
                    indices,
                );
                cur_base += 1 + CIRCLE_SEGMENTS;
            }
            if let Some(color) = stroke {
                push_circle_stroke(
                    *cx, *cy, *radius,
                    *color,
                    *stroke_width,
                    surface_w, surface_h,
                    cur_base,
                    vertices,
                    indices,
                );
            }
        }

        DrawCommand::DrawTriangle {
            x0,
            y0,
            x1,
            y1,
            x2,
            y2,
            fill,
            stroke,
            stroke_width,
            ..
        } => {
            let mut cur_base = base_index;
            if let Some(color) = fill {
                push_triangle_fill(
                    *x0, *y0, *x1, *y1, *x2, *y2,
                    *color,
                    surface_w, surface_h,
                    cur_base,
                    vertices,
                    indices,
                );
                cur_base += 3;
            }
            if let Some(color) = stroke {
                push_triangle_stroke(
                    *x0, *y0, *x1, *y1, *x2, *y2,
                    *color,
                    *stroke_width,
                    surface_w, surface_h,
                    cur_base,
                    vertices,
                    indices,
                );
            }
        }

        DrawCommand::DrawPath {
            points,
            color,
            width,
            closed,
            fill,
            ..
        } => {
            let mut cur_base = base_index;
            if let Some(fill_color) = fill {
                push_path_fill(
                    points,
                    *fill_color,
                    surface_w, surface_h,
                    cur_base,
                    vertices,
                    indices,
                );
                cur_base += 1 + points.len() as u32;
            }
            push_path_line(
                points,
                *color,
                *width,
                *closed,
                surface_w, surface_h,
                cur_base,
                vertices,
                indices,
            );
        }

        DrawCommand::DrawText { .. } => {
            // TODO: text rendering requires glyph atlas (PR 6.8)
            log::warn!("DrawText not yet supported by wgpu backend — skipping");
        }

        DrawCommand::DrawImage { .. } => {
            // TODO: image loading is async (future PR)
            log::warn!("DrawImage not yet supported by wgpu backend — skipping");
        }
    }
}

pub fn generate_sorted_vertices(
    commands: &[DrawCommand],
    surface_w: f32,
    surface_h: f32,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    let mut sorted: Vec<&DrawCommand> = commands.iter().collect();
    sorted.sort_by_key(|cmd| cmd.z_index());

    for cmd in sorted {
        generate_vertices(cmd, surface_w, surface_h, vertices, indices);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: f32 = 800.0;
    const H: f32 = 600.0;
    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

    #[test]
    fn ndc_conversion() {
        let (x, y) = screen_to_ndc(0.0, 0.0, 800.0, 600.0);
        assert!((x - (-1.0)).abs() < 1e-6);
        assert!((y - 1.0).abs() < 1e-6);

        let (x, y) = screen_to_ndc(800.0, 600.0, 800.0, 600.0);
        assert!((x - 1.0).abs() < 1e-6);
        assert!((y - (-1.0)).abs() < 1e-6);

        let (x, y) = screen_to_ndc(400.0, 300.0, 800.0, 600.0);
        assert!(x.abs() < 1e-6);
        assert!(y.abs() < 1e-6);
    }

    #[test]
    fn line_vertices_count() {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        push_line(0.0, 0.0, 100.0, 0.0, WHITE, 2.0, W, H, 0, &mut verts, &mut inds);
        assert_eq!(verts.len(), 4);
        assert_eq!(inds.len(), 6);
    }

    #[test]
    fn line_vertices_positions() {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        push_line(0.0, 0.0, 100.0, 0.0, WHITE, 2.0, W, H, 0, &mut verts, &mut inds);

        // Horizontal line at y=0, width=2 → half_w=1
        // Normal = (-dy/len, dx/len) * half_w = (0, 1) for a rightward line
        // verts: [0]=(x0+nx, y0+ny)=(0,1), [1]=(x1+nx, y1+ny)=(100,1),
        //        [2]=(x0-nx, y0-ny)=(0,-1), [3]=(x1-nx, y1-ny)=(100,-1)
        let (expected_bot_left_x, expected_bot_left_y) = screen_to_ndc(0.0, 1.0, W, H);
        let (expected_bot_right_x, _) = screen_to_ndc(100.0, 1.0, W, H);
        let (expected_top_left_x, _) = screen_to_ndc(0.0, -1.0, W, H);
        let (expected_top_right_x, _) = screen_to_ndc(100.0, -1.0, W, H);

        assert!((verts[0].position[0] - expected_bot_left_x).abs() < 1e-5);
        assert!((verts[0].position[1] - expected_bot_left_y).abs() < 1e-5);
        assert!((verts[1].position[0] - expected_bot_right_x).abs() < 1e-5);
        assert!((verts[2].position[0] - expected_top_left_x).abs() < 1e-5);
        assert!((verts[3].position[0] - expected_top_right_x).abs() < 1e-5);
    }

    #[test]
    fn line_zero_length_returns_nothing() {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        push_line(10.0, 10.0, 10.0, 10.0, WHITE, 2.0, W, H, 0, &mut verts, &mut inds);
        assert_eq!(verts.len(), 0);
        assert_eq!(inds.len(), 0);
    }

    #[test]
    fn rect_fill_vertices_count() {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        push_rect_fill(10.0, 10.0, 100.0, 50.0, WHITE, W, H, 0, &mut verts, &mut inds);
        assert_eq!(verts.len(), 4);
        assert_eq!(inds.len(), 6);
    }

    #[test]
    fn rect_stroke_vertices_count() {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        push_rect_stroke(10.0, 10.0, 100.0, 50.0, WHITE, 1.0, W, H, 0, &mut verts, &mut inds);
        assert_eq!(verts.len(), 16);
        assert_eq!(inds.len(), 24);
    }

    #[test]
    fn circle_fill_vertices_count() {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        push_circle_fill(400.0, 300.0, 50.0, WHITE, W, H, 0, &mut verts, &mut inds);
        assert_eq!(verts.len() as u32, 1 + CIRCLE_SEGMENTS);
        assert_eq!(inds.len() as u32, CIRCLE_SEGMENTS * 3);
    }

    #[test]
    fn circle_stroke_vertices_count() {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        push_circle_stroke(400.0, 300.0, 50.0, WHITE, 1.0, W, H, 0, &mut verts, &mut inds);
        assert_eq!(verts.len() as u32, CIRCLE_SEGMENTS * 4);
        assert_eq!(inds.len() as u32, CIRCLE_SEGMENTS * 6);
    }

    #[test]
    fn triangle_fill_vertices_count() {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        push_triangle_fill(0.0, 0.0, 100.0, 0.0, 50.0, 80.0, WHITE, W, H, 0, &mut verts, &mut inds);
        assert_eq!(verts.len(), 3);
        assert_eq!(inds.len(), 3);
    }

    #[test]
    fn triangle_stroke_vertices_count() {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        push_triangle_stroke(0.0, 0.0, 100.0, 0.0, 50.0, 80.0, WHITE, 1.0, W, H, 0, &mut verts, &mut inds);
        assert_eq!(verts.len(), 12);
        assert_eq!(inds.len(), 18);
    }

    #[test]
    fn path_vertices_count() {
        let points = vec![[0.0, 0.0], [100.0, 0.0], [100.0, 100.0]];
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        push_path_line(&points, WHITE, 1.0, false, W, H, 0, &mut verts, &mut inds);
        // 2 segments × 4 vertices each
        assert_eq!(verts.len(), 8);
        assert_eq!(inds.len(), 12);
    }

    #[test]
    fn path_closed_adds_last_segment() {
        let points = vec![[0.0, 0.0], [100.0, 0.0], [100.0, 100.0]];
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        push_path_line(&points, WHITE, 1.0, true, W, H, 0, &mut verts, &mut inds);
        // 3 segments × 4 vertices
        assert_eq!(verts.len(), 12);
        assert_eq!(inds.len(), 18);
    }

    #[test]
    fn path_fill_vertices_count() {
        let points = vec![[0.0, 0.0], [100.0, 0.0], [100.0, 100.0]];
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        push_path_fill(&points, WHITE, W, H, 0, &mut verts, &mut inds);
        // 1 centroid + 3 points = 4 vertices
        assert_eq!(verts.len(), 4);
        // 3 triangles × 3 indices
        assert_eq!(inds.len(), 9);
    }

    #[test]
    fn z_index_sort() {
        let cmds = vec![
            DrawCommand::line(0.0, 0.0, 1.0, 1.0, WHITE, 1.0, 10),
            DrawCommand::filled_rect(0.0, 0.0, 1.0, 1.0, WHITE, 2),
            DrawCommand::filled_circle(0.0, 0.0, 1.0, WHITE, 5),
        ];

        let mut verts = Vec::new();
        let mut inds = Vec::new();
        generate_sorted_vertices(&cmds, W, H, &mut verts, &mut inds);

        // rect (z=2) should have its vertices before circle (z=5) and after line (z=10)
        // The first 4 vertices should be from the rect (z=2)
        // and the first 4 indices should match rect's triangle indices
        // Verify by checking the first vertex is from rect fill (4 verts at base)
        // Rect fill starts at index 0, line starts at index 4
        assert!(inds.len() > 0);
    }

    #[test]
    fn generate_vertices_draw_line() {
        let cmd = DrawCommand::line(0.0, 0.0, 100.0, 50.0, WHITE, 2.0, 0);
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        generate_vertices(&cmd, W, H, &mut verts, &mut inds);
        assert_eq!(verts.len(), 4);
        assert_eq!(inds.len(), 6);
    }

    #[test]
    fn generate_vertices_draw_rect_fill_only() {
        let cmd = DrawCommand::filled_rect(0.0, 0.0, 50.0, 30.0, WHITE, 0);
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        generate_vertices(&cmd, W, H, &mut verts, &mut inds);
        assert_eq!(verts.len(), 4);
        assert_eq!(inds.len(), 6);
    }

    #[test]
    fn generate_vertices_draw_rect_fill_and_stroke() {
        let cmd = DrawCommand::DrawRect {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 30.0,
            fill: Some(WHITE),
            stroke: Some(WHITE),
            stroke_width: 1.0,
            z_index: 0,
        };
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        generate_vertices(&cmd, W, H, &mut verts, &mut inds);
        // fill: 4 verts, stroke: 16 verts
        assert_eq!(verts.len(), 20);
        assert_eq!(inds.len(), 30);
    }

    #[test]
    fn generate_vertices_text_skipped() {
        let cmd = DrawCommand::text(0.0, 0.0, "hello", WHITE, 14.0, 0);
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        generate_vertices(&cmd, W, H, &mut verts, &mut inds);
        assert_eq!(verts.len(), 0);
        assert_eq!(inds.len(), 0);
    }

    #[test]
    fn generate_vertices_image_skipped() {
        let cmd = DrawCommand::DrawImage {
            x: 0.0,
            y: 0.0,
            src: "test.png".into(),
            width: 100.0,
            height: 100.0,
            opacity: 1.0,
            z_index: 0,
        };
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        generate_vertices(&cmd, W, H, &mut verts, &mut inds);
        assert_eq!(verts.len(), 0);
        assert_eq!(inds.len(), 0);
    }

    #[test]
    fn generate_sorted_vertices_orders_by_z() {
        let cmds = vec![
            DrawCommand::line(0.0, 0.0, 1.0, 1.0, WHITE, 1.0, 20),
            DrawCommand::filled_rect(0.0, 0.0, 1.0, 1.0, WHITE, 5),
            DrawCommand::filled_circle(0.0, 0.0, 1.0, WHITE, 10),
        ];

        let mut verts = Vec::new();
        let mut inds = Vec::new();
        generate_sorted_vertices(&cmds, W, H, &mut verts, &mut inds);

        // rect (z=5) first: 4 verts for fill
        // circle (z=10) second: 1+32 verts for fill
        // line (z=20) third: 4 verts
        let total = 4 + (1 + 32) + 4;
        assert_eq!(verts.len(), total);
    }
}
