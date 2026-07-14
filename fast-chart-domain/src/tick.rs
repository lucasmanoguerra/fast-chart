use crate::error::ChartError;

/// A single tick (bid/ask/last) snapshot from a live market feed.
///
/// Unlike [`Bar`](crate::Bar), a tick represents an instantaneous price
/// point rather than an aggregated time period.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tick {
    pub timestamp: u64,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: u64,
}

impl Tick {
    pub fn new(
        timestamp: u64,
        bid: f64,
        ask: f64,
        last: f64,
        volume: u64,
    ) -> Result<Self, ChartError> {
        if bid < 0.0 || ask < 0.0 || last < 0.0 {
            return Err(ChartError::InvalidPriceData(
                "prices must be non-negative".into(),
            ));
        }
        if ask < bid {
            return Err(ChartError::InvalidPriceData(
                "ask must be >= bid".into(),
            ));
        }
        Ok(Self {
            timestamp,
            bid,
            ask,
            last,
            volume,
        })
    }

    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_construction() {
        let tick = Tick::new(1000, 100.0, 100.05, 100.02, 500).unwrap();
        assert_eq!(tick.timestamp, 1000);
        assert_eq!(tick.bid, 100.0);
        assert_eq!(tick.ask, 100.05);
        assert_eq!(tick.last, 100.02);
        assert_eq!(tick.volume, 500);
    }

    #[test]
    fn zero_volume_succeeds() {
        let tick = Tick::new(1, 10.0, 10.5, 10.2, 0).unwrap();
        assert_eq!(tick.volume, 0);
    }

    #[test]
    fn spread() {
        let tick = Tick::new(1, 100.0, 100.05, 100.02, 100).unwrap();
        assert!((tick.spread() - 0.05).abs() < 1e-10);
    }

    #[test]
    fn zero_spread() {
        let tick = Tick::new(1, 100.0, 100.0, 100.0, 100).unwrap();
        assert_eq!(tick.spread(), 0.0);
    }

    #[test]
    fn reject_ask_less_than_bid() {
        let result = Tick::new(1, 100.05, 100.0, 100.02, 100);
        assert!(matches!(result, Err(ChartError::InvalidPriceData(_))));
    }

    #[test]
    fn reject_negative_price() {
        let result = Tick::new(1, -1.0, 10.0, 5.0, 100);
        assert!(matches!(result, Err(ChartError::InvalidPriceData(_))));
    }
}
