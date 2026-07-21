// ---------------------------------------------------------------------------
// StepLineSeries — line chart with horizontal-then-vertical segments
// ---------------------------------------------------------------------------

use crate::render::commands::DrawCommand;
use fc_primitives::Rect;
use crate::render::series_renderer::{SeriesHit, SeriesRenderer};

/// A data point for a step line series.
///
/// Contains a timestamp and a single value (typically close price).
/// The step line holds the value horizontally until the next data point,
/// then moves vertically to the new value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepPoint {
    pub timestamp: u64,
    pub value: f64,
}

impl StepPoint {
    pub fn new(timestamp: u64, value: f64) -> Self {
        Self { timestamp, value }
    }
}

/// Step line series renderer.
///
/// Renders a line that holds its value constant between data points
/// (horizontal segment) then jumps vertically to the next value.
/// This is useful for step functions, OHLC close-only charts, and
/// level indicators.
#[derive(Debug, Clone)]
pub struct StepLineSeries {
    /// Data points in chronological order.
    data: Vec<StepPoint>,
    /// Color as [r, g, b, a] in 0.0..1.0.
    pub color: [f32; 4],
    /// Line width in pixels.
    pub line_width: f32,
    /// Bounding rect (set after update).
    bounds: Rect,
}

impl StepLineSeries {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            color: [0.2, 0.6, 1.0, 1.0],
            line_width: 1.5,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Set the data points for this series.
    pub fn set_data(&mut self, data: Vec<StepPoint>) {
        self.data = data;
    }

    /// Get the data points.
    pub fn data(&self) -> &[StepPoint] {
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

    /// Compute min and max values of the series.
    pub fn value_range(&self) -> Option<(f64, f64)> {
        if self.data.is_empty() {
            return None;
        }
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for p in &self.data {
            if p.value < min {
                min = p.value;
            }
            if p.value > max {
                max = p.value;
            }
        }
        Some((min, max))
    }

    /// Generate step line draw commands from the data.
    ///
    /// Each data point produces a horizontal segment from the previous
    /// timestamp to the current timestamp at the previous value,
    /// followed by a vertical segment from the previous value to the current value.
    fn generate_commands(&self, bounds: Rect) -> Vec<DrawCommand> {
        if self.data.len() < 2 {
            return Vec::new();
        }

        let ts_min = self.data[0].timestamp;
        let ts_max = match self.data.last() {
            Some(p) => p.timestamp,
            None => return Vec::new(),
        };
        let ts_range = (ts_max - ts_min) as f64;

        let (val_min, val_max) = self.value_range().unwrap_or((0.0, 1.0));
        let val_range = val_max - val_min;

        let mut commands = Vec::with_capacity((self.data.len() - 1) * 2);

        for i in 1..self.data.len() {
            let prev = &self.data[i - 1];
            let curr = &self.data[i];

            let x_prev = bounds.x
                + ((prev.timestamp - ts_min) as f64 / ts_range * bounds.width as f64) as f32;
            let x_curr = bounds.x
                + ((curr.timestamp - ts_min) as f64 / ts_range * bounds.width as f64) as f32;

            let y_prev = if val_range.abs() < f64::EPSILON {
                bounds.y + bounds.height / 2.0
            } else {
                bounds.y
                    + ((val_max - prev.value) / val_range * bounds.height as f64) as f32
            };
            let y_curr = if val_range.abs() < f64::EPSILON {
                bounds.y + bounds.height / 2.0
            } else {
                bounds.y
                    + ((val_max - curr.value) / val_range * bounds.height as f64) as f32
            };

            // Horizontal segment: prev.timestamp -> curr.timestamp at prev.value
            commands.push(DrawCommand::DrawLine {
                x0: x_prev,
                y0: y_prev,
                x1: x_curr,
                y1: y_prev,
                color: self.color,
                width: self.line_width,
                style: crate::render::LineStyle::Solid,
                z_index: 600,
            });

            // Vertical segment: prev.value -> curr.value at curr.timestamp
            commands.push(DrawCommand::DrawLine {
                x0: x_curr,
                y0: y_prev,
                x1: x_curr,
                y1: y_curr,
                color: self.color,
                width: self.line_width,
                style: crate::render::LineStyle::Solid,
                z_index: 600,
            });
        }

        commands
    }
}

impl Default for StepLineSeries {
    fn default() -> Self {
        Self::new()
    }
}

impl SeriesRenderer for StepLineSeries {
    fn update(&mut self, _data: &[DrawCommand], bounds: Rect) -> Vec<DrawCommand> {
        self.bounds = bounds;
        self.generate_commands(bounds)
    }

