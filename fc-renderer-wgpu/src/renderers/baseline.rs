use fc_core::render::commands::DrawCommand;
use fc_core::render::series_renderer::{Rect, SeriesHit, SeriesRenderer};

/// A single data point for baseline rendering (screen coordinates).
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    pub x: f32,
    pub y: f32,
}

/// Baseline renderer — line colored differently above/below a baseline,
/// with filled areas from the line to the baseline.
pub struct BaselineRenderer {
    pub above_color: [f32; 4],
    pub below_color: [f32; 4],
    pub width: f32,
    pub baseline_y: f32,
    bounds: Rect,
}

impl BaselineRenderer {
    pub fn new(_color: [f32; 4], width: f32) -> Self {
        Self {
            above_color: [0.0, 0.8, 0.0, 1.0],
            below_color: [0.8, 0.0, 0.0, 1.0],
            width,
            baseline_y: 0.0,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Generate draw commands from screen-space data points.
    pub fn render(&self, points: &[DataPoint], commands: &mut Vec<DrawCommand>) {
        if points.len() < 2 {
            return;
        }

        // Split into above-baseline and below-baseline segments and draw fills.
        // For simplicity: draw the full polyline, then fill above and below regions.
        let line_points: Vec<[f32; 2]> = points.iter().map(|p| [p.x, p.y]).collect();

        // Fill above baseline (where y < baseline_y in screen coords)
        let mut above_pts: Vec<[f32; 2]> = Vec::new();
        let mut below_pts: Vec<[f32; 2]> = Vec::new();

        for p in points {
            if p.y <= self.baseline_y {
                above_pts.push([p.x, p.y]);
            } else {
                below_pts.push([p.x, p.y]);
            }
        }

        if above_pts.len() >= 2 {
            let mut fill = above_pts;
            fill.push([fill.last().unwrap()[0], self.baseline_y]);
            fill.push([fill.first().unwrap()[0], self.baseline_y]);
            commands.push(DrawCommand::filled_polygon(
                fill,
                [self.above_color[0], self.above_color[1], self.above_color[2], 0.15],
                630,
            ));
        }

        if below_pts.len() >= 2 {
            let mut fill = below_pts;
            fill.push([fill.last().unwrap()[0], self.baseline_y]);
            fill.push([fill.first().unwrap()[0], self.baseline_y]);
            commands.push(DrawCommand::filled_polygon(
                fill,
                [self.below_color[0], self.below_color[1], self.below_color[2], 0.15],
                630,
            ));
        }

        // Stroke the full line
        commands.push(DrawCommand::polyline(
            line_points,
            [0.6, 0.6, 0.6, 1.0],
            self.width,
            650,
        ));
    }
}

impl Default for BaselineRenderer {
    fn default() -> Self {
        Self::new([0.2, 0.6, 1.0, 1.0], 1.5)
    }
}

impl SeriesRenderer for BaselineRenderer {
    fn update(
        &mut self,
        _data: &[DrawCommand],
        bounds: Rect,
    ) -> Vec<DrawCommand> {
        self.bounds = bounds;
        Vec::new()
    }

    fn hit_test(&self, x: f32, y: f32) -> Option<SeriesHit> {
        if self.bounds.contains(x, y) {
            Some(SeriesHit {
                index: 0,
                distance: 0.0,
            })
        } else {
            None
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn layer_z_index(&self) -> i32 {
        630
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_renderer() {
        let r = BaselineRenderer::new([1.0; 4], 2.0);
        assert_eq!(r.above_color, [0.0, 0.8, 0.0, 1.0]);
        assert_eq!(r.below_color, [0.8, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn render_produces_commands() {
        let r = BaselineRenderer::new([1.0; 4], 1.5);
        let mut cmds = Vec::new();
        let pts = vec![
            DataPoint { x: 0.0, y: 80.0 },
            DataPoint { x: 50.0, y: 120.0 },
            DataPoint { x: 100.0, y: 60.0 },
        ];
        r.render(&pts, &mut cmds);
        assert!(!cmds.is_empty());
    }

    #[test]
    fn render_empty_data() {
        let r = BaselineRenderer::new([1.0; 4], 1.5);
        let mut cmds = Vec::new();
        r.render(&[], &mut cmds);
        assert!(cmds.is_empty());
    }

    #[test]
    fn render_single_point_produces_nothing() {
        let r = BaselineRenderer::new([1.0; 4], 1.5);
        let mut cmds = Vec::new();
        r.render(&[DataPoint { x: 0.0, y: 10.0 }], &mut cmds);
        assert!(cmds.is_empty());
    }
}
