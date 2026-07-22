use fc_primitives::bar::Bar;
use crate::indicator::Indicator;
use fc_primitives::series::TimeSeries;
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
        if bars.len() < self.period + 1 {
            return result;
        }

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

        if tr_values.len() < self.period {
            return result;
        }

        let period_f = self.period as f64;
        let mut atr: f64 = tr_values[..self.period].iter().sum::<f64>() / period_f;
        let mut smooth_plus_dm: f64 =
            plus_dm_values[..self.period].iter().sum::<f64>() / period_f;
        let mut smooth_minus_dm: f64 =
            minus_dm_values[..self.period].iter().sum::<f64>() / period_f;

        let (_plus_di, _minus_di, dx) = compute_di_dx(atr, smooth_plus_dm, smooth_minus_dm);

        let mut dx_accumulator = dx;
        let mut dx_seeded = false;
        let mut adx = 0.0;

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
                if idx + 2 >= self.period {
                    adx = dx_accumulator / period_f;
                    result.push(adx);
                    dx_seeded = true;
                }
            } else {
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

    // Clasificación: determinística — verifica adx_name
    #[test]
    fn adx_name() {
        let adx = Adx { period: 14 };
        assert_eq!(adx.name(), "ADX");
    }

    // Clasificación: determinística — verifica adx_insufficient_data
    #[test]
    fn adx_insufficient_data() {
        let bars = make_bars(10);
        let adx = Adx { period: 14 };
        let result = adx.calculate(&bars);
        assert!(result.is_empty());
    }

    // Clasificación: determinística — verifica adx_basic
    #[test]
    fn adx_basic() {
        let bars = make_bars(100);
        let adx = Adx { period: 14 };
        let result = adx.calculate(&bars);
        assert!(!result.is_empty());
    }

    // Clasificación: determinística — verifica adx_bounds
    #[test]
    fn adx_bounds() {
        let bars = make_bars(200);
        let adx = Adx { period: 14 };
        let result = adx.calculate(&bars);
        assert!(result.iter().all(|v| *v >= 0.0 && *v <= 100.0));
    }

    // Clasificación: determinística — verifica adx_strong_trend
    #[test]
    fn adx_strong_trend() {
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
        let last_adx = *result.latest().unwrap();
        assert!(last_adx > 25.0, "ADX for strong trend should be > 25, got {}", last_adx);
    }

    // Clasificación: determinística — verifica adx_no_trend
    #[test]
    fn adx_no_trend() {
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
        let last_adx = *result.latest().unwrap();
        assert!((last_adx - 0.0).abs() < 1e-10, "ADX for no trend should be 0, got {}", last_adx);
    }

    // Clasificación: determinística — verifica adx_exact_period
    #[test]
    fn adx_exact_period() {
        let bars = make_bars(15);
        let adx = Adx { period: 14 };
        let result = adx.calculate(&bars);
        assert!(result.len() <= 1);
    }
}