    fn hit_test(&self, x: f32, y: f32) -> Option<SeriesHit> {
        if self.data.is_empty() {
            return None;
        }

        let ts_min = self.data[0].timestamp;
        let ts_max = match self.data.last() {
            Some(p) => p.timestamp,
            None => return None,
        };
        let ts_range = (ts_max - ts_min) as f64;
        if ts_range < 1.0 {
            return None;
        }

        let (val_min, val_max) = self.value_range()?;
        let val_range = val_max - val_min;

        // Find the nearest data point by horizontal distance
        let mut best = None;
        let mut best_dist = f32::MAX;

        for (i, p) in self.data.iter().enumerate() {
            let px = self.bounds.x
                + ((p.timestamp - ts_min) as f64 / ts_range * self.bounds.width as f64) as f32;
            let py = if val_range.abs() < f64::EPSILON {
                self.bounds.y + self.bounds.height / 2.0
            } else {
                self.bounds.y + ((val_max - p.value) / val_range * self.bounds.height as f64) as f32
            };

            let dist = ((px - x).powi(2) + (py - y).powi(2)).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best = Some(SeriesHit {
                    index: i,
                    distance: dist,
                });
            }
        }

        best
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

    fn sample_data() -> Vec<StepPoint> {
        vec![
            StepPoint::new(1000, 100.0),
            StepPoint::new(2000, 120.0),
            StepPoint::new(3000, 110.0),
            StepPoint::new(4000, 130.0),
        ]
    }

    #[test]
    fn step_point_new() {
        let p = StepPoint::new(1000, 50.0);
        assert_eq!(p.timestamp, 1000);
        assert!((p.value - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn step_line_new() {
        let s = StepLineSeries::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
        assert_eq!(s.color, [0.2, 0.6, 1.0, 1.0]);
        assert!((s.line_width - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn step_line_set_data() {
        let mut s = StepLineSeries::new();
        let data = sample_data();
        s.set_data(data.clone());
        assert_eq!(s.len(), 4);
        assert_eq!(s.data(), &data);
    }

    #[test]
    fn step_line_value_range() {
        let mut s = StepLineSeries::new();
        assert_eq!(s.value_range(), None);

        s.set_data(sample_data());
        let (min, max) = s.value_range().unwrap();
        assert!((min - 100.0).abs() < f64::EPSILON);
        assert!((max - 130.0).abs() < f64::EPSILON);
    }

    #[test]
    fn step_line_empty_data_no_commands() {
        let mut s = StepLineSeries::new();
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
        assert!(cmds.is_empty());
    }

    #[test]
    fn step_line_single_point_no_commands() {
        let mut s = StepLineSeries::new();
        s.set_data(vec![StepPoint::new(1000, 100.0)]);
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
        assert!(cmds.is_empty());
    }

    #[test]
    fn step_line_generates_correct_command_count() {
        let mut s = StepLineSeries::new();
        s.set_data(sample_data());
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
        // 4 points → 3 transitions × 2 lines each = 6 commands
        assert_eq!(cmds.len(), 6);
    }

    #[test]
    fn step_line_commands_are_lines() {
        let mut s = StepLineSeries::new();
        s.set_data(sample_data());
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));

        for cmd in &cmds {
            match cmd {
                DrawCommand::DrawLine { .. } => {}
                _ => panic!("expected DrawLine commands"),
            }
        }
    }

    #[test]
    fn step_line_bounds_set_after_update() {
        let mut s = StepLineSeries::new();
        s.set_data(sample_data());
        let bounds = Rect::new(10.0, 20.0, 780.0, 380.0);
        s.update(&[], bounds);
        assert_eq!(s.bounds(), bounds);
    }

    #[test]
    fn step_line_hit_test_returns_nearest() {
        let mut s = StepLineSeries::new();
        s.set_data(sample_data());
        let bounds = Rect::new(0.0, 0.0, 800.0, 400.0);
        s.update(&[], bounds);

        // Hit near first point
        let hit = s.hit_test(0.0, 400.0);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().index, 0);
    }

    #[test]
    fn step_line_hit_test_empty() {
        let s = StepLineSeries::new();
        assert!(s.hit_test(0.0, 0.0).is_none());
    }

    #[test]
    fn step_line_default() {
        let s = StepLineSeries::default();
        assert!(s.is_empty());
    }

    #[test]
    fn step_line_clone() {
        let mut s = StepLineSeries::new();
        s.set_data(sample_data());
        let s2 = s.clone();
        assert_eq!(s.len(), s2.len());
    }

    #[test]
    fn step_line_two_points_one_horizontal_one_vertical() {
        let mut s = StepLineSeries::new();
        s.set_data(vec![StepPoint::new(0, 100.0), StepPoint::new(100, 150.0)]);
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 100.0, 100.0));
        assert_eq!(cmds.len(), 2); // 1 horizontal + 1 vertical
    }
}
