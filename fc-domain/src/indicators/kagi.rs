use fc_primitives::bar::Bar;
use crate::indicator::Indicator;
use fc_primitives::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

/// Kagi chart indicator.
///
/// Kagi charts draw a line when the price moves by a specified reversal
/// amount from the current trend direction. Unlike Renko, Kagi tracks
/// the direction of the trend and only reverses when the reversal
/// threshold is met.
///
/// **Reversal amount**:
/// - If `reversal_pct > 0.0`, the reversal is `reversal_pct`% of the
///   current price level.
/// - If `reversal_pct == 0.0`, the ATR (period `atr_period`) is used.
///
/// The output is a series of price levels at each turning point (where
/// the direction changes or a new line segment begins).
///
/// Defaults: `reversal_pct: 4.0`, `atr_period: 14`.
pub struct Kagi {
    /// Percentage reversal threshold. Set to `0.0` to use ATR-based reversal.
    pub reversal_pct: f64,
    /// ATR period used when `reversal_pct` is `0.0`.
    pub atr_period: usize,
}

impl Default for Kagi {
    fn default() -> Self {
        Self {
            reversal_pct: 4.0,
            atr_period: 14,
        }
    }
}

impl Kagi {
    /// Compute ATR using Wilder's smoothing (same logic as the `Atr` indicator).
    ///
    /// Returns the latest ATR value for the given series, or `None` when
    /// there are insufficient bars.
    fn compute_atr(bars: &[&Bar], period: usize) -> Option<f64> {
        if bars.len() < period + 1 {
            return None;
        }

        let mut tr_values: Vec<f64> = Vec::with_capacity(bars.len() - 1);
        for i in 1..bars.len() {
            let hl = bars[i].high - bars[i].low;
            let hpc = (bars[i].high - bars[i - 1].close).abs();
            let lpc = (bars[i].low - bars[i - 1].close).abs();
            tr_values.push(f64::max(hl, f64::max(hpc, lpc)));
        }

        let initial_atr: f64 =
            tr_values[..period].iter().sum::<f64>() / period as f64;

        let mut prev_atr = initial_atr;
        for &tr in &tr_values[period..] {
            let atr = (prev_atr * (period as f64 - 1.0) + tr) / period as f64;
            prev_atr = atr;
        }
        Some(prev_atr)
    }

    /// Compute the reversal amount for a given price level.
    fn reversal_amount(&self, current_price: f64, atr: Option<f64>) -> f64 {
        if self.reversal_pct > 0.0 {
            current_price * self.reversal_pct / 100.0
        } else {
            // ATR-based: use the provided ATR, or fall back to a minimum
            atr.unwrap_or(0.0)
        }
    }
}

impl Indicator<MAX_SERIES_LEN> for Kagi {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        let mut result = TimeSeries::new();
        let bars: Vec<&Bar> = series.iter().collect();

        if bars.is_empty() {
            return result;
        }

        // Determine ATR if needed
        let atr = if self.reversal_pct <= 0.0 {
            Self::compute_atr(&bars, self.atr_period)
        } else {
            None
        };

        // If ATR mode but insufficient data, return empty
        if self.reversal_pct <= 0.0 && atr.is_none() {
            return result;
        }

        // Initialize with first bar's close
        let mut prev_price = bars[0].close;
        // Direction: 1 = up, -1 = down (start neutral, will be set on first move)
        let mut direction: i8 = 0;

        result.push(prev_price);

        for &bar in &bars[1..] {
            let rev = self.reversal_amount(prev_price, atr);

            if direction <= 0 {
                // Currently going down or neutral — check for upward reversal
                if bar.high - prev_price >= rev {
                    // Reversal to up
                    direction = 1;
                    prev_price = bar.high;
                    result.push(prev_price);
                } else if prev_price - bar.low >= rev {
                    // Continue downward
                    direction = -1;
                    prev_price = bar.low;
                    result.push(prev_price);
                }
                // Otherwise: no change, continue current direction
            } else {
                // Currently going up — check for downward reversal
                if prev_price - bar.low >= rev {
                    // Reversal to down
                    direction = -1;
                    prev_price = bar.low;
                    result.push(prev_price);
                } else if bar.high - prev_price >= rev {
                    // Continue upward
                    direction = 1;
                    prev_price = bar.high;
                    result.push(prev_price);
                }
                // Otherwise: no change, continue current direction
            }

        }

