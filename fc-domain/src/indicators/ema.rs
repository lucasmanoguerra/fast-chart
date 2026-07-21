use fc_primitives::bar::Bar;
use crate::indicator::Indicator;
use fc_primitives::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

pub struct Ema {
    pub period: usize,
}

impl Indicator<MAX_SERIES_LEN> for Ema {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        let mut result = TimeSeries::new();
        let bars: Vec<&Bar> = series.iter().collect();
        if bars.len() < self.period {
            return result;
        }

        let alpha = 2.0 / (self.period as f64 + 1.0);
        let first_sma: f64 =
            bars[..self.period].iter().map(|b| b.close).sum::<f64>() / self.period as f64;
        result.push(first_sma);

        let mut prev = first_sma;
        for i in self.period..bars.len() {
            let ema = alpha * bars[i].close + (1.0 - alpha) * prev;
            result.push(ema);
            prev = ema;
        }
        result
    }

    fn name(&self) -> &str {
        "EMA"
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
    fn ema_name() {
        let ema = Ema { period: 12 };
        assert_eq!(ema.name(), "EMA");
    }

    #[test]
    fn ema_insufficient_data() {
        let bars = make_bars(5);
        let ema = Ema { period: 12 };
        let result = ema.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn ema_basic() {
        let bars = make_bars(100);
        let ema = Ema { period: 12 };
        let result = ema.calculate(&bars);
        assert_eq!(result.len(), 89);
        assert!(result.iter().all(|v| *v > 0.0));
    }

    #[test]
    fn ema_first_value_matches_sma() {
        let bars = make_bars(30);
        let period = 10;
        let ema = Ema { period };
        let result = ema.calculate(&bars);

        let sma_val: f64 = bars.iter().take(period).map(|b| b.close).sum::<f64>() / period as f64;
        assert!((result.get(0).unwrap() - sma_val).abs() < 1e-10);
    }

    #[test]
    fn ema_converges_to_constant_input() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..50 {
            bars.push(Bar {
                timestamp: i * 60000,
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000,
            });
        }
        let ema = Ema { period: 10 };
        let result = ema.calculate(&bars);
        let last = *result.latest().unwrap();
        assert!((last - 100.0).abs() < 1e-10);
    }
}
