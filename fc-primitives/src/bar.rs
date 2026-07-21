use crate::error::ChartError;

/// An OHLCV (Open, High, Low, Close, Volume) price bar.
///
/// Represents a single time period in a financial time series.
/// All prices must be non-negative, `high >= low`, and both `open` and
/// `close` must lie within `[low, high]`.
///
/// # Examples
///
/// ```
/// use fc_primitives::Bar;
///
/// let bar = Bar::new(1000, 100.0, 105.0, 99.0, 102.0, 5000).unwrap();
/// assert_eq!(bar.open, 100.0);
/// assert_eq!(bar.close, 102.0);
/// assert!(bar.is_bullish());  // close > open
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bar {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

impl Bar {
    pub fn new(
        timestamp: u64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: u64,
    ) -> Result<Self, ChartError> {
        if open < 0.0 || high < 0.0 || low < 0.0 || close < 0.0 {
            return Err(ChartError::InvalidPriceData(
                "prices must be non-negative".into(),
            ));
        }
        if high < low {
            return Err(ChartError::InvalidPriceData(
                "high must be >= low".into(),
            ));
        }
        if open < low || open > high {
            return Err(ChartError::InvalidPriceData(
                "open must be within [low, high]".into(),
            ));
        }
        if close < low || close > high {
            return Err(ChartError::InvalidPriceData(
                "close must be within [low, high]".into(),
            ));
        }
        Ok(Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        })
    }

    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    pub fn body(&self) -> f64 {
        (self.close - self.open).abs()
    }

    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    pub fn midpoint(&self) -> f64 {
        (self.high + self.low) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_bar() -> Bar {
        Bar::new(1000, 100.0, 105.0, 99.5, 102.0, 15000).unwrap()
    }

    #[test]
    fn valid_construction() {
        let bar = valid_bar();
        assert_eq!(bar.timestamp, 1000);
        assert_eq!(bar.open, 100.0);
        assert_eq!(bar.high, 105.0);
        assert_eq!(bar.low, 99.5);
        assert_eq!(bar.close, 102.0);
        assert_eq!(bar.volume, 15000);
    }

    #[test]
    fn reject_high_less_than_low() {
        let result = Bar::new(1, 100.0, 90.0, 95.0, 100.0, 100);
        assert!(matches!(result, Err(ChartError::InvalidPriceData(_))));
    }

    #[test]
    fn reject_open_below_low() {
        let result = Bar::new(1, 90.0, 105.0, 95.0, 100.0, 100);
        assert!(matches!(result, Err(ChartError::InvalidPriceData(_))));
    }

    #[test]
    fn reject_close_above_high() {
        let result = Bar::new(1, 100.0, 105.0, 99.0, 110.0, 100);
        assert!(matches!(result, Err(ChartError::InvalidPriceData(_))));
    }

    #[test]
    fn reject_negative_price() {
        let result = Bar::new(1, -1.0, 10.0, 0.0, 5.0, 100);
        assert!(matches!(result, Err(ChartError::InvalidPriceData(_))));
    }

    #[test]
    fn bullish_bar() {
        let bar = Bar::new(1, 100.0, 105.0, 99.0, 102.0, 100).unwrap();
        assert!(bar.is_bullish());
    }

    #[test]
    fn bearish_bar() {
        let bar = Bar::new(1, 102.0, 105.0, 99.0, 100.0, 100).unwrap();
        assert!(!bar.is_bullish());
    }

    #[test]
    fn body() {
        let bar = Bar::new(1, 100.0, 105.0, 99.0, 103.0, 100).unwrap();
        assert_eq!(bar.body(), 3.0);
    }

    #[test]
    fn body_bearish() {
        let bar = Bar::new(1, 103.0, 105.0, 99.0, 100.0, 100).unwrap();
        assert_eq!(bar.body(), 3.0);
    }

    #[test]
    fn range() {
        let bar = valid_bar();
        assert_eq!(bar.range(), 5.5);
    }

    #[test]
    fn midpoint() {
        let bar = valid_bar();
        assert_eq!(bar.midpoint(), 102.25);
    }

    #[test]
    fn doji_bar() {
        let bar = Bar::new(1, 100.0, 105.0, 99.0, 100.0, 100).unwrap();
        assert!(!bar.is_bullish());
        assert_eq!(bar.body(), 0.0);
    }
}
