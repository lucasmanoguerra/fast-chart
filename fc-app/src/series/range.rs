// ---------------------------------------------------------------------------
// RangeSeries — price range bars with fixed high-low range
// ---------------------------------------------------------------------------

use crate::render::commands::DrawCommand;
use fc_primitives::Rect;
use crate::render::series_renderer::{SeriesHit, SeriesRenderer};

/// A data point for a range series.
///
/// Represents a bar where the high-low range is fixed (range bar).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RangeBar {
    pub timestamp: u64,
    pub high: f64,
    pub low: f64,
    /// Whether close >= open (bullish).
    pub bullish: bool,
}

impl RangeBar {
    pub fn new(timestamp: u64, high: f64, low: f64, bullish: bool) -> Self {
        Self {
            timestamp,
            high,
            low,
            bullish,
        }
    }
}

/// Range series renderer.
///
/// Renders price range bars where each bar has a fixed high-low range.
/// This filters out noise by requiring price to move through the full
/// range before a new bar is created.
#[derive(Debug, Clone)]
pub struct RangeSeries {
    /// Data points in chronological order.
    data: Vec<RangeBar>,
    /// Fixed range size (high - low for each bar).
    pub range_size: f64,
    /// Color for bullish bars [r, g, b, a].
    pub bullish_color: [f32; 4],
    /// Color for bearish bars [r, g, b, a].
    pub bearish_color: [f32; 4],
    /// Bar width as fraction of available space (0.0 – 1.0).
    pub bar_ratio: f32,
    /// Bounding rect (set after update).
    bounds: Rect,
}

