use fc_primitives::bar::Bar;
use crate::indicator::Indicator;
use fc_primitives::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

/// Average True Range.
///
/// True Range = max(High - Low, |High - PrevClose|, |Low - PrevClose|)
///
/// Uses Wilder's smoothing (exponential moving average):
/// - First ATR = SMA of True Range over the period
/// - Subsequent ATR = (prev_ATR × (period - 1) + TR) / period
///
/// Default period: 14
pub struct Atr {
    pub period: usize,
}

impl Indicator<MAX_SERIES_LEN> for Atr {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        let mut result = TimeSeries::new();
        let bars: Vec<&Bar> = series.iter().collect();
        if bars.len() < self.period + 1 {
            return result;
        }

        let mut tr_values: Vec<f64> = Vec::with_capacity(bars.len() - 1);
        for i in 1..bars.len() {
            let hl = bars[i].high - bars[i].low;
            let hpc = (bars[i].high - bars[i - 1].close).abs();
            let lpc = (bars[i].low - bars[i - 1].close).abs();
            tr_values.push(f64::max(hl, f64::max(hpc, lpc)));
        }

        let initial_atr: f64 = tr_values[..self.period].iter().sum::<f64>() / self.period as f64;
        result.push(initial_atr);

        let mut prev_atr = initial_atr;
        for &tr in &tr_values[self.period..] {
            let atr = (prev_atr * (self.period as f64 - 1.0) + tr) / self.period as f64;
            result.push(atr);
            prev_atr = atr;
        }

        result
    }

    fn name(&self) -> &str {
        "ATR"
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

    // Clasificación: determinística — verifica atr_name
    #[test]
    fn atr_name() {
        let atr = Atr { period: 14 };
        assert_eq!(atr.name(), "ATR");
    }

    // Clasificación: determinística — verifica atr_insufficient_data
    #[test]
    fn atr_insufficient_data() {
        let bars = make_bars(10);
        let atr = Atr { period: 14 };
        let result = atr.calculate(&bars);
        assert!(result.is_empty());
    }

    // Clasificación: determinística — verifica atr_basic
    #[test]
    fn atr_basic() {
        let bars = make_bars(50);
        let atr = Atr { period: 14 };
        let result = atr.calculate(&bars);
        assert_eq!(result.len(), 36);
        assert!(result.iter().all(|v| *v >= 0.0));
    }

    // Clasificación: determinística — verifica atr_exact_period
    #[test]
    fn atr_exact_period() {
        let bars = make_bars(15);
        let atr = Atr { period: 14 };
        let result = atr.calculate(&bars);
        assert_eq!(result.len(), 1);
    }

    // Clasificación: determinística — verifica atr_constant_range
    #[test]
    fn atr_constant_range() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..20 {
            bars.push(Bar {
                timestamp: i * 60000,
                open: 105.0,
                high: 110.0,
                low: 100.0,
                close: 105.0,
                volume: 1000,
            });
        }
        let atr = Atr { period: 14 };
        let result = atr.calculate(&bars);
        assert!(result.iter().all(|v| (*v - 10.0).abs() < 1e-10));
    }

    // Clasificación: determinística — verifica atr_always_non_negative
    #[test]
    fn atr_always_non_negative() {
        let bars = make_bars(100);
        let atr = Atr { period: 14 };
        let result = atr.calculate(&bars);
        assert!(result.iter().all(|v| *v >= 0.0));
    }
}
