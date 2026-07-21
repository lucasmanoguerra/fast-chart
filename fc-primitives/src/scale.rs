/// Maps a value range to pixel y-coordinates (Y-axis, inverted).
///
/// # Examples
///
/// ```
/// use fc_primitives::scale::LinearScale;
///
/// let scale = LinearScale { min: 90.0, max: 120.0, height: 300.0 };
///
/// // Value at min maps to bottom (y = height)
/// assert_eq!(scale.map_to_y(90.0), 300.0);
///
/// // Value at max maps to top (y = 0)
/// assert_eq!(scale.map_to_y(120.0), 0.0);
///
/// // Roundtrip: map_to_y -> map_from_y preserves value
/// let y = scale.map_to_y(105.0);
/// let value = scale.map_from_y(y);
/// assert!((value - 105.0).abs() < f64::EPSILON);
/// ```
#[derive(Debug, Clone)]
pub struct LinearScale {
    pub min: f64,
    pub max: f64,
    pub height: f64,
}

impl LinearScale {
    pub fn map_to_y(&self, value: f64) -> f64 {
        if (self.max - self.min).abs() < f64::EPSILON {
            return self.height / 2.0;
        }
        let ratio = (value - self.min) / (self.max - self.min);
        self.height * (1.0 - ratio)
    }

    pub fn map_from_y(&self, y: f64) -> f64 {
        if self.height.abs() < f64::EPSILON {
            return (self.min + self.max) / 2.0;
        }
        let ratio = 1.0 - (y / self.height);
        self.min + ratio * (self.max - self.min)
    }
}

/// Maps a time range to pixel x-coordinates.
///
/// # Examples
///
/// ```
/// use fc_primitives::scale::TimeScale;
///
/// let ts = TimeScale { start: 0, end: 2000, width: 800.0, bar_spacing: 8.0, right_offset: 0.0 };
///
/// // Time at start maps to x = 0
/// assert_eq!(ts.map_to_x(0), 0.0);
///
/// // Time at end maps to x = width
/// assert_eq!(ts.map_to_x(2000), 800.0);
///
/// // Roundtrip: map_to_x -> map_from_x preserves time
/// let x = ts.map_to_x(1000);
/// let time = ts.map_from_x(x);
/// assert_eq!(time, 1000);
/// ```
#[derive(Debug, Clone)]
pub struct TimeScale {
    pub start: u64,
    pub end: u64,
    pub width: f64,
    /// Pixels per bar. Determines how many bars fit in the visible area.
    pub bar_spacing: f64,
    /// Offset from the right edge in pixels. Positive = bars don't reach the edge.
    pub right_offset: f64,
}

impl TimeScale {
    pub fn map_to_x(&self, time: u64) -> f64 {
        let range = self.end as f64 - self.start as f64;
        if range < f64::EPSILON {
            return self.width / 2.0;
        }
        let ratio = (time as f64 - self.start as f64) / range;
        self.width * ratio
    }

    pub fn map_from_x(&self, x: f64) -> u64 {
        let range = self.end as f64 - self.start as f64;
        if self.width < f64::EPSILON {
            return (self.start + self.end) / 2;
        }
        let ratio = x / self.width;
        (self.start as f64 + ratio * range) as u64
    }

    /// Number of bars that fit in the visible area.
    pub fn visible_bars(&self) -> usize {
        if self.bar_spacing <= 0.0 {
            return 0;
        }
        ((self.width - self.right_offset) / self.bar_spacing).floor() as usize
    }

    /// Scroll so the last bar is at the right edge (with right_offset).
    pub fn scroll_to_end(&mut self, data_len: usize) {
        if data_len == 0 || self.bar_spacing <= 0.0 {
            return;
        }
        let visible = self.visible_bars();
        let visible = visible.max(1);
        let last_bar_index = data_len.saturating_sub(1);
        let first_visible = last_bar_index.saturating_sub(visible - 1);

        self.start = first_visible as u64;
        self.end = last_bar_index as u64;
    }

