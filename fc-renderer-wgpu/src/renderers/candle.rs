use fc_render::commands::DrawCommand;
use fc_primitives::Rect;
use fc_render::series_renderer::{SeriesHit, SeriesRenderer};

/// A single OHLC data point already mapped to screen coordinates.
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    pub x: f32,
    pub y: f32,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
}

/// Candlestick renderer — produces filled body + wick line for each bar.
pub struct CandleRenderer {
    pub bullish_color: [f32; 4],
    pub bearish_color: [f32; 4],
    pub body_width: f32,
    pub wick_width: f32,
    bounds: Rect,
}

impl CandleRenderer {
    pub fn new(_color: [f32; 4], width: f32) -> Self {
        Self {
            bullish_color: [0.0, 0.8, 0.0, 1.0],
            bearish_color: [0.8, 0.0, 0.0, 1.0],
            body_width: width,
            wick_width: 1.0,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Generate draw commands from screen-space data points.
    pub fn render(&self, points: &[DataPoint], commands: &mut Vec<DrawCommand>) {
        for p in points {
            let color = if p.close >= p.open {
                self.bullish_color
            } else {
                self.bearish_color
            };

            // Wick: vertical line from high to low
            commands.push(DrawCommand::line(
                p.x,
                p.high,
                p.x,
                p.low,
                color,
                self.wick_width,
                600,
            ));

            // Body: filled rect from open to close
            let top = p.open.min(p.close);
            let height = (p.open - p.close).abs().max(1.0);
            commands.push(DrawCommand::filled_rect(
                p.x - self.body_width / 2.0,
                top,
                self.body_width,
                height,
                color,
                600,
            ));
        }
    }
}

impl Default for CandleRenderer {
    fn default() -> Self {
        Self::new([1.0; 4], 8.0)
    }
}

impl SeriesRenderer for CandleRenderer {
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

    fn sample_points() -> Vec<DataPoint> {
        vec![
            DataPoint {
                x: 10.0,
                y: 100.0,
                open: 110.0,
                high: 120.0,
                low: 90.0,
                close: 100.0,
            },
            DataPoint {
                x: 30.0,
                y: 90.0,
                open: 80.0,
                high: 95.0,
                low: 75.0,
                close: 90.0,
            },
        ]
    }

    #[test]
    fn new_renderer() {
        let r = CandleRenderer::new([1.0; 4], 8.0);
        assert_eq!(r.bullish_color, [0.0, 0.8, 0.0, 1.0]);
        assert_eq!(r.bearish_color, [0.8, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn render_produces_commands() {
        let r = CandleRenderer::new([1.0; 4], 8.0);
        let mut cmds = Vec::new();
        r.render(&sample_points(), &mut cmds);
        assert!(!cmds.is_empty());
    }

    #[test]
    fn render_empty_data() {
        let r = CandleRenderer::new([1.0; 4], 8.0);
        let mut cmds = Vec::new();
        r.render(&[], &mut cmds);
        assert!(cmds.is_empty());
    }

    #[test]
    fn candle_bullish_color() {
        let r = CandleRenderer::new([1.0; 4], 8.0);
        let mut cmds = Vec::new();
        // close (90) > open (80) → bullish
        let pt = DataPoint {
            x: 10.0,
            y: 90.0,
            open: 80.0,
            high: 95.0,
            low: 75.0,
            close: 90.0,
        };
        r.render(&[pt], &mut cmds);
        // First command is the wick line
        if let DrawCommand::DrawLine { color, .. } = &cmds[0] {
            assert_eq!(*color, [0.0, 0.8, 0.0, 1.0]);
        } else {
            panic!("expected DrawLine for wick");
        }
    }

    #[test]
    fn candle_bearish_color() {
        let r = CandleRenderer::new([1.0; 4], 8.0);
        let mut cmds = Vec::new();
        // close (70) < open (80) → bearish
        let pt = DataPoint {
            x: 10.0,
            y: 70.0,
            open: 80.0,
            high: 85.0,
            low: 65.0,
            close: 70.0,
        };
        r.render(&[pt], &mut cmds);
        if let DrawCommand::DrawLine { color, .. } = &cmds[0] {
            assert_eq!(*color, [0.8, 0.0, 0.0, 1.0]);
        } else {
            panic!("expected DrawLine for wick");
        }
    }

    #[test]
    fn two_candles_produce_four_commands() {
        let r = CandleRenderer::new([1.0; 4], 8.0);
        let mut cmds = Vec::new();
        r.render(&sample_points(), &mut cmds);
        // 2 candles × (wick + body) = 4 commands
        assert_eq!(cmds.len(), 4);
    }
}
