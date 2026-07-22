use fc_primitives::bar::Bar;
use crate::indicator::Indicator;
use fc_primitives::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

/// SuperTrend — ATR-based trend following indicator.
///
/// Formulas:
/// - HL2        = (High + Low) / 2
/// - Upper Band = HL2 + multiplier × ATR
/// - Lower Band = HL2 - multiplier × ATR
///
/// The final bands are adjusted to avoid switching when price hasn't
/// clearly broken the band.  The SuperTrend line follows the Lower Band
/// in an uptrend and the Upper Band in a downtrend.
///
/// Default period: 10, default multiplier: 3.0
pub struct Supertrend {
    pub period: usize,
    pub multiplier: f64,
}

impl Default for Supertrend {
    fn default() -> Self {
        Self {
            period: 10,
            multiplier: 3.0,
        }
    }
}

impl Indicator<MAX_SERIES_LEN> for Supertrend {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        let mut result = TimeSeries::new();
        let bars: Vec<&Bar> = series.iter().collect();

        // Need at least period + 1 bars: period bars for the first TR-based
        // SMA, plus bar[0] to compute the first TR at bar[1].
        if bars.len() < self.period + 1 {
            return result;
        }

        // ── True Range ──────────────────────────────────────────────
        // tr[i] is the TR of bar[i + 1] vs bar[i]; length = bars.len() - 1.
        let mut tr_values: Vec<f64> = Vec::with_capacity(bars.len() - 1);
        for i in 1..bars.len() {
            let hl = bars[i].high - bars[i].low;
            let hpc = (bars[i].high - bars[i - 1].close).abs();
            let lpc = (bars[i].low - bars[i - 1].close).abs();
            tr_values.push(f64::max(hl, f64::max(hpc, lpc)));
        }

        // ── ATR (Wilder's smoothing) ───────────────────────────────
        // atr[0] = SMA of tr_values[0..period]  → corresponds to bar[period]
        // atr[j] corresponds to bar[period + j]
        let num_atr = tr_values.len() - self.period + 1; // = bars.len() - period
        let mut atr_values: Vec<f64> = Vec::with_capacity(num_atr);

        let initial_atr: f64 =
            tr_values[..self.period].iter().sum::<f64>() / self.period as f64;
        atr_values.push(initial_atr);

        let mut prev_atr = initial_atr;
        for &tr in &tr_values[self.period..] {
            let atr = (prev_atr * (self.period as f64 - 1.0) + tr) / self.period as f64;
            atr_values.push(atr);
            prev_atr = atr;
        }

        // ── SuperTrend computation ──────────────────────────────────
        // For bar[i] (i = period .. bars.len()-1), ATR index = i - period.
        let start = self.period; // first bar index with ATR
        let count = bars.len() - start;

        let mut final_upper: Vec<f64> = Vec::with_capacity(count);
        let mut final_lower: Vec<f64> = Vec::with_capacity(count);
        let mut supertrend: Vec<f64> = Vec::with_capacity(count);

        for j in 0..count {
            let i = start + j;
            let hl2 = (bars[i].high + bars[i].low) / 2.0;
            let atr = atr_values[j];
            let basic_upper = hl2 + self.multiplier * atr;
            let basic_lower = hl2 - self.multiplier * atr;

            if j == 0 {
                // First value: final bands = basic bands.
                final_upper.push(basic_upper);
                final_lower.push(basic_lower);
                // Default initial direction to uptrend (SuperTrend = lower band).
                supertrend.push(basic_lower);
            } else {
                // Final Upper Band
                let prev_close = bars[i - 1].close;
                let fu = if basic_upper < final_upper[j - 1]
                    || prev_close > final_upper[j - 1]
                {
                    basic_upper
                } else {
                    final_upper[j - 1]
                };
                final_upper.push(fu);

                // Final Lower Band
                let fl = if basic_lower > final_lower[j - 1]
                    || prev_close < final_lower[j - 1]
                {
                    basic_lower
                } else {
                    final_lower[j - 1]
                };
                final_lower.push(fl);

                // SuperTrend direction
                let close = bars[i].close;
                let st = if supertrend[j - 1] == final_upper[j - 1] {
                    // Previous downtrend
                    if close > final_upper[j] {
                        final_lower[j] // switch to uptrend
                    } else {
                        final_upper[j] // stay downtrend
                    }
                } else {
                    // Previous uptrend (or initial default)
                    if close < final_lower[j] {
                        final_upper[j] // switch to downtrend
                    } else {
                        final_lower[j] // stay uptrend
                    }
                };
                supertrend.push(st);
            }
        }

