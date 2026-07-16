// ---------------------------------------------------------------------------
// PointFigureSeries — Point & Figure (X/O) chart
// ---------------------------------------------------------------------------

use crate::render::commands::DrawCommand;
use crate::render::series_renderer::{Rect, SeriesHit, SeriesRenderer};

/// A column in a Point & Figure chart.
///
/// Each column is either a rise (X's) or fall (O's) of a fixed box size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PfColumn {
    /// Rising column (X's): price increased.
    Rise { boxes: u32 },
    /// Falling column (O's): price decreased.
    Fall { boxes: u32 },
}

impl PfColumn {
    pub fn is_rise(&self) -> bool {
        matches!(self, PfColumn::Rise { .. })
    }

    pub fn is_fall(&self) -> bool {
        matches!(self, PfColumn::Fall { .. })
    }

    pub fn boxes(&self) -> u32 {
        match self {
            PfColumn::Rise { boxes } | PfColumn::Fall { boxes } => *boxes,
        }
    }
}

/// Point & Figure series renderer.
///
/// Renders an X/O chart where columns represent price movements of a
/// configurable box size. A reversal of `reversal_size` boxes switches
/// from X to O or vice versa. No time axis — columns are plotted left
/// to right in chronological order, but spacing is uniform.
#[derive(Debug, Clone)]
pub struct PointFigureSeries {
    /// Columns in chronological order.
    columns: Vec<PfColumn>,
    /// Box size (price increment per box).
    pub box_size: f64,
    /// Number of boxes needed to reverse direction.
    pub reversal_size: u32,
    /// Color for X (rise) columns [r, g, b, a].
    pub rise_color: [f32; 4],
    /// Color for O (fall) columns [r, g, b, a].
    pub fall_color: [f32; 4],
    /// Bounding rect (set after update).
    bounds: Rect,
}

