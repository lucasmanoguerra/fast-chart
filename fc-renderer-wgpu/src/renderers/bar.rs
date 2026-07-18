use fast_chart::render::commands::DrawCommand;
use fast_chart::render::series_renderer::{Rect, SeriesHit, SeriesRenderer};

/// OHLC bar data point in screen coordinates.
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    pub x: f32,
    pub y: f32,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
}

/// OHLC bar renderer — vertical line from low to high,
/// horizontal tick left for open, right for close.
pub struct BarRenderer {
    pub color: [f32; 4],
    pub tick_width: f32,
    pub wick_width: f32,
    bounds: Rect,
}

impl BarRenderer {
    pub fn new(color: [f32; 4], width: f32) -> Self {
        Self {
            color,
            tick_width: width,
            wick_width: 1.0,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Generate draw commands from screen-space OHLC data points.
    pub fn render(&self, points: &[DataPoint], commands: &mut Vec<DrawCommand>) {
        for p in points {
            let color = if p.close >= p.open {
                [0.0, 0.8, 0.0, 1.0]
            } else {
                [0.8, 0.0, 0.0, 1.0]
            };

            // Vertical line from high to low
            commands.push(DrawCommand::line(
                p.x,
                p.high,
                p.x,
                p.low,
                color,
                self.wick_width,
                600,
            ));

            // Open tick: horizontal line to the left
            commands.push(DrawCommand::line(
                p.x - self.tick_width,
                p.open,
                p.x,
                p.open,
                color,
                self.wick_width,
                600,
            ));

            // Close tick: horizontal line to the right
            commands.push(DrawCommand::line(
                p.x,
                p.close,
                p.x + self.tick_width,
                p.close,
                color,
                self.wick_width,
                600,
            ));
        }
    }
}

impl Default for BarRenderer {
    fn default() -> Self {
        Self::new([0.2, 0.6, 1.0, 1.0], 6.0)
    }
}

impl SeriesRenderer for BarRenderer {
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
        600
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bullish_point() -> DataPoint {
        DataPoint {
            x: 50.0,
            y: 100.0,
            open: 110.0,
            high: 120.0,
            low: 90.0,
            close: 100.0,
        }
    }

    #[test]
    fn new_renderer() {
        let r = BarRenderer::new([1.0; 4], 6.0);
        assert_eq!(r.color, [1.0; 4]);
        assert_eq!(r.tick_width, 6.0);
    }

    #[test]
    fn render_produces_commands() {
        let r = BarRenderer::new([1.0; 4], 6.0);
        let mut cmds = Vec::new();
        r.render(&[bullish_point()], &mut cmds);
        // 1 bar = 3 commands: vertical + open tick + close tick
        assert_eq!(cmds.len(), 3);
    }

    #[test]
    fn render_empty_data() {
        let r = BarRenderer::new([1.0; 4], 6.0);
        let mut cmds = Vec::new();
        r.render(&[], &mut cmds);
        assert!(cmds.is_empty());
    }
}