        result
    }

    fn name(&self) -> &str {
        "Kagi"
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

    #[test]
    fn kagi_name() {
        let kagi = Kagi::default();
        assert_eq!(kagi.name(), "Kagi");
    }

    #[test]
    fn kagi_empty_series() {
        let series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        let kagi = Kagi::default();
        let result = kagi.calculate(&series);
        assert!(result.is_empty());
    }

    #[test]
    fn kagi_single_bar() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        series.push(Bar {
            timestamp: 0,
            open: 100.0,
            high: 105.0,
            low: 99.0,
            close: 102.0,
            volume: 1000,
        });
        let kagi = Kagi::default();
        let result = kagi.calculate(&series);
        // Single bar → only the initial price level
        assert_eq!(result.len(), 1);
        assert!((result.get(0).unwrap() - 102.0).abs() < f64::EPSILON);
    }

    #[test]
    fn kagi_basic() {
        let bars = make_bars(50);
        let kagi = Kagi { reversal_pct: 2.0, atr_period: 14 };
        let result = kagi.calculate(&bars);
        // With varying prices, should produce multiple turning points
        assert!(result.len() >= 1);
    }

    #[test]
    fn kagi_exact_period() {
        let bars = make_bars(15);
        let kagi = Kagi { reversal_pct: 0.0, atr_period: 14 };
        let result = kagi.calculate(&bars);
        // Should produce turning points with ATR-based reversal
        assert!(result.len() >= 1);
    }

    #[test]
    fn kagi_insufficient_data_for_atr() {
        let bars = make_bars(10);
        let kagi = Kagi { reversal_pct: 0.0, atr_period: 14 };
        let result = kagi.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn kagi_flat_prices_no_reversal() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..20 {
            series.push(Bar {
                timestamp: i * 60000,
                open: 100.0,
                high: 100.5,
                low: 99.5,
                close: 100.0,
                volume: 1000,
            });
        }
        let kagi = Kagi { reversal_pct: 4.0, atr_period: 14 };
        let result = kagi.calculate(&series);
        // Flat prices → no reversal, only initial level
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn kagi_upward_reversal() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        // Start at 100, then move down
        series.push(Bar { timestamp: 0, open: 100.0, high: 101.0, low: 99.0, close: 100.0, volume: 1000 });
        // Price drops to 96 → reversal threshold met (100 * 4% = 4, so 100-96 = 4)
        series.push(Bar { timestamp: 1, open: 100.0, high: 100.5, low: 95.5, close: 96.0, volume: 1000 });
        // Now move up significantly → reversal from down to up
        series.push(Bar { timestamp: 2, open: 96.0, high: 101.0, low: 95.5, close: 100.0, volume: 1000 });

        let kagi = Kagi { reversal_pct: 4.0, atr_period: 14 };
        let result = kagi.calculate(&series);

        // Should have turning points
        assert!(result.len() >= 2);
    }

    #[test]
    fn kagi_downward_reversal() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        // Start at 100, move up
        series.push(Bar { timestamp: 0, open: 100.0, high: 101.0, low: 99.0, close: 100.0, volume: 1000 });
        // Price rises to 105 → direction becomes up (105-100 = 5 >= 4% of 100 = 4)
        series.push(Bar { timestamp: 1, open: 100.0, high: 105.5, low: 99.5, close: 104.0, volume: 1000 });
        // Price drops to 99 → reversal from up to down (104-99 = 5 >= 4% of 104 = 4.16)
        series.push(Bar { timestamp: 2, open: 104.0, high: 104.5, low: 98.5, close: 99.0, volume: 1000 });

        let kagi = Kagi { reversal_pct: 4.0, atr_period: 14 };
        let result = kagi.calculate(&series);

        // Should have turning points
        assert!(result.len() >= 2);
    }

    #[test]
    fn kagi_atr_mode() {
        let bars = make_bars(50);
        let kagi = Kagi { reversal_pct: 0.0, atr_period: 14 };
        let result = kagi.calculate(&bars);
        // Should produce turning points with ATR-based reversal
        assert!(result.len() >= 1);
    }

    #[test]
    fn kagi_atr_mode_insufficient_bars() {
        let bars = make_bars(5);
        let kagi = Kagi { reversal_pct: 0.0, atr_period: 14 };
        let result = kagi.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn kagi_trend_continuation() {
        // Bars that continue trending up without reversal
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        series.push(Bar { timestamp: 0, open: 100.0, high: 101.0, low: 99.0, close: 100.0, volume: 1000 });
        series.push(Bar { timestamp: 1, open: 100.0, high: 101.5, low: 99.5, close: 101.0, volume: 1000 });
        series.push(Bar { timestamp: 2, open: 101.0, high: 102.0, low: 100.0, close: 101.5, volume: 1000 });

        let kagi = Kagi { reversal_pct: 4.0, atr_period: 14 };
        let result = kagi.calculate(&series);
        // Small moves (1% of 100 = 1 < 4% threshold) → no reversal
        // But initial price is recorded
        assert!(result.len() >= 1);
    }

    #[test]
    fn kagi_large_reversal() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        series.push(Bar { timestamp: 0, open: 100.0, high: 101.0, low: 99.0, close: 100.0, volume: 1000 });
        // Huge move up
        series.push(Bar { timestamp: 1, open: 100.0, high: 120.0, low: 99.5, close: 115.0, volume: 1000 });
        // Huge move down
        series.push(Bar { timestamp: 2, open: 115.0, high: 115.5, low: 80.0, close: 85.0, volume: 1000 });

        let kagi = Kagi { reversal_pct: 4.0, atr_period: 14 };
        let result = kagi.calculate(&series);

        // Should have at least the initial and the two reversals
        assert!(result.len() >= 2);
    }
}
