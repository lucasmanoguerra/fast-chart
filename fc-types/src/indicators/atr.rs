use crate::bar::Bar;
use crate::indicator::Indicator;
use crate::series::TimeSeries;
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
        // Need at least period + 1 bars: period bars for first TR, plus the
        // initial SMA window requires bars[0..period] with TR starting at index 1.
        // Total TR values available = bars.len() - 1.  We need period TR values
        // for the initial SMA.
        if bars.len() < self.period + 1 {
            return result;
        }

        // Compute all True Range values
        let mut tr_values: Vec<f64> = Vec::with_capacity(bars.len() - 1);
        for i in 1..bars.len() {
            let hl = bars[i].high - bars[i].low;
            let hpc = (bars[i].high - bars[i - 1].close).abs();
            let lpc = (bars[i].low - bars[i - 1].close).abs();
            tr_values.push(f64::max(hl, f64::max(hpc, lpc)));
        }

        // First ATR = SMA of first `period` True Range values
        let initial_atr: f64 = tr_values[..self.period].iter().sum::<f64>() / self.period as f64;
        result.push(initial_atr);

        // Wilder's smoothing for subsequent values
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
    use crate::bar::Bar;

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
    fn atr_name() {
        let atr = Atr { period: 14 };
        assert_eq!(atr.name(), "ATR");
    }

    #[test]
    fn atr_insufficient_data() {
        let bars = make_bars(10);
        let atr = Atr { period: 14 };
        let result = atr.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn atr_basic() {
        let bars = make_bars(50);
        let atr = Atr { period: 14 };
        let result = atr.calculate(&bars);
        // Need 14+1=15 bars minimum, produces (50-1) - 14 + 1 = 36 outputs
        assert_eq!(result.len(), 36);
        assert!(result.iter().all(|v| *v >= 0.0));
    }

    #[test]
    fn atr_exact_period() {
        // Need exactly period + 1 bars for exactly 1 output
        let bars = make_bars(15); // 14 + 1
        let atr = Atr { period: 14 };
        let result = atr.calculate(&bars);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn atr_constant_range() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        // All bars have the same range: high=110, low=100, close=105
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
        // Every TR = 10.0 (high - low), so initial ATR = 10.0, and Wilder's
        // smoothing on constant input converges to 10.0 immediately.
        assert!(result.iter().all(|v| (*v - 10.0).abs() < 1e-10));
    }

    #[test]
    fn atr_always_non_negative() {
        let bars = make_bars(100);
        let atr = Atr { period: 14 };
        let result = atr.calculate(&bars);
        assert!(result.iter().all(|v| *v >= 0.0));
    }
}
