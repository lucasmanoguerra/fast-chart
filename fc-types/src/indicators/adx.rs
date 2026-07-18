use crate::bar::Bar;
use crate::indicator::Indicator;
use crate::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

/// Average Directional Index.
///
/// ADX measures trend strength on a scale of 0–100.
///
/// Components:
/// - True Range (TR): max(H-L, |H-PrevC|, |L-PrevC|)
/// - +DM: High - PrevHigh (if > 0 and > -DM)
/// - -DM: PrevLow - Low   (if > 0 and > +DM)
/// - ATR: Wilder's smoothed average of TR
/// - +DI: Wilder's smoothed +DM / ATR × 100
/// - -DI: Wilder's smoothed -DM / ATR × 100
/// - DX:  |+DI - -DI| / (+DI + -DI) × 100
/// - ADX: Wilder's smoothed DX
///
/// Default period: 14
pub struct Adx {
    pub period: usize,
}

impl Indicator<MAX_SERIES_LEN> for Adx {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        let mut result = TimeSeries::new();
        let bars: Vec<&Bar> = series.iter().collect();
        // Need period + 1 bars for initial TR/+DM/-DM computation,
        // plus period bars for the first Wilder's smoothing seed.
        // Total: bars with indices 0..period → period+1 bars, producing
        // first smoothed values at index `period`. Then one more per bar.
        // Minimum: period + 1 bars total, but we need at least 2 for TR.
        // Effective minimum: period + 1 bars (index 0 is prev for index 1).
        if bars.len() < self.period + 1 {
            return result;
        }

        // Compute TR, +DM, -DM for each bar (starting from index 1)
        let mut tr_values: Vec<f64> = Vec::with_capacity(bars.len() - 1);
        let mut plus_dm_values: Vec<f64> = Vec::with_capacity(bars.len() - 1);
        let mut minus_dm_values: Vec<f64> = Vec::with_capacity(bars.len() - 1);

        for i in 1..bars.len() {
            let hl = bars[i].high - bars[i].low;
            let hpc = (bars[i].high - bars[i - 1].close).abs();
            let lpc = (bars[i].low - bars[i - 1].close).abs();
            tr_values.push(f64::max(hl, f64::max(hpc, lpc)));

            let up_move = bars[i].high - bars[i - 1].high;
            let down_move = bars[i - 1].low - bars[i].low;

            if up_move > 0.0 && up_move > down_move {
                plus_dm_values.push(up_move);
                minus_dm_values.push(0.0);
            } else if down_move > 0.0 && down_move > up_move {
                plus_dm_values.push(0.0);
                minus_dm_values.push(down_move);
            } else {
                plus_dm_values.push(0.0);
                minus_dm_values.push(0.0);
            }
        }

        // Need at least `period` TR values for initial Wilder's smoothing
        if tr_values.len() < self.period {
            return result;
        }

        // Initial Wilder's smoothing: sum first `period` values / period
        let period_f = self.period as f64;
        let mut atr: f64 = tr_values[..self.period].iter().sum::<f64>() / period_f;
        let mut smooth_plus_dm: f64 =
            plus_dm_values[..self.period].iter().sum::<f64>() / period_f;
        let mut smooth_minus_dm: f64 =
            minus_dm_values[..self.period].iter().sum::<f64>() / period_f;

        // Compute initial DI and DX
        let (_plus_di, _minus_di, dx) = compute_di_dx(atr, smooth_plus_dm, smooth_minus_dm);

        // We need `period` DX values before we can compute ADX via Wilder's smoothing.
        // The first DX was computed from the initial seed. Accumulate the next
        // (period - 1) DX values, then seed ADX with their average.
        let mut dx_accumulator = dx;
        let mut dx_seeded = false;
        let mut adx = 0.0;

