// ---------------------------------------------------------------------------
// LineBreakSeries — N-line break chart
// ---------------------------------------------------------------------------

use crate::render::commands::DrawCommand;
use fc_primitives::Rect;
use crate::render::series_renderer::{SeriesHit, SeriesRenderer};

/// A block in a line break chart.
///
/// Each block represents a price movement of at least `line_count` times
/// the previous close's range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineBreakBlock {
    /// Whether this is an up block (green) or down block (red).
    pub is_up: bool,
    /// The opening price of this block.
    pub open: f64,
    /// The closing price of this block.
    pub close: f64,
}

impl LineBreakBlock {
    pub fn new(is_up: bool, open: f64, close: f64) -> Self {
        Self { is_up, open, close }
    }
}

/// Line break series renderer.
///
/// Similar to Renko, but uses N-line break logic: a new block is drawn
/// when price moves by `line_count` times the previous block's range.
/// This filters out minor price fluctuations.
#[derive(Debug, Clone)]
pub struct LineBreakSeries {
    /// Blocks in chronological order.
    blocks: Vec<LineBreakBlock>,
    /// Number of lines required for a break.
    pub line_count: u32,
    /// Color for up blocks [r, g, b, a].
    pub up_color: [f32; 4],
    /// Color for down blocks [r, g, b, a].
    pub down_color: [f32; 4],
    /// Bounding rect (set after update).
    bounds: Rect,
}

