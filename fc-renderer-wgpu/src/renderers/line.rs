use fc_render::commands::DrawCommand;
use fc_primitives::Rect;
use fc_render::series_renderer::{SeriesHit, SeriesRenderer};

/// A single data point for line rendering (screen coordinates).
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    pub x: f32,
    pub y: f32,
}

/// Line renderer — produces a polyline between consecutive data points.
pub struct LineRenderer {
    pub color: [f32; 4],
    pub width: f32,
    bounds: Rect,
}

impl LineRenderer {
    pub fn new(color: [f32; 4], width: f32) -> Self {
        Self {
            color,
            width,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Generate draw commands from screen-space data points.
    pub fn render(&self, points: &[DataPoint], commands: &mut Vec<DrawCommand>) {
        if points.len() < 2 {
            return;
        }

        let line_points: Vec<[f32; 2]> = points.iter().map(|p| [p.x, p.y]).collect();

        commands.push(DrawCommand::polyline(
            line_points,
            self.color,
            self.width,
            650,
        ));
    }
}

impl Default for LineRenderer {
    fn default() -> Self {
        Self::new([0.2, 0.6, 1.0, 1.0], 1.5)
    }
}

impl SeriesRenderer for LineRenderer {
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
        650
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Clasificación: determinística — verifica new_renderer
    #[test]
    fn new_renderer() {
        let r = LineRenderer::new([1.0; 4], 2.0);
        assert_eq!(r.color, [1.0; 4]);
        assert_eq!(r.width, 2.0);
    }

    // Clasificación: determinística — verifica render_produces_commands
    #[test]
    fn render_produces_commands() {
        let r = LineRenderer::new([1.0; 4], 1.5);
        let mut cmds = Vec::new();
        let pts = vec![
            DataPoint { x: 0.0, y: 10.0 },
            DataPoint { x: 50.0, y: 20.0 },
            DataPoint { x: 100.0, y: 15.0 },
        ];
        r.render(&pts, &mut cmds);
        assert_eq!(cmds.len(), 1);
    }

    // Clasificación: determinística — verifica render_empty_data
    #[test]
    fn render_empty_data() {
        let r = LineRenderer::new([1.0; 4], 1.5);
        let mut cmds = Vec::new();
        r.render(&[], &mut cmds);
        assert!(cmds.is_empty());
    }

    // Clasificación: determinística — verifica render_single_point_produces_nothing
    #[test]
    fn render_single_point_produces_nothing() {
        let r = LineRenderer::new([1.0; 4], 1.5);
        let mut cmds = Vec::new();
        r.render(&[DataPoint { x: 0.0, y: 10.0 }], &mut cmds);
        assert!(cmds.is_empty());
    }

    // Clasificación: determinística — verifica line_has_correct_points
    #[test]
    fn line_has_correct_points() {
        let r = LineRenderer::new([1.0; 4], 1.5);
        let mut cmds = Vec::new();
        let pts = vec![
            DataPoint { x: 0.0, y: 10.0 },
            DataPoint { x: 50.0, y: 20.0 },
        ];
        r.render(&pts, &mut cmds);
        if let DrawCommand::DrawPath { points, color, width, closed, .. } = &cmds[0] {
            assert_eq!(points.len(), 2);
            assert_eq!(points[0], [0.0, 10.0]);
            assert_eq!(points[1], [50.0, 20.0]);
            assert_eq!(*color, [1.0; 4]);
            assert_eq!(*width, 1.5);
            assert!(!closed);
        } else {
            panic!("expected DrawPath");
        }
    }
}
