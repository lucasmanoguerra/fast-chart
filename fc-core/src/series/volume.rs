// ---------------------------------------------------------------------------
// VolumeSeries — histogram bars colored by price direction
// ---------------------------------------------------------------------------

use crate::render::commands::DrawCommand;
use crate::render::series_renderer::{Rect, SeriesHit, SeriesRenderer};

/// A data point for a volume series.
///
/// Contains a timestamp, volume, and the price direction (bullish/bearish)
/// to determine bar color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolumeBar {
    pub timestamp: u64,
    pub volume: f64,
    /// True when close >= open (bullish bar).
    pub bullish: bool,
}

impl VolumeBar {
    pub fn new(timestamp: u64, volume: f64, bullish: bool) -> Self {
        Self {
            timestamp,
            volume,
            bullish,
        }
    }
}

/// Volume series renderer.
///
/// Renders histogram bars at the bottom of a pane, colored by price
/// direction: green for bullish (close >= open), red for bearish.
/// Can be overlaid on the main chart or rendered in a separate pane.
#[derive(Debug, Clone)]
pub struct VolumeSeries {
    /// Data points in chronological order.
    data: Vec<VolumeBar>,
    /// Color for bullish bars [r, g, b, a].
    pub bullish_color: [f32; 4],
    /// Color for bearish bars [r, g, b, a].
    pub bearish_color: [f32; 4],
    /// Bar width as fraction of available space (0.0 – 1.0).
    pub bar_ratio: f32,
    /// Maximum bars to render (0 = unlimited).
    pub max_bars: usize,
    /// Bounding rect (set after update).
    bounds: Rect,
}

impl VolumeSeries {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            bullish_color: [0.2, 0.8, 0.2, 0.7],
            bearish_color: [0.8, 0.2, 0.2, 0.7],
            bar_ratio: 0.8,
            max_bars: 0,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Set the data points for this series.
    pub fn set_data(&mut self, data: Vec<VolumeBar>) {
        self.data = data;
    }

    /// Get the data points.
    pub fn data(&self) -> &[VolumeBar] {
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

    /// Maximum volume value.
    pub fn max_volume(&self) -> f64 {
        self.data
            .iter()
            .map(|b| b.volume)
            .fold(0.0_f64, f64::max)
    }

    /// Generate volume bar draw commands.
    fn generate_commands(&self, bounds: Rect) -> Vec<DrawCommand> {
        if self.data.is_empty() {
            return Vec::new();
        }

        let count = self.data.len();
        let max_vol = self.max_volume();
        if max_vol <= 0.0 {
            return Vec::new();
        }

        let slot_width = bounds.width / count as f32;
        let bar_width = slot_width * self.bar_ratio;
        let half_gap = (slot_width - bar_width) / 2.0;

        let mut commands = Vec::with_capacity(count);

        for (i, bar) in self.data.iter().enumerate() {
            let bar_height = (bar.volume / max_vol) as f32 * bounds.height;
            let x = bounds.x + i as f32 * slot_width + half_gap;
            let y = bounds.y + bounds.height - bar_height;

            let color = if bar.bullish {
                self.bullish_color
            } else {
                self.bearish_color
            };

            commands.push(DrawCommand::DrawRect {
                x,
                y,
                width: bar_width,
                height: bar_height,
                fill: Some(color),
                stroke: None,
                stroke_width: 0.0,
                z_index: 500,
            });
        }

        commands
    }
}

impl Default for VolumeSeries {
    fn default() -> Self {
        Self::new()
    }
}

impl SeriesRenderer for VolumeSeries {
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

    fn layer_z_index(&self) -> i32 {
        500 // below candle layer (600)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> Vec<VolumeBar> {
        vec![
            VolumeBar::new(1000, 5000.0, true),
            VolumeBar::new(2000, 3000.0, false),
            VolumeBar::new(3000, 7000.0, true),
            VolumeBar::new(4000, 2000.0, false),
        ]
    }

    #[test]
    fn volume_bar_new() {
        let b = VolumeBar::new(1000, 5000.0, true);
        assert_eq!(b.timestamp, 1000);
        assert!((b.volume - 5000.0).abs() < f64::EPSILON);
        assert!(b.bullish);
    }

    #[test]
    fn volume_series_new() {
        let s = VolumeSeries::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn volume_series_set_data() {
        let mut s = VolumeSeries::new();
        let data = sample_data();
        s.set_data(data.clone());
        assert_eq!(s.len(), 4);
        assert_eq!(s.data(), &data);
    }

    #[test]
    fn volume_series_max_volume() {
        let mut s = VolumeSeries::new();
        s.set_data(sample_data());
        assert!((s.max_volume() - 7000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn volume_series_empty_max_volume() {
        let s = VolumeSeries::new();
        assert!((s.max_volume()).abs() < f64::EPSILON);
    }

    #[test]
    fn volume_series_generates_rect_commands() {
        let mut s = VolumeSeries::new();
        s.set_data(sample_data());
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
        assert_eq!(cmds.len(), 4);

        for cmd in &cmds {
            match cmd {
                DrawCommand::DrawRect { .. } => {}
                _ => panic!("expected DrawRect commands"),
            }
        }
    }

    #[test]
    fn volume_series_empty_no_commands() {
        let mut s = VolumeSeries::new();
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
        assert!(cmds.is_empty());
    }

    #[test]
    fn volume_series_zero_volume_no_commands() {
        let mut s = VolumeSeries::new();
        s.set_data(vec![VolumeBar::new(1000, 0.0, true)]);
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
        assert!(cmds.is_empty());
    }

    #[test]
    fn volume_series_hit_test() {
        let mut s = VolumeSeries::new();
        s.set_data(sample_data());
        let bounds = Rect::new(0.0, 0.0, 800.0, 400.0);
        s.update(&[], bounds);

        let hit = s.hit_test(100.0, 200.0);
        assert!(hit.is_some());
        assert!(hit.unwrap().index < 4);
    }

    #[test]
    fn volume_series_hit_test_empty() {
        let s = VolumeSeries::new();
        assert!(s.hit_test(0.0, 0.0).is_none());
    }

    #[test]
    fn volume_series_layer_z_index() {
        let s = VolumeSeries::new();
        assert_eq!(s.layer_z_index(), 500);
    }

    #[test]
    fn volume_series_bounds_set_after_update() {
        let mut s = VolumeSeries::new();
        s.set_data(sample_data());
        let bounds = Rect::new(10.0, 20.0, 780.0, 380.0);
        s.update(&[], bounds);
        assert_eq!(s.bounds(), bounds);
    }

    #[test]
    fn volume_series_bullish_and_bearish_colors() {
        let mut s = VolumeSeries::new();
        s.set_data(sample_data());
        let cmds = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));

        // First bar is bullish, second is bearish
        if let DrawCommand::DrawRect { fill: Some(color), .. } = &cmds[0] {
            assert_eq!(*color, s.bullish_color);
        } else {
            panic!("expected fill color");
        }

        if let DrawCommand::DrawRect { fill: Some(color), .. } = &cmds[1] {
            assert_eq!(*color, s.bearish_color);
        } else {
            panic!("expected fill color");
        }
    }

    #[test]
    fn volume_series_default() {
        let s = VolumeSeries::default();
        assert!(s.is_empty());
    }
}
