use crate::bar::Bar;
use crate::series::TimeSeries;

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
