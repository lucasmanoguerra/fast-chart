use fc_primitives::bar::Bar;
use crate::indicator::Indicator;
use fc_primitives::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

pub struct Bollinger {
    pub period: usize,
    pub k: f64,
}

impl Default for Bollinger {
    fn default() -> Self {
        Self {
            period: 20,
            k: 2.0,
        }
    }
}

pub struct BollingerResult {
    pub upper: TimeSeries<f64, MAX_SERIES_LEN>,
    pub middle: TimeSeries<f64, MAX_SERIES_LEN>,
    pub lower: TimeSeries<f64, MAX_SERIES_LEN>,
}

impl Bollinger {
    pub fn calculate_full(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> BollingerResult {
        let mut upper = TimeSeries::new();
        let mut middle = TimeSeries::new();
        let mut lower = TimeSeries::new();
        let bars: Vec<&Bar> = series.iter().collect();
        if bars.len() < self.period {
            return BollingerResult {
                upper,
                middle,
                lower,
            };
        }

        for i in (self.period - 1)..bars.len() {
            let window: Vec<f64> = bars[(i + 1 - self.period)..=i]
                .iter()
                .map(|b| b.close)
                .collect();
            let mean = window.iter().sum::<f64>() / self.period as f64;
            let variance = window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / self.period as f64;
            let stddev = variance.sqrt();
            middle.push(mean);
            upper.push(mean + self.k * stddev);
            lower.push(mean - self.k * stddev);
        }

        BollingerResult {
            upper,
            middle,
            lower,
        }
    }
}

impl Indicator<MAX_SERIES_LEN> for Bollinger {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        self.calculate_full(series).middle
    }

    fn name(&self) -> &str {
        "BOLLINGER"
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
                high: p + 1.0,
                low: p - 1.0,
                close: p + 0.5,
                volume: 1000,
            });
        }
        s
    }

    #[test]
    fn bollinger_name() {
        let bb = Bollinger::default();
        assert_eq!(bb.name(), "BOLLINGER");
    }

    #[test]
    fn bollinger_insufficient_data() {
        let bars = make_bars(5);
        let bb = Bollinger::default();
        let result = bb.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn bollinger_band_ordering() {
        let bars = make_bars(100);
        let bb = Bollinger::default();
        let full = bb.calculate_full(&bars);
        assert_eq!(full.upper.len(), full.middle.len());
        assert_eq!(full.lower.len(), full.middle.len());
        for i in 0..full.middle.len() {
            assert!(full.upper.get(i).unwrap() >= full.middle.get(i).unwrap());
            assert!(full.middle.get(i).unwrap() >= full.lower.get(i).unwrap());
        }
    }

    #[test]
    fn bollinger_middle_matches_sma() {
        let bars = make_bars(100);
        let period = 20;
        let bb = Bollinger {
            period,
            k: 2.0,
        };
        let full = bb.calculate_full(&bars);

        let mut sum = 0.0;
        for i in 0..period {
            sum += bars.get(i).unwrap().close;
        }
        let expected_first = sum / period as f64;
        let actual_first = *full.middle.get(0).unwrap();
        assert!((actual_first - expected_first).abs() < 1e-10);
    }

    #[test]
    fn bollinger_default_params() {
        let bb = Bollinger::default();
        assert_eq!(bb.period, 20);
        assert_eq!(bb.k, 2.0);
    }
}