impl RangeSeries {
    pub fn new(range_size: f64) -> Self {
        Self {
            data: Vec::new(),
            range_size,
            bullish_color: [0.2, 0.8, 0.2, 0.9],
            bearish_color: [0.8, 0.2, 0.2, 0.9],
            bar_ratio: 0.7,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Set the data points for this series.
    pub fn set_data(&mut self, data: Vec<RangeBar>) {
        self.data = data;
    }

    /// Get the data points.
    pub fn data(&self) -> &[RangeBar] {
        &self.data
    }

    /// Get the number of data points.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true when there are no data points.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Build range bars from OHLC bars.
    ///
    /// Each time the price moves through the full range_size, a new range bar is created.
    pub fn build_from_bars(bars: &[(f64, f64, f64)], range_size: f64) -> Vec<RangeBar> {
        if bars.is_empty() || range_size <= 0.0 {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut base = bars[0].2; // start at first close

        for &(high, low, close) in bars {
            // Accumulate range
            if high - base >= range_size {
                let bar_high = base + range_size;
                let bar_low = base;
                result.push(RangeBar::new(
                    0, // timestamp will be assigned by caller if needed
                    bar_high,
                    bar_low,
                    close >= base,
                ));
                base = bar_high;
            } else if base - low >= range_size {
                let bar_high = base;
                let bar_low = base - range_size;
                result.push(RangeBar::new(
                    0,
                    bar_high,
                    bar_low,
                    close >= base,
                ));
                base = bar_low;
            }
        }

        result
    }

    /// Compute min and max values.
    fn value_range(&self) -> Option<(f64, f64)> {
        if self.data.is_empty() {
            return None;
        }
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for b in &self.data {
            if b.low < min { min = b.low; }
            if b.high > max { max = b.high; }
        }
        Some((min, max))
    }

    /// Generate draw commands.
    fn generate_commands(&self, bounds: Rect) -> Vec<DrawCommand> {
        if self.data.is_empty() {
            return Vec::new();
        }

        let (min_val, max_val) = self.value_range().unwrap_or((0.0, 1.0));
        let val_range = (max_val - min_val).max(1.0);
        let slot_width = bounds.width / self.data.len() as f32;
        let bar_width = slot_width * self.bar_ratio;
        let half_gap = (slot_width - bar_width) / 2.0;

        let mut commands = Vec::with_capacity(self.data.len());

        for (i, bar) in self.data.iter().enumerate() {
            let color = if bar.bullish {
                self.bullish_color
            } else {
                self.bearish_color
            };

            let x = bounds.x + i as f32 * slot_width + half_gap;
            let y_high = bounds.y + ((max_val - bar.high) / val_range * bounds.height as f64) as f32;
            let y_low = bounds.y + ((max_val - bar.low) / val_range * bounds.height as f64) as f32;
            let height = (y_low - y_high).max(1.0);

            commands.push(DrawCommand::DrawRect {
                x,
                y: y_high,
                width: bar_width,
                height,
                fill: Some(color),
                stroke: None,
                stroke_width: 0.0,
                z_index: 600,
            });
        }

        commands
    }
}

impl Default for RangeSeries {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl SeriesRenderer for RangeSeries {
    fn update(&mut self, _data: &[DrawCommand], bounds: Rect) -> Vec<DrawCommand> {
        self.bounds = bounds;
        self.generate_commands(bounds)
    }

    fn hit_test(&self, x: f32, _y: f32) -> Option<SeriesHit> {
        if self.data.is_empty() {
            return None;
        }

        let slot_width = self.bounds.width / self.data.len() as f32;
        let index = ((x - self.bounds.x) / slot_width) as usize;
        let index = index.min(self.data.len() - 1);

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
    fn range_bar_new() {
        let b = RangeBar::new(1000, 105.0, 100.0, true);
        assert_eq!(b.timestamp, 1000);
        assert!((b.high - 105.0).abs() < f64::EPSILON);
        assert!((b.low - 100.0).abs() < f64::EPSILON);
        assert!(b.bullish);
    }

    #[test]
    fn range_series_new() {
        let s = RangeSeries::new(5.0);
        assert!(s.is_empty());
        assert!((s.range_size - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn range_series_default() {
        let s = RangeSeries::default();
        assert!((s.range_size - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn range_series_set_data() {
        let mut s = RangeSeries::new(5.0);
        let data = vec![
            RangeBar::new(1000, 105.0, 100.0, true),
            RangeBar::new(2000, 100.0, 95.0, false),
        ];
        s.set_data(data.clone());
        assert_eq!(s.len(), 2);
        assert_eq!(s.data(), &data);
    }

    #[test]
    fn range_series_empty_no_commands() {
        let mut s = RangeSeries::new(5.0);
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
        assert!(cmds.is_empty());
    }

    #[test]
    fn range_series_generates_rect_commands() {
        let mut s = RangeSeries::new(5.0);
        s.set_data(vec![
            RangeBar::new(1000, 105.0, 100.0, true),
            RangeBar::new(2000, 100.0, 95.0, false),
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
    fn range_series_bullish_bearish_colors() {
        let mut s = RangeSeries::new(5.0);
        s.set_data(vec![
            RangeBar::new(1000, 105.0, 100.0, true),
            RangeBar::new(2000, 100.0, 95.0, false),
        ]);
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));

        if let DrawCommand::DrawRect { fill: Some(color), .. } = &cmds[0] {
            assert_eq!(*color, s.bullish_color);
        } else {
            panic!("expected fill");
        }
        if let DrawCommand::DrawRect { fill: Some(color), .. } = &cmds[1] {
            assert_eq!(*color, s.bearish_color);
        } else {
            panic!("expected fill");
        }
    }

    #[test]
    fn range_build_empty() {
        let bars = RangeSeries::build_from_bars(&[], 5.0);
        assert!(bars.is_empty());
    }

    #[test]
    fn range_build_zero_range() {
        let bars = RangeSeries::build_from_bars(&[(100.0, 99.0, 100.5)], 0.0);
        assert!(bars.is_empty());
    }

    #[test]
    fn range_series_hit_test() {
        let mut s = RangeSeries::new(5.0);
        s.set_data(vec![
            RangeBar::new(1000, 105.0, 100.0, true),
            RangeBar::new(2000, 100.0, 95.0, false),
        ]);
        s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));

        let hit = s.hit_test(200.0, 200.0);
        assert!(hit.is_some());
        assert!(hit.unwrap().index < 2);
    }

    #[test]
    fn range_hit_test_empty() {
        let s = RangeSeries::new(5.0);
        assert!(s.hit_test(0.0, 0.0).is_none());
    }
}