impl PointFigureSeries {
    pub fn new(box_size: f64, reversal_size: u32) -> Self {
        Self {
            columns: Vec::new(),
            box_size,
            reversal_size,
            rise_color: [0.2, 0.8, 0.2, 1.0],
            fall_color: [0.8, 0.2, 0.2, 1.0],
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Set the columns for this series.
    pub fn set_columns(&mut self, columns: Vec<PfColumn>) {
        self.columns = columns;
    }

    /// Get the columns.
    pub fn columns(&self) -> &[PfColumn] {
        &self.columns
    }

    /// Get the number of columns.
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    /// Returns true when there are no columns.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Maximum box count across all columns.
    pub fn max_boxes(&self) -> u32 {
        self.columns.iter().map(|c| c.boxes()).max().unwrap_or(0)
    }

    /// Build Point & Figure columns from OHLC bars.
    ///
    /// This is a simplified builder that tracks direction and boxes.
    pub fn build_from_prices(prices: &[(f64, f64)], box_size: f64, reversal: u32) -> Vec<PfColumn> {
        if prices.is_empty() {
            return Vec::new();
        }

        let mut columns: Vec<PfColumn> = Vec::new();
        let mut current_high = prices[0].0;
        let mut current_low = prices[0].1;
        let mut in_rise = true;
        let mut boxes = 1u32;

        for &(high, low) in &prices[1..] {
            if in_rise {
                if high >= current_high {
                    let new_boxes = ((high - current_low) / box_size).floor() as u32;
                    if new_boxes > boxes {
                        boxes = new_boxes;
                    }
                    current_high = high;
                } else if low <= current_high - box_size * reversal as f64 {
                    // Reversal
                    if boxes > 0 {
                        columns.push(PfColumn::Rise { boxes });
                    }
                    in_rise = false;
                    current_low = low;
                    boxes = ((current_high - low) / box_size).floor().max(1.0) as u32;
                }
            } else {
                if low <= current_low {
                    let new_boxes = ((current_high - low) / box_size).floor() as u32;
                    if new_boxes > boxes {
                        boxes = new_boxes;
                    }
                    current_low = low;
                } else if high >= current_low + box_size * reversal as f64 {
                    // Reversal
                    if boxes > 0 {
                        columns.push(PfColumn::Fall { boxes });
                    }
                    in_rise = true;
                    current_high = high;
                    boxes = ((high - current_low) / box_size).floor().max(1.0) as u32;
                }
            }
        }

        // Push the last column
        if boxes > 0 {
            if in_rise {
                columns.push(PfColumn::Rise { boxes });
            } else {
                columns.push(PfColumn::Fall { boxes });
            }
        }

        columns
    }

    /// Generate draw commands from columns.
    fn generate_commands(&self, bounds: Rect) -> Vec<DrawCommand> {
        if self.columns.is_empty() {
            return Vec::new();
        }

        let max = self.max_boxes().max(1);
        let slot_width = bounds.width / self.columns.len() as f32;
        let box_height = bounds.height / max as f32;

        let mut commands = Vec::with_capacity(self.columns.len());

        for (i, col) in self.columns.iter().enumerate() {
            let color = if col.is_rise() {
                self.rise_color
            } else {
                self.fall_color
            };

            let x = bounds.x + i as f32 * slot_width;
            let n = col.boxes();

            // Draw each box in the column
            for b in 0..n {
                let y = bounds.y + bounds.height - (b + 1) as f32 * box_height;
                commands.push(DrawCommand::DrawRect {
                    x,
                    y,
                    width: slot_width,
                    height: box_height,
                    fill: Some(color),
                    stroke: None,
                    stroke_width: 0.0,
                    z_index: 600,
                });
            }
        }

        commands
    }
}

impl Default for PointFigureSeries {
    fn default() -> Self {
        Self::new(1.0, 3)
    }
}

impl SeriesRenderer for PointFigureSeries {
    fn update(&mut self, _data: &[DrawCommand], bounds: Rect) -> Vec<DrawCommand> {
        self.bounds = bounds;
        self.generate_commands(bounds)
    }

    fn hit_test(&self, x: f32, _y: f32) -> Option<SeriesHit> {
        if self.columns.is_empty() {
            return None;
        }

        let slot_width = self.bounds.width / self.columns.len() as f32;
        let index = ((x - self.bounds.x) / slot_width) as usize;
        let index = index.min(self.columns.len() - 1);

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
    fn pf_column_rise() {
        let c = PfColumn::Rise { boxes: 5 };
        assert!(c.is_rise());
        assert!(!c.is_fall());
        assert_eq!(c.boxes(), 5);
    }

    #[test]
    fn pf_column_fall() {
        let c = PfColumn::Fall { boxes: 3 };
        assert!(!c.is_rise());
        assert!(c.is_fall());
        assert_eq!(c.boxes(), 3);
    }

    #[test]
    fn pf_series_new() {
        let s = PointFigureSeries::new(2.5, 3);
        assert!(s.is_empty());
        assert!((s.box_size - 2.5).abs() < f64::EPSILON);
        assert_eq!(s.reversal_size, 3);
    }

    #[test]
    fn pf_series_default() {
        let s = PointFigureSeries::default();
        assert!((s.box_size - 1.0).abs() < f64::EPSILON);
        assert_eq!(s.reversal_size, 3);
    }

    #[test]
    fn pf_series_set_columns() {
        let mut s = PointFigureSeries::new(1.0, 3);
        let cols = vec![PfColumn::Rise { boxes: 5 }, PfColumn::Fall { boxes: 3 }];
        s.set_columns(cols.clone());
        assert_eq!(s.len(), 2);
        assert_eq!(s.columns(), &cols);
    }

    #[test]
    fn pf_series_max_boxes() {
        let mut s = PointFigureSeries::new(1.0, 3);
        s.set_columns(vec![
            PfColumn::Rise { boxes: 5 },
            PfColumn::Fall { boxes: 8 },
            PfColumn::Rise { boxes: 3 },
        ]);
        assert_eq!(s.max_boxes(), 8);
    }

    #[test]
    fn pf_series_empty_no_commands() {
        let mut s = PointFigureSeries::new(1.0, 3);
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
        assert!(cmds.is_empty());
    }

    #[test]
    fn pf_series_generates_rect_commands() {
        let mut s = PointFigureSeries::new(1.0, 3);
        s.set_columns(vec![PfColumn::Rise { boxes: 5 }]);
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 100.0, 200.0));
        assert_eq!(cmds.len(), 5); // 5 boxes

        for cmd in &cmds {
            match cmd {
                DrawCommand::DrawRect { .. } => {}
                _ => panic!("expected DrawRect commands"),
            }
        }
    }

    #[test]
    fn pf_series_multiple_columns() {
        let mut s = PointFigureSeries::new(1.0, 3);
        s.set_columns(vec![
            PfColumn::Rise { boxes: 3 },
            PfColumn::Fall { boxes: 2 },
        ]);
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 200.0, 200.0));
        assert_eq!(cmds.len(), 5); // 3 + 2 boxes
    }

    #[test]
    fn pf_series_hit_test() {
        let mut s = PointFigureSeries::new(1.0, 3);
        s.set_columns(vec![
            PfColumn::Rise { boxes: 5 },
            PfColumn::Fall { boxes: 3 },
        ]);
        let bounds = Rect::new(0.0, 0.0, 200.0, 200.0);
        s.update(&[], bounds);

        let hit = s.hit_test(50.0, 100.0);
        assert!(hit.is_some());
        assert!(hit.unwrap().index < 2);
    }

    #[test]
    fn pf_series_hit_test_empty() {
        let s = PointFigureSeries::new(1.0, 3);
        assert!(s.hit_test(0.0, 0.0).is_none());
    }

    #[test]
    fn pf_build_from_prices_empty() {
        let cols = PointFigureSeries::build_from_prices(&[], 1.0, 3);
        assert!(cols.is_empty());
    }

    #[test]
    fn pf_build_from_prices_monotonic_rise() {
        let prices = vec![
            (100.0, 99.0),
            (105.0, 101.0),
            (110.0, 106.0),
        ];
        let cols = PointFigureSeries::build_from_prices(&prices, 1.0, 3);
        // Should produce at least one Rise column
        assert!(!cols.is_empty());
        assert!(cols[0].is_rise());
    }

    #[test]
    fn pf_series_bands_set_after_update() {
        let mut s = PointFigureSeries::new(1.0, 3);
        s.set_columns(vec![PfColumn::Rise { boxes: 5 }]);
        let bounds = Rect::new(10.0, 20.0, 780.0, 380.0);
        s.update(&[], bounds);
        assert_eq!(s.bounds(), bounds);
    }
}
