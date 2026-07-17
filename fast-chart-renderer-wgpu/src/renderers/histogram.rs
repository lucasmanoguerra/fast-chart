use fast_chart::render::commands::DrawCommand;
use fast_chart::render::series_renderer::{Rect, SeriesHit, SeriesRenderer};

/// A single histogram bar in screen coordinates.
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    pub x: f32,
    pub y: f32,
    pub value: f32,
}

/// Histogram renderer — filled rectangles from zero line to value.
pub struct HistogramRenderer {
    pub positive_color: [f32; 4],
    pub negative_color: [f32; 4],
    pub bar_width: f32,
    pub zero_y: f32,
    bounds: Rect,
}

impl HistogramRenderer {
    pub fn new(color: [f32; 4], width: f32) -> Self {
        Self {
            positive_color: color,
            negative_color: [0.8, 0.0, 0.0, 1.0],
            bar_width: width,
            zero_y: 0.0,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Generate draw commands from screen-space histogram data points.
    pub fn render(&self, points: &[DataPoint], commands: &mut Vec<DrawCommand>) {
        for p in points {
            let color = if p.value >= 0.0 {
                self.positive_color
            } else {
                self.negative_color
            };

            let top = p.y.min(self.zero_y);
            let height = (p.y - self.zero_y).abs().max(1.0);

            commands.push(DrawCommand::filled_rect(
                p.x - self.bar_width / 2.0,
                top,
                self.bar_width,
                height,
                color,
                580,
            ));
        }
    }
}

impl Default for HistogramRenderer {
    fn default() -> Self {
        Self::new([0.0, 0.8, 0.0, 1.0], 8.0)
    }
}

impl SeriesRenderer for HistogramRenderer {
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
        580
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_renderer() {
        let r = HistogramRenderer::new([0.0, 1.0, 0.0, 1.0], 10.0);
        assert_eq!(r.positive_color, [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(r.bar_width, 10.0);
    }

    #[test]
    fn render_produces_commands() {
        let r = HistogramRenderer::new([0.0; 4], 8.0);
        let mut cmds = Vec::new();
        let pts = vec![
            DataPoint { x: 10.0, y: 50.0, value: 1.0 },
            DataPoint { x: 30.0, y: 150.0, value: -1.0 },
        ];
        r.render(&pts, &mut cmds);
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn render_empty_data() {
        let r = HistogramRenderer::new([0.0; 4], 8.0);
        let mut cmds = Vec::new();
        r.render(&[], &mut cmds);
        assert!(cmds.is_empty());
    }

    #[test]
    fn histogram_positive_above_zero() {
        let mut r = HistogramRenderer::new([0.0, 1.0, 0.0, 1.0], 8.0);
        r.zero_y = 100.0;
        let mut cmds = Vec::new();
        let pt = DataPoint { x: 10.0, y: 50.0, value: 1.0 };
        r.render(&[pt], &mut cmds);
        if let DrawCommand::DrawRect { fill, y, .. } = &cmds[0] {
            assert_eq!(*fill, Some([0.0, 1.0, 0.0, 1.0]));
            // y should be 50 (above zero at 100)
            assert_eq!(*y, 50.0);
        } else {
            panic!("expected DrawRect");
        }
    }

    #[test]
    fn histogram_negative_below_zero() {
        let mut r = HistogramRenderer::new([0.0, 1.0, 0.0, 1.0], 8.0);
        r.zero_y = 100.0;
        let mut cmds = Vec::new();
        let pt = DataPoint { x: 10.0, y: 150.0, value: -1.0 };
        r.render(&[pt], &mut cmds);
        if let DrawCommand::DrawRect { fill, .. } = &cmds[0] {
            assert_eq!(*fill, Some([0.8, 0.0, 0.0, 1.0]));
        } else {
            panic!("expected DrawRect");
        }
    }
}
