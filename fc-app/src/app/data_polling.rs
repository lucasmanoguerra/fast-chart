use crate::ports::data_provider::{DataError, DataEvent, DataProvider};
use super::chart_state::ChartState;
use fc_primitives::invalidation::InvalidationLevel;

/// Service that polls a [`DataProvider`] for new market events and
/// pushes them into [`ChartState`]. Extracted from
/// [`ChartController`](super::chart_controller::ChartController) to
/// respect the Single Responsibility Principle.
pub(crate) struct DataPollingService {
    data_provider: Box<dyn DataProvider>,
}

impl DataPollingService {
    pub(crate) fn new(data_provider: Box<dyn DataProvider>) -> Self {
        Self { data_provider }
    }

    /// Poll the data provider for new events and update chart state.
    pub(crate) fn poll(&mut self, state: &mut ChartState) {
        if let Some(rx) = self.data_provider.receiver() {
            while let Ok(event) = rx.try_recv() {
                match event {
                    DataEvent::BarClosed(bar) => {
                        state.time_series.push(bar);
                        state.mark_dirty(InvalidationLevel::Full);
                    }
                    DataEvent::TickUpdate(_tick) => {
                        state.mark_dirty(InvalidationLevel::Full);
                    }
                    DataEvent::SymbolChanged(_) | DataEvent::TimeframeChanged(_) => {
                        state.mark_dirty(InvalidationLevel::Full);
                    }
                }
            }
        }

        // Auto-fit viewport on first data
        if state.invalidation.contains(InvalidationLevel::Full)
            && state.time_series.len() > 0
            && state.viewport.time_start == 0
            && state.viewport.time_end == 3600_000
        {
            if let Some(first) = state.time_series.get(0) {
                if let Some(last) = state.time_series.latest() {
                    state.viewport.time_start = first.timestamp;
                    state.viewport.time_end = last.timestamp;
                    state.viewport.value_min = f64::MIN;
                    state.viewport.value_max = f64::MAX;
                }
            }
        }
    }

    /// Start the underlying data provider.
    pub(crate) fn start(&mut self) -> Result<(), DataError> {
        self.data_provider.start()
    }

    /// Stop the underlying data provider.
    pub(crate) fn stop(&mut self) -> Result<(), DataError> {
        self.data_provider.stop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::data_provider::{DataProvider, DataEvent};
    use super::super::chart_state::ChartState;
    use fc_primitives::bar::Bar;
    use std::sync::mpsc;

    struct MockDataProvider {
        rx: Option<mpsc::Receiver<DataEvent>>,
        tx: Option<mpsc::Sender<DataEvent>>,
        started: bool,
    }

    impl MockDataProvider {
        fn new() -> Self {
            let (tx, rx) = mpsc::channel();
            Self {
                rx: Some(rx),
                tx: Some(tx),
                started: false,
            }
        }

        fn sender(&self) -> Option<mpsc::Sender<DataEvent>> {
            self.tx.clone()
        }
    }

    impl DataProvider for MockDataProvider {
        fn start(&mut self) -> Result<(), DataError> {
            self.started = true;
            Ok(())
        }

        fn receiver(&self) -> Option<&mpsc::Receiver<DataEvent>> {
            self.rx.as_ref()
        }

        fn stop(&mut self) -> Result<(), DataError> {
            self.started = false;
            Ok(())
        }

        fn name(&self) -> &str {
            "MockDataProvider"
        }
    }

    #[test]
    fn poll_processes_bar_closed() {
        let mock = MockDataProvider::new();
        let tx = mock.sender().unwrap();
        let mut svc = DataPollingService::new(Box::new(mock));
        let mut state = ChartState::new();

        let bar = Bar::new(1000, 100.0, 105.0, 99.0, 102.0, 5000).unwrap();
        tx.send(DataEvent::BarClosed(bar)).unwrap();

        svc.poll(&mut state);
        assert_eq!(state.time_series.len(), 1);
        assert!(state.time_series.latest().unwrap().timestamp == 1000);
    }

    #[test]
    fn poll_marks_dirty() {
        let mock = MockDataProvider::new();
        let tx = mock.sender().unwrap();
        let mut svc = DataPollingService::new(Box::new(mock));
        let mut state = ChartState::new();

        let bar = Bar::new(1000, 100.0, 105.0, 99.0, 102.0, 5000).unwrap();
        tx.send(DataEvent::BarClosed(bar)).unwrap();

        svc.poll(&mut state);
        assert!(state.invalidation.level() > InvalidationLevel::Nothing);
    }

    #[test]
    fn poll_no_data_no_dirty() {
        let mock = MockDataProvider::new();
        let mut svc = DataPollingService::new(Box::new(mock));
        let mut state = ChartState::new();

        svc.poll(&mut state);
        assert!(state.invalidation.is_empty());
    }

    #[test]
    fn poll_processes_multiple_events() {
        let mock = MockDataProvider::new();
        let tx = mock.sender().unwrap();
        let mut svc = DataPollingService::new(Box::new(mock));
        let mut state = ChartState::new();

        for i in 0..5 {
            let bar = Bar::new(i, 100.0, 105.0, 99.0, 102.0, 1000).unwrap();
            tx.send(DataEvent::BarClosed(bar)).unwrap();
        }

        svc.poll(&mut state);
        assert_eq!(state.time_series.len(), 5);
    }

    #[test]
    fn start_stop_delegates_to_provider() {
        let mock = MockDataProvider::new();
        let mut svc = DataPollingService::new(Box::new(mock));
        assert!(svc.start().is_ok());
        assert!(svc.stop().is_ok());
    }
}