    /// Which data indices are currently visible.
    ///
    /// Returns `(first_visible, last_visible)` inclusive. Clamps to valid range.
    pub fn visible_range(&self, data_len: usize) -> (usize, usize) {
        if data_len == 0 {
            return (0, 0);
        }
        let first = self.start as usize;
        let last = self.end as usize;
        let first = first.min(data_len.saturating_sub(1));
        let last = last.min(data_len.saturating_sub(1));
        (first, last)
    }

    /// Set bar spacing and recompute start/end to keep the same right edge.
    pub fn set_bar_spacing(&mut self, new_spacing: f64) {
        if new_spacing <= 0.0 {
            return;
        }
        let visible = ((self.width - self.right_offset) / new_spacing).floor() as u64;
        let visible = visible.max(1);
        self.bar_spacing = new_spacing;
        self.start = self.end.saturating_sub(visible.saturating_sub(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_midpoint() {
        let scale = LinearScale {
            min: 100.0,
            max: 110.0,
            height: 500.0,
        };
        assert_eq!(scale.map_to_y(105.0), 250.0);
    }

    #[test]
    fn linear_top() {
        let scale = LinearScale {
            min: 100.0,
            max: 110.0,
            height: 500.0,
        };
        assert_eq!(scale.map_to_y(110.0), 0.0);
    }

    #[test]
    fn linear_bottom() {
        let scale = LinearScale {
            min: 100.0,
            max: 110.0,
            height: 500.0,
        };
        assert_eq!(scale.map_to_y(100.0), 500.0);
    }

    #[test]
    fn linear_roundtrip() {
        let scale = LinearScale {
            min: 50.0,
            max: 200.0,
            height: 600.0,
        };
        let value = 125.0;
        let y = scale.map_to_y(value);
        let back = scale.map_from_y(y);
        assert!((back - value).abs() < f64::EPSILON);
    }

    #[test]
    fn linear_equal_min_max() {
        let scale = LinearScale {
            min: 100.0,
            max: 100.0,
            height: 500.0,
        };
        assert_eq!(scale.map_to_y(100.0), 250.0);
    }

    #[test]
    fn time_midpoint() {
        let scale = TimeScale {
            start: 0,
            end: 1000,
            width: 800.0,
            bar_spacing: 8.0,
            right_offset: 0.0,
        };
        assert_eq!(scale.map_to_x(500), 400.0);
    }

    #[test]
    fn time_start() {
        let scale = TimeScale {
            start: 0,
            end: 1000,
            width: 800.0,
            bar_spacing: 8.0,
            right_offset: 0.0,
        };
        assert_eq!(scale.map_to_x(0), 0.0);
    }

    #[test]
    fn time_end() {
        let scale = TimeScale {
            start: 0,
            end: 1000,
            width: 800.0,
            bar_spacing: 8.0,
            right_offset: 0.0,
        };
        assert_eq!(scale.map_to_x(1000), 800.0);
    }

    #[test]
    fn time_roundtrip() {
        let scale = TimeScale {
            start: 1000,
            end: 2000,
            width: 600.0,
            bar_spacing: 6.0,
            right_offset: 0.0,
        };
        let time = 1500u64;
        let x = scale.map_to_x(time);
        let back = scale.map_from_x(x);
        assert_eq!(back, time);
    }

    #[test]
    fn time_equal_start_end() {
        let scale = TimeScale {
            start: 1000,
            end: 1000,
            width: 800.0,
            bar_spacing: 8.0,
            right_offset: 0.0,
        };
        assert_eq!(scale.map_to_x(1000), 400.0);
    }

    // ---- New tests for bar_spacing, right_offset, scroll, visible_range ----

    #[test]
    fn visible_bars() {
        let scale = TimeScale {
            start: 0,
            end: 99,
            width: 800.0,
            bar_spacing: 8.0,
            right_offset: 0.0,
        };
        assert_eq!(scale.visible_bars(), 100);
    }

    #[test]
    fn visible_bars_with_offset() {
        let scale = TimeScale {
            start: 0,
            end: 99,
            width: 800.0,
            bar_spacing: 8.0,
            right_offset: 40.0,
        };
        // (800 - 40) / 8 = 95
        assert_eq!(scale.visible_bars(), 95);
    }

    #[test]
    fn visible_bars_zero_spacing() {
        let scale = TimeScale {
            start: 0,
            end: 100,
            width: 800.0,
            bar_spacing: 0.0,
            right_offset: 0.0,
        };
        assert_eq!(scale.visible_bars(), 0);
    }

    #[test]
    fn scroll_to_end() {
        let mut scale = TimeScale {
            start: 0,
            end: 200,
            width: 800.0,
            bar_spacing: 8.0,
            right_offset: 0.0,
        };
        scale.scroll_to_end(500);
        // visible_bars = 100, last = 499, first = 400
        assert_eq!(scale.start, 400);
        assert_eq!(scale.end, 499);
    }

    #[test]
    fn scroll_to_end_short_data() {
        let mut scale = TimeScale {
            start: 0,
            end: 200,
            width: 800.0,
            bar_spacing: 8.0,
            right_offset: 0.0,
        };
        scale.scroll_to_end(10);
        // Only 10 bars, all visible
        assert_eq!(scale.start, 0);
        assert_eq!(scale.end, 9);
    }

    #[test]
    fn scroll_to_end_empty() {
        let mut scale = TimeScale {
            start: 0,
            end: 0,
            width: 800.0,
            bar_spacing: 8.0,
            right_offset: 0.0,
        };
        scale.scroll_to_end(0);
        // No change
        assert_eq!(scale.start, 0);
        assert_eq!(scale.end, 0);
    }

    #[test]
    fn visible_range() {
        let scale = TimeScale {
            start: 100,
            end: 200,
            width: 800.0,
            bar_spacing: 8.0,
            right_offset: 0.0,
        };
        let (first, last) = scale.visible_range(500);
        assert_eq!(first, 100);
        assert_eq!(last, 200);
    }

    #[test]
    fn visible_range_clamped() {
        let scale = TimeScale {
            start: 100,
            end: 600,
            width: 800.0,
            bar_spacing: 8.0,
            right_offset: 0.0,
        };
        let (first, last) = scale.visible_range(150);
        assert_eq!(first, 100);
        assert_eq!(last, 149); // clamped to data_len - 1
    }

    #[test]
    fn visible_range_empty() {
        let scale = TimeScale {
            start: 0,
            end: 100,
            width: 800.0,
            bar_spacing: 8.0,
            right_offset: 0.0,
        };
        let (first, last) = scale.visible_range(0);
        assert_eq!(first, 0);
        assert_eq!(last, 0);
    }

    #[test]
    fn set_bar_spacing() {
        let mut scale = TimeScale {
            start: 0,
            end: 200,
            width: 800.0,
            bar_spacing: 8.0,
            right_offset: 0.0,
        };
        scale.set_bar_spacing(4.0);
        // visible = 800/4 = 200, end = 200, start = 200 - 199 = 1
        assert_eq!(scale.bar_spacing, 4.0);
        assert_eq!(scale.end, 200);
        assert_eq!(scale.start, 1);
    }

    #[test]
    fn set_bar_spacing_with_offset() {
        let mut scale = TimeScale {
            start: 0,
            end: 200,
            width: 800.0,
            bar_spacing: 8.0,
            right_offset: 40.0,
        };
        scale.set_bar_spacing(10.0);
        // visible = (800-40)/10 = 76, end = 200, start = 200 - 75 = 125
        assert_eq!(scale.start, 125);
        assert_eq!(scale.end, 200);
    }
}
