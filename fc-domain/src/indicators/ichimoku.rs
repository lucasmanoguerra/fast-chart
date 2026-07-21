use fc_primitives::bar::Bar;
use crate::indicator::Indicator;
use fc_primitives::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

pub struct Ichimoku {
    pub tenkan: usize,
    pub kijun: usize,
    pub senkou_b: usize,
}

impl Default for Ichimoku {
    fn default() -> Self {
        Self {
            tenkan: 9,
            kijun: 26,
            senkou_b: 52,
        }
    }
}

pub struct IchimokuResult {
    pub tenkan_sen: TimeSeries<f64, MAX_SERIES_LEN>,
    pub kijun_sen: TimeSeries<f64, MAX_SERIES_LEN>,
    pub senkou_a: TimeSeries<f64, MAX_SERIES_LEN>,
    pub senkou_b: TimeSeries<f64, MAX_SERIES_LEN>,
}

impl Ichimoku {
    pub fn calculate_full(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> IchimokuResult {
        let bars: Vec<&Bar> = series.iter().collect();
        let mut tenkan_sen = TimeSeries::new();
        let mut kijun_sen = TimeSeries::new();

        if bars.len() < self.tenkan {
            return IchimokuResult {
                tenkan_sen,
                kijun_sen,
                senkou_a: TimeSeries::new(),
                senkou_b: TimeSeries::new(),
            };
        }

        for i in (self.tenkan - 1)..bars.len() {
            let window = &bars[(i + 1 - self.tenkan)..=i];
            let highest = window.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
            let lowest = window.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
            tenkan_sen.push((highest + lowest) / 2.0);
        }

        if bars.len() >= self.kijun {
            for i in (self.kijun - 1)..bars.len() {
                let window = &bars[(i + 1 - self.kijun)..=i];
                let highest = window.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
                let lowest = window.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
                kijun_sen.push((highest + lowest) / 2.0);
            }
        }

        let tenkan_vals: Vec<f64> = tenkan_sen.iter().copied().collect();
        let kijun_vals: Vec<f64> = kijun_sen.iter().copied().collect();
        let mut senkou_a = TimeSeries::new();

        let kijun_offset = self.kijun - self.tenkan;
        for i in 0..kijun_vals.len() {
            if i + kijun_offset < tenkan_vals.len() {
                senkou_a.push((tenkan_vals[i + kijun_offset] + kijun_vals[i]) / 2.0);
            }
        }

        let mut senkou_b_vals = TimeSeries::new();
        if bars.len() >= self.senkou_b {
            for i in (self.senkou_b - 1)..bars.len() {
                let window = &bars[(i + 1 - self.senkou_b)..=i];
                let highest = window.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
                let lowest = window.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
                senkou_b_vals.push((highest + lowest) / 2.0);
            }
        }

        IchimokuResult {
            tenkan_sen,
            kijun_sen,
            senkou_a,
            senkou_b: senkou_b_vals,
        }
    }
}

impl Indicator<MAX_SERIES_LEN> for Ichimoku {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        self.calculate_full(series).tenkan_sen
    }

    fn name(&self) -> &str {
        "ICHIMOKU"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fc_primitives::bar::Bar;

    fn make_bars(count: usize) -> TimeSeries<Bar, MAX_SERIES_LEN> {
        let mut s = TimeSeries::new();
        for i in 0..count {
            let p = 100.0 + (i as f64 * 0.1).sin() * 10.0;
            s.push(Bar {
                timestamp: i as u64 * 60000,
                open: p,
                high: p + 3.0,
                low: p - 3.0,
                close: p + 0.5,
                volume: 1000,
            });
        }
        s
    }

    #[test]
    fn ichimoku_name() {
        let ich = Ichimoku::default();
        assert_eq!(ich.name(), "ICHIMOKU");
    }

    #[test]
    fn ichimoku_insufficient_data() {
        let bars = make_bars(5);
        let ich = Ichimoku::default();
        let result = ich.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn ichimoku_tenkan_length() {
        let bars = make_bars(100);
        let ich = Ichimoku::default();
        let full = ich.calculate_full(&bars);
        assert_eq!(full.tenkan_sen.len(), 100 - ich.tenkan + 1);
    }

    #[test]
    fn ichimoku_kijun_length() {
        let bars = make_bars(100);
        let ich = Ichimoku::default();
        let full = ich.calculate_full(&bars);
        assert_eq!(full.kijun_sen.len(), 100 - ich.kijun + 1);
    }

    #[test]
    fn ichimoku_tenkan_midpoint() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..10 {
            bars.push(Bar {
                timestamp: i * 60000,
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 105.0,
                volume: 1000,
            });
        }
        let ich = Ichimoku {
            tenkan: 5,
            kijun: 10,
            senkou_b: 10,
        };
        let full = ich.calculate_full(&bars);
        assert_eq!(*full.tenkan_sen.get(0).unwrap(), 100.0);
    }

    #[test]
    fn ichimoku_default_params() {
        let ich = Ichimoku::default();
        assert_eq!(ich.tenkan, 9);
        assert_eq!(ich.kijun, 26);
        assert_eq!(ich.senkou_b, 52);
    }
}