        // Process remaining TR/DM values after the initial window
        for (idx, &tr) in tr_values[self.period..].iter().enumerate() {
            let dm_idx = self.period + idx;
            atr = (atr * (period_f - 1.0) + tr) / period_f;
            smooth_plus_dm =
                (smooth_plus_dm * (period_f - 1.0) + plus_dm_values[dm_idx]) / period_f;
            smooth_minus_dm =
                (smooth_minus_dm * (period_f - 1.0) + minus_dm_values[dm_idx]) / period_f;

            let (_, _, dx_val) = compute_di_dx(atr, smooth_plus_dm, smooth_minus_dm);

            if !dx_seeded {
                dx_accumulator += dx_val;
                // We now have (idx + 2) DX values: the initial seed + (idx+1) from this loop
                if idx + 2 >= self.period {
                    adx = dx_accumulator / period_f;
                    result.push(adx);
                    dx_seeded = true;
                }
            } else {
                // Wilder's smoothing for ADX
                adx = (adx * (period_f - 1.0) + dx_val) / period_f;
                result.push(adx);
            }
        }

        result
    }

    fn name(&self) -> &str {
        "ADX"
    }
}

/// Compute +DI, -DI, and DX from smoothed values and ATR.
/// Returns (+DI, -DI, DX).
fn compute_di_dx(atr: f64, smooth_plus_dm: f64, smooth_minus_dm: f64) -> (f64, f64, f64) {
    if atr == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let plus_di = (smooth_plus_dm / atr) * 100.0;
    let minus_di = (smooth_minus_dm / atr) * 100.0;
    let di_sum = plus_di + minus_di;
    let dx = if di_sum == 0.0 {
        0.0
    } else {
        ((plus_di - minus_di).abs() / di_sum) * 100.0
    };
    (plus_di, minus_di, dx)
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
    fn adx_name() {
        let adx = Adx { period: 14 };
        assert_eq!(adx.name(), "ADX");
    }

    #[test]
    fn adx_insufficient_data() {
        let bars = make_bars(10);
        let adx = Adx { period: 14 };
        let result = adx.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn adx_basic() {
        let bars = make_bars(100);
        let adx = Adx { period: 14 };
        let result = adx.calculate(&bars);
        // Produces a non-empty result with enough data
        assert!(!result.is_empty());
    }

    #[test]
    fn adx_bounds() {
        let bars = make_bars(200);
        let adx = Adx { period: 14 };
        let result = adx.calculate(&bars);
        // ADX should be between 0 and 100
        assert!(result.iter().all(|v| *v >= 0.0 && *v <= 100.0));
    }

    #[test]
    fn adx_strong_trend() {
        // Create a strong uptrend: each bar higher than the previous
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..50 {
            let base = 100.0 + i as f64 * 2.0;
            bars.push(Bar {
                timestamp: i * 60000,
                open: base,
                high: base + 3.0,
                low: base + 1.0,
                close: base + 2.0,
                volume: 1000,
            });
        }
        let adx = Adx { period: 14 };
        let result = adx.calculate(&bars);
        assert!(!result.is_empty());
        // Strong trend should produce ADX > 25 (typical threshold)
        let last_adx = *result.latest().unwrap();
        assert!(last_adx > 25.0, "ADX for strong trend should be > 25, got {}", last_adx);
    }

    #[test]
    fn adx_no_trend() {
        // Flat market: all bars same price => no directional movement
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..50 {
            bars.push(Bar {
                timestamp: i * 60000,
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 1000,
            });
        }
        let adx = Adx { period: 14 };
        let result = adx.calculate(&bars);
        assert!(!result.is_empty());
        // No directional movement => ADX should be 0
        let last_adx = *result.latest().unwrap();
        assert!((last_adx - 0.0).abs() < 1e-10, "ADX for no trend should be 0, got {}", last_adx);
    }

    #[test]
    fn adx_exact_period() {
        // Minimum viable input: period + 1 bars → may or may not produce output
        // depending on internal accumulation. Use a generous size.
        let bars = make_bars(15); // period + 1 for 14
        let adx = Adx { period: 14 };
        let result = adx.calculate(&bars);
        // With exactly 15 bars we get 14 TR values, which seeds the initial
        // smoothing but we need additional bars to accumulate DX for ADX.
        // Result may be empty — that's valid. Just verify no panic.
        assert!(result.len() <= 1);
    }
}
