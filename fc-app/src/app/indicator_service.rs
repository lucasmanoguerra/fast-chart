use fc_primitives::bar::Bar;
use fc_domain::indicator::Indicator;
use fc_primitives::series::TimeSeries;
use std::collections::HashMap;

const CHART_CAPACITY: usize = 100_000;

pub struct IndicatorRegistry {
    indicators: HashMap<String, Box<dyn Indicator<CHART_CAPACITY>>>,
}

impl IndicatorRegistry {
    pub fn new() -> Self {
        Self {
            indicators: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, indicator: Box<dyn Indicator<CHART_CAPACITY>>) {
        self.indicators.insert(name, indicator);
    }

    pub fn calculate_all(
        &self,
        source: &TimeSeries<Bar, CHART_CAPACITY>,
    ) -> HashMap<String, TimeSeries<f64, CHART_CAPACITY>> {
        self.indicators
            .iter()
            .map(|(name, ind)| (name.clone(), ind.calculate(source)))
            .collect()
    }

    pub fn get(&self, name: &str) -> Option<&dyn Indicator<CHART_CAPACITY>> {
        self.indicators.get(name).map(|b| b.as_ref())
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.indicators.remove(name).is_some()
    }

    pub fn names(&self) -> Vec<&str> {
        self.indicators.keys().map(|s| s.as_str()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.indicators.is_empty()
    }

    pub fn len(&self) -> usize {
        self.indicators.len()
    }
}

impl Default for IndicatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct MockIndicator {
        name: String,
    }

    impl MockIndicator {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl Indicator<CHART_CAPACITY> for MockIndicator {
        fn calculate(&self, _series: &TimeSeries<Bar, CHART_CAPACITY>) -> TimeSeries<f64, CHART_CAPACITY> {
            let mut result: TimeSeries<f64, CHART_CAPACITY> = TimeSeries::new();
            // Push a single value to indicate this indicator ran
            result.push(42.0);
            result
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    struct ZeroIndicator;

    impl Indicator<CHART_CAPACITY> for ZeroIndicator {
        fn calculate(&self, _series: &TimeSeries<Bar, CHART_CAPACITY>) -> TimeSeries<f64, CHART_CAPACITY> {
            TimeSeries::new()
        }

        fn name(&self) -> &str {
            "ZeroIndicator"
        }
    }

    fn make_bar(timestamp: u64) -> Bar {
        Bar::new(timestamp, 100.0, 105.0, 99.0, 102.0, 1000).unwrap()
    }

    #[test]
    fn empty_registry() {
        let reg = IndicatorRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn register_and_get() {
        let mut reg = IndicatorRegistry::new();
        reg.register(
            "sma14".to_string(),
            Box::new(MockIndicator::new("SMA(14)")),
        );
        assert_eq!(reg.len(), 1);
        assert!(!reg.is_empty());

        let ind = reg.get("sma14").unwrap();
        assert_eq!(ind.name(), "SMA(14)");
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let reg = IndicatorRegistry::new();
        assert!(reg.get("unknown").is_none());
    }

    #[test]
    fn register_overwrites_existing() {
        let mut reg = IndicatorRegistry::new();
        reg.register(
            "sma".to_string(),
            Box::new(MockIndicator::new("SMA(10)")),
        );
        reg.register(
            "sma".to_string(),
            Box::new(MockIndicator::new("SMA(20)")),
        );
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.get("sma").unwrap().name(), "SMA(20)");
    }

    #[test]
    fn remove_existing() {
        let mut reg = IndicatorRegistry::new();
        reg.register(
            "rsi".to_string(),
            Box::new(MockIndicator::new("RSI(14)")),
        );
        assert!(reg.remove("rsi"));
        assert!(reg.is_empty());
    }

    #[test]
    fn remove_nonexistent() {
        let mut reg = IndicatorRegistry::new();
        assert!(!reg.remove("unknown"));
    }

    #[test]
    fn calculate_all() {
        let mut reg = IndicatorRegistry::new();
        reg.register(
            "sma".to_string(),
            Box::new(MockIndicator::new("SMA(14)")),
        );
        reg.register(
            "rsi".to_string(),
            Box::new(MockIndicator::new("RSI(14)")),
        );

        let mut series: TimeSeries<Bar, CHART_CAPACITY> = TimeSeries::new();
        series.push(make_bar(1000));

        let results = reg.calculate_all(&series);
        assert_eq!(results.len(), 2);
        assert!(results.contains_key("sma"));
        assert!(results.contains_key("rsi"));
        // Both mock indicators push 42.0
        assert_eq!(results["sma"].latest(), Some(&42.0));
        assert_eq!(results["rsi"].latest(), Some(&42.0));
    }

    #[test]
    fn calculate_all_empty_registry() {
        let reg = IndicatorRegistry::new();
        let series: TimeSeries<Bar, CHART_CAPACITY> = TimeSeries::new();
        let results = reg.calculate_all(&series);
        assert!(results.is_empty());
    }

    #[test]
    fn names_returns_all() {
        let mut reg = IndicatorRegistry::new();
        reg.register(
            "a".to_string(),
            Box::new(MockIndicator::new("A")),
        );
        reg.register(
            "b".to_string(),
            Box::new(MockIndicator::new("B")),
        );
        let mut names = reg.names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn default_is_empty() {
        let reg = IndicatorRegistry::default();
        assert!(reg.is_empty());
    }

    #[test]
    fn register_zero_indicator_and_calculate() {
        let mut reg = IndicatorRegistry::new();
        reg.register("zero".to_string(), Box::new(ZeroIndicator));
        let series: TimeSeries<Bar, CHART_CAPACITY> = TimeSeries::new();
        let results = reg.calculate_all(&series);
        assert_eq!(results["zero"].len(), 0);
    }
}