        for &v in &supertrend {
            result.push(v);
        }

        result
    }

    fn name(&self) -> &str {
        "SuperTrend"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // Clasificación: determinística — verifica name_returns_correct_string
    #[test]
    fn name_returns_correct_string() {
        let st = Supertrend::default();
        assert_eq!(st.name(), "SuperTrend");
    }

    // Clasificación: determinística — verifica default_values
    #[test]
    fn default_values() {
        let st = Supertrend::default();
        assert_eq!(st.period, 10);
        assert!((st.multiplier - 3.0).abs() < f64::EPSILON);
    }

    // Clasificación: determinística — verifica insufficient_data
    #[test]
    fn insufficient_data() {
        let bars = make_bars(10); // period + 1 = 11 needed
        let st = Supertrend { period: 10, multiplier: 3.0 };
        let result = st.calculate(&bars);
        assert!(result.is_empty());
    }

    // Clasificación: determinística — verifica exact_period_plus_one
    #[test]
    fn exact_period_plus_one() {
        // Exactly period + 1 bars → 1 output value
        let bars = make_bars(11);
        let st = Supertrend { period: 10, multiplier: 3.0 };
        let result = st.calculate(&bars);
        assert_eq!(result.len(), 1);
    }

    // Clasificación: determinística — verifica basic_output_count
    #[test]
    fn basic_output_count() {
        let bars = make_bars(50);
        let st = Supertrend { period: 10, multiplier: 3.0 };
        let result = st.calculate(&bars);
        // Output count = bars.len() - period = 50 - 10 = 40
        assert_eq!(result.len(), 40);
    }

    // Clasificación: determinística — verifica output_values_are_positive
    #[test]
    fn output_values_are_positive() {
        let bars = make_bars(100);
        let st = Supertrend::default();
        let result = st.calculate(&bars);
        assert!(result.iter().all(|v| *v > 0.0));
    }

    // Clasificación: determinística — verifica trend_reversal
    #[test]
    fn trend_reversal() {
        // Create bars that go up then down to trigger a reversal
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        // 15 bars trending up
        for i in 0..15 {
            let base = 100.0 + i as f64;
            series.push(Bar {
                timestamp: i * 60000,
                open: base,
                high: base + 2.0,
                low: base - 0.5,
                close: base + 1.0,
                volume: 1000,
            });
        }
        // 15 bars trending down
        for i in 15..30 {
            let base = 115.0 - (i - 15) as f64;
            series.push(Bar {
                timestamp: i * 60000,
                open: base,
                high: base + 0.5,
                low: base - 2.0,
                close: base - 1.0,
                volume: 1000,
            });
        }
        let st = Supertrend { period: 10, multiplier: 3.0 };
        let result = st.calculate(&series);
        // Should have 30 - 10 = 20 values
        assert_eq!(result.len(), 20);
        // All values positive
        assert!(result.iter().all(|v| *v > 0.0));
        // The last few values should differ from the first few (reversal happened)
        let first = result.iter().next().unwrap();
        let last = result.iter().last().unwrap();
        assert_ne!(first, last);
    }

    // Clasificación: determinística — verifica flat_prices
    #[test]
    fn flat_prices() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..20 {
            series.push(Bar {
                timestamp: i * 60000,
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 1000,
            });
        }
        let st = Supertrend { period: 10, multiplier: 3.0 };
        let result = st.calculate(&series);
        assert_eq!(result.len(), 10);
        // With flat prices, ATR is constant at ~2.0 and SuperTrend should be stable
        assert!(result.iter().all(|v| *v > 0.0));
    }

    // Clasificación: determinística — verifica single_bar
    #[test]
    fn single_bar() {
        let bars = make_bars(1);
        let st = Supertrend::default();
        let result = st.calculate(&bars);
        assert!(result.is_empty());
    }
}
