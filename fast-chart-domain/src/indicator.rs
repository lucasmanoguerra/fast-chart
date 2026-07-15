use crate::bar::Bar;
use crate::series::TimeSeries;

/// Trait for technical indicators that compute derived values from price series.
///
/// # Examples
///
/// ```
/// use fast_chart_domain::Indicator;
/// use fast_chart_domain::Bar;
/// use fast_chart_domain::series::TimeSeries;
///
/// struct SimpleMovingAverage { period: usize }
///
/// impl Indicator<100> for SimpleMovingAverage {
///     fn calculate(&self, series: &TimeSeries<Bar, 100>) -> TimeSeries<f64, 100> {
///         let mut result = TimeSeries::new();
///         // ... SMA calculation logic
///         result
///     }
///     fn name(&self) -> &str { "SMA" }
/// }
///
/// let sma = SimpleMovingAverage { period: 14 };
/// assert_eq!(sma.name(), "SMA");
/// ```
pub trait Indicator<const N: usize>: Send + Sync {
    fn calculate(&self, series: &TimeSeries<Bar, N>) -> TimeSeries<f64, N>;
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyIndicator;

    impl Indicator<100> for DummyIndicator {
        fn calculate(&self, _series: &TimeSeries<Bar, 100>) -> TimeSeries<f64, 100> {
            TimeSeries::new()
        }

        fn name(&self) -> &str {
            "DummyIndicator"
        }
    }

    #[test]
    fn indicator_name() {
        let ind = DummyIndicator;
        assert_eq!(ind.name(), "DummyIndicator");
    }

    #[test]
    fn indicator_calculate_returns_empty() {
        let ind = DummyIndicator;
        let series: TimeSeries<Bar, 100> = TimeSeries::new();
        let result = ind.calculate(&series);
        assert!(result.is_empty());
    }

    #[test]
    fn trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DummyIndicator>();
    }
}
