use fc_render::commands::DrawCommand;
use fc_primitives::Rect;
use fc_render::series_renderer::{SeriesHit, SeriesRenderer};

/// A single data point for area rendering (screen coordinates).
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    pub x: f32,
    pub y: f32,
}

/// Area renderer — line with filled polygon below to a baseline.
pub struct AreaRenderer {
    pub color: [f32; 4],
    pub fill_color: [f32; 4],
    pub width: f32,
    pub baseline_y: f32,
    bounds: Rect,
}

impl AreaRenderer {
    pub fn new(color: [f32; 4], width: f32) -> Self {
        let fill_color = [color[0], color[1], color[2], 0.2];
        Self {
            color,
            fill_color,
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

        // Fill polygon: line points + baseline at bottom
        let mut fill_points: Vec<[f32; 2]> = points.iter().map(|p| [p.x, p.y]).collect();
        // Close the polygon along the baseline
        if let (Some(first), Some(last)) = (points.first(), points.last()) {
            fill_points.push([last.x, self.baseline_y]);
            fill_points.push([first.x, self.baseline_y]);
        }

        commands.push(DrawCommand::filled_polygon(
            fill_points,
            self.fill_color,
            640,
        ));

        // Stroke line on top
        let line_points: Vec<[f32; 2]> = points.iter().map(|p| [p.x, p.y]).collect();
        commands.push(DrawCommand::polyline(
            line_points,
            self.color,
            self.width,
            650,
        ));
    }
}

impl Default for AreaRenderer {
    fn default() -> Self {
        Self::new([0.2, 0.6, 1.0, 1.0], 1.5)
    }
}

impl SeriesRenderer for AreaRenderer {
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
        640
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_renderer() {
        let r = AreaRenderer::new([1.0, 0.0, 0.0, 1.0], 2.0);
        assert_eq!(r.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(r.width, 2.0);
    }

    #[test]
    fn render_produces_commands() {
        let r = AreaRenderer::new([1.0; 4], 1.5);
        let mut cmds = Vec::new();
        let pts = vec![
            DataPoint { x: 0.0, y: 10.0 },
            DataPoint { x: 50.0, y: 20.0 },
            DataPoint { x: 100.0, y: 15.0 },
        ];
        r.render(&pts, &mut cmds);
        // 2 commands: fill polygon + stroke line
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn render_empty_data() {
        let r = AreaRenderer::new([1.0; 4], 1.5);
        let mut cmds = Vec::new();
        r.render(&[], &mut cmds);
        assert!(cmds.is_empty());
    }

    #[test]
    fn render_single_point_produces_nothing() {
        let r = AreaRenderer::new([1.0; 4], 1.5);
        let mut cmds = Vec::new();
        r.render(&[DataPoint { x: 0.0, y: 10.0 }], &mut cmds);
        assert!(cmds.is_empty());
    }

    #[test]
    fn fill_polygon_is_closed() {
        let r = AreaRenderer::new([1.0; 4], 1.5);
        let mut cmds = Vec::new();
        let pts = vec![
            DataPoint { x: 0.0, y: 10.0 },
            DataPoint { x: 50.0, y: 20.0 },
        ];
        r.render(&pts, &mut cmds);
        if let DrawCommand::DrawPath { closed, fill, points, .. } = &cmds[0] {
            assert!(*closed);
            assert!(fill.is_some());
            // 2 data points + 2 baseline points = 4
            assert_eq!(points.len(), 4);
        } else {
            panic!("expected DrawPath for fill polygon");
        }
    }
}