impl LineBreakSeries {
    pub fn new(line_count: u32) -> Self {
        Self {
            blocks: Vec::new(),
            line_count,
            up_color: [0.2, 0.8, 0.2, 1.0],
            down_color: [0.8, 0.2, 0.2, 1.0],
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Set the blocks for this series.
    pub fn set_blocks(&mut self, blocks: Vec<LineBreakBlock>) {
        self.blocks = blocks;
    }

    /// Get the blocks.
    pub fn blocks(&self) -> &[LineBreakBlock] {
        &self.blocks
    }

    /// Get the number of blocks.
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Returns true when there are no blocks.
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Build line break blocks from OHLC bars.
    pub fn build_from_bars(bars: &[(f64, f64, f64)], line_count: u32) -> Vec<LineBreakBlock> {
        if bars.len() < 2 {
            return Vec::new();
        }

        let mut blocks = Vec::new();
        let mut prev_close = bars[0].2; // close of first bar
        let mut current_open = bars[0].2;
        let mut current_is_up = true;

        for &(high, low, close) in &bars[1..] {
            let range = prev_close;
            let threshold = range * line_count as f64;

            if close >= current_open + threshold {
                // Up break
                if !current_is_up && !blocks.is_empty() {
                    blocks.push(LineBreakBlock::new(current_is_up, current_open, prev_close));
                }
                current_is_up = true;
                current_open = prev_close;
                prev_close = close;
            } else if close <= current_open - threshold {
                // Down break
                if current_is_up && !blocks.is_empty() {
                    blocks.push(LineBreakBlock::new(current_is_up, current_open, prev_close));
                }
                current_is_up = false;
                current_open = prev_close;
                prev_close = close;
            } else {
                prev_close = close;
            }
        }

        // Push the last block
        blocks.push(LineBreakBlock::new(current_is_up, current_open, prev_close));
        blocks
    }

    /// Generate draw commands from blocks.
    fn generate_commands(&self, bounds: Rect) -> Vec<DrawCommand> {
        if self.blocks.is_empty() {
            return Vec::new();
        }

        let slot_width = bounds.width / self.blocks.len() as f32;
        let (min_val, max_val) = self.value_range().unwrap_or((0.0, 1.0));
        let val_range = (max_val - min_val).max(1.0);

        let mut commands = Vec::with_capacity(self.blocks.len());

        for (i, block) in self.blocks.iter().enumerate() {
            let color = if block.is_up {
                self.up_color
            } else {
                self.down_color
            };

            let x = bounds.x + i as f32 * slot_width;
            let y_open = bounds.y + ((max_val - block.open) / val_range * bounds.height as f64) as f32;
            let y_close = bounds.y + ((max_val - block.close) / val_range * bounds.height as f64) as f32;
            let y_top = y_open.min(y_close);
            let height = (y_open - y_close).abs();

            commands.push(DrawCommand::DrawRect {
                x,
                y: y_top,
                width: slot_width,
                height: height.max(1.0),
                fill: Some(color),
                stroke: None,
                stroke_width: 0.0,
                z_index: 600,
            });
        }

        commands
    }

    /// Compute min and max values across all blocks.
    fn value_range(&self) -> Option<(f64, f64)> {
        if self.blocks.is_empty() {
            return None;
        }
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for b in &self.blocks {
            let lo = b.open.min(b.close);
            let hi = b.open.max(b.close);
            if lo < min { min = lo; }
            if hi > max { max = hi; }
        }
        Some((min, max))
    }
}

impl Default for LineBreakSeries {
    fn default() -> Self {
        Self::new(3)
    }
}

impl SeriesRenderer for LineBreakSeries {
    fn update(&mut self, _data: &[DrawCommand], bounds: Rect) -> Vec<DrawCommand> {
        self.bounds = bounds;
        self.generate_commands(bounds)
    }

    fn hit_test(&self, x: f32, _y: f32) -> Option<SeriesHit> {
        if self.blocks.is_empty() {
            return None;
        }

        let slot_width = self.bounds.width / self.blocks.len() as f32;
        let index = ((x - self.bounds.x) / slot_width) as usize;
        let index = index.min(self.blocks.len() - 1);

        let center_x = self.bounds.x + index as f32 * slot_width + slot_width / 2.0;
        let distance = (x - center_x).abs();

        Some(SeriesHit { index, distance })
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_break_block_new() {
        let b = LineBreakBlock::new(true, 100.0, 110.0);
        assert!(b.is_up);
        assert!((b.open - 100.0).abs() < f64::EPSILON);
        assert!((b.close - 110.0).abs() < f64::EPSILON);
    }

    #[test]
    fn line_break_series_new() {
        let s = LineBreakSeries::new(3);
        assert!(s.is_empty());
        assert_eq!(s.line_count, 3);
    }

    #[test]
    fn line_break_series_default() {
        let s = LineBreakSeries::default();
        assert_eq!(s.line_count, 3);
    }

    #[test]
    fn line_break_series_set_blocks() {
        let mut s = LineBreakSeries::new(3);
        let blocks = vec![
            LineBreakBlock::new(true, 100.0, 110.0),
            LineBreakBlock::new(false, 110.0, 105.0),
        ];
        s.set_blocks(blocks.clone());
        assert_eq!(s.len(), 2);
        assert_eq!(s.blocks(), &blocks);
    }

    #[test]
    fn line_break_series_empty_no_commands() {
        let mut s = LineBreakSeries::new(3);
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
        assert!(cmds.is_empty());
    }

    #[test]
    fn line_break_series_generates_rect_commands() {
        let mut s = LineBreakSeries::new(3);
        s.set_blocks(vec![
            LineBreakBlock::new(true, 100.0, 110.0),
            LineBreakBlock::new(false, 110.0, 105.0),
        ]);
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
        assert_eq!(cmds.len(), 2);

        for cmd in &cmds {
            match cmd {
                DrawCommand::DrawRect { .. } => {}
                _ => panic!("expected DrawRect"),
            }
        }
    }

    #[test]
    fn line_break_series_up_down_colors() {
        let mut s = LineBreakSeries::new(3);
        s.set_blocks(vec![
            LineBreakBlock::new(true, 100.0, 110.0),
            LineBreakBlock::new(false, 110.0, 105.0),
        ]);
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));

        if let DrawCommand::DrawRect { fill: Some(color), .. } = &cmds[0] {
            assert_eq!(*color, s.up_color);
        } else {
            panic!("expected fill color");
        }

        if let DrawCommand::DrawRect { fill: Some(color), .. } = &cmds[1] {
            assert_eq!(*color, s.down_color);
        } else {
            panic!("expected fill color");
        }
    }

    #[test]
    fn line_break_build_empty() {
        let blocks = LineBreakSeries::build_from_bars(&[], 3);
        assert!(blocks.is_empty());
    }

    #[test]
    fn line_break_build_single_bar() {
        let blocks = LineBreakSeries::build_from_bars(&[(100.0, 99.0, 100.5)], 3);
        assert!(blocks.is_empty());
    }

    #[test]
    fn line_break_series_hit_test() {
        let mut s = LineBreakSeries::new(3);
        s.set_blocks(vec![
            LineBreakBlock::new(true, 100.0, 110.0),
            LineBreakBlock::new(false, 110.0, 105.0),
        ]);
        s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));

        let hit = s.hit_test(200.0, 200.0);
        assert!(hit.is_some());
        assert!(hit.unwrap().index < 2);
    }

    #[test]
    fn line_break_hit_test_empty() {
        let s = LineBreakSeries::new(3);
        assert!(s.hit_test(0.0, 0.0).is_none());
    }
}
