use crate::ports::data_provider::{DataError, DataProvider};
use crate::ports::interaction::{InteractionCommand, InteractionHandler};
use fc_primitives::invalidation::InvalidationLevel;
use fc_primitives::kinetic::KineticScroll;
use fc_primitives::series::TimeSeries;

use super::chart_state::ChartState;
use super::data_polling::DataPollingService;
use super::viewport_interaction::ViewportInteractionService;

const CHART_CAPACITY: usize = 100_000;

/// Central orchestrator that owns the data pipeline and interaction handler.
///
/// `ChartController` sits at the application layer: it polls a [`DataProvider`]
/// for new market events and applies user interactions via an
/// [`InteractionHandler`]. Rendering is handled by the app layer
/// (e.g. `GpuRenderer`) which reads [`ChartState`] directly.
pub struct ChartController {
    data_polling: DataPollingService,
    interaction_svc: ViewportInteractionService,
    state: ChartState,
    indicator_overlays: Vec<(String, TimeSeries<f64, CHART_CAPACITY>)>,
    kinetic: KineticScroll,
}

impl ChartController {
    pub fn new(
        data_provider: Box<dyn DataProvider>,
        interaction: Box<dyn InteractionHandler>,
    ) -> Self {
        Self {
            data_polling: DataPollingService::new(data_provider),
            interaction_svc: ViewportInteractionService::new(interaction),
            state: ChartState::new(),
            indicator_overlays: Vec::new(),
            kinetic: KineticScroll::new(0.95),
        }
    }

    /// Poll the data provider for new events, update the time series, and
    /// re-render when new data arrives.
    pub fn tick(&mut self) {
        self.data_polling.poll(&mut self.state);
    }

    /// Forward an interaction command to the handler and apply the resulting
    /// viewport commands to the chart state.
    pub fn handle_input(&mut self, command: InteractionCommand) {
        self.interaction_svc
            .handle_input(command, &mut self.state, &mut self.kinetic);
    }

    pub fn state(&self) -> &ChartState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut ChartState {
        &mut self.state
    }

    pub fn add_indicator_overlay(&mut self, name: String, data: TimeSeries<f64, CHART_CAPACITY>) {
        self.indicator_overlays.push((name, data));
        self.state.mark_dirty(InvalidationLevel::Full);
    }

    pub fn clear_indicator_overlays(&mut self) {
        self.indicator_overlays.clear();
        self.state.mark_dirty(InvalidationLevel::Full);
    }

    pub fn start_data_provider(&mut self) -> Result<(), DataError> {
        self.data_polling.start()
    }

    pub fn stop_data_provider(&mut self) -> Result<(), DataError> {
        self.data_polling.stop()
    }

    /// Update kinetic scroll. Call this each frame.
    /// Returns true if the viewport was displaced (caller should request redraw).
    pub fn update_kinetic(&mut self) -> bool {
        if !self.kinetic.is_active() {
            return false;
        }

        let displacement = self.kinetic.update();
        if displacement.abs() > 0.1 {
            self.state.viewport.pan(displacement as i64);
            self.state.mark_dirty(InvalidationLevel::Full);
            true
        } else {
            self.kinetic.stop();
            false
        }
    }

    /// Stop kinetic scrolling.
    pub fn stop_kinetic(&mut self) {
        self.kinetic.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::data_provider::{DataProvider, DataEvent};
    use crate::ports::interaction::{InteractionHandler, InteractionCommand, ViewportCommand};
    use fc_primitives::bar::Bar;
    use std::cell::RefCell;
    use std::sync::mpsc;

    // --- Mock Data Provider ---
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

    // --- Mock Interaction Handler ---
    struct MockInteractionHandler {
        responses: RefCell<Vec<Vec<ViewportCommand>>>,
    }

    impl MockInteractionHandler {
        fn new() -> Self {
            Self {
                responses: RefCell::new(Vec::new()),
            }
        }

        fn push_response(&self, cmds: Vec<ViewportCommand>) {
            self.responses.borrow_mut().push(cmds);
        }
    }

    impl InteractionHandler for MockInteractionHandler {
        fn handle_event(&self, _command: InteractionCommand) -> Vec<ViewportCommand> {
            self.responses.borrow_mut().pop().unwrap_or_default()
        }
    }

    fn make_controller() -> (ChartController, mpsc::Sender<DataEvent>) {
        let mock_provider = MockDataProvider::new();
        let sender = mock_provider.sender().unwrap();
        let provider = Box::new(mock_provider);
        let handler = Box::new(MockInteractionHandler::new());
        (ChartController::new(provider, handler), sender)
    }

    #[test]
    fn tick_processes_bar_events() {
        let (mut ctrl, tx) = make_controller();
        let bar = Bar::new(1000, 100.0, 105.0, 99.0, 102.0, 5000).unwrap();
        tx.send(DataEvent::BarClosed(bar)).unwrap();

        ctrl.tick();
        assert_eq!(ctrl.state().time_series.len(), 1);
        assert!(ctrl.state().time_series.latest().unwrap().timestamp == 1000);
    }

    #[test]
    fn tick_renders_when_dirty() {
        let (mut ctrl, tx) = make_controller();
        let bar = Bar::new(1000, 100.0, 105.0, 99.0, 102.0, 5000).unwrap();
        tx.send(DataEvent::BarClosed(bar)).unwrap();
        ctrl.tick();
        // After tick, invalidation stays set — the renderer consumes it later.
        assert!(ctrl.state().invalidation.level() > InvalidationLevel::Nothing);
    }

    #[test]
    fn tick_skips_render_when_no_data() {
        let (mut ctrl, _tx) = make_controller();
        ctrl.tick();
        // No data → no render
        assert_eq!(ctrl.state().time_series.len(), 0);
        assert!(ctrl.state().invalidation.is_empty());
    }

    #[test]
    fn tick_processes_multiple_events() {
        let (mut ctrl, tx) = make_controller();
        for i in 0..5 {
            let bar = Bar::new(i, 100.0, 105.0, 99.0, 102.0, 1000).unwrap();
            tx.send(DataEvent::BarClosed(bar)).unwrap();
        }
        ctrl.tick();
        assert_eq!(ctrl.state().time_series.len(), 5);
    }

    #[test]
    fn handle_input_sets_crosshair() {
        let handler = Box::new(MockInteractionHandler::new());
        handler.push_response(vec![ViewportCommand::SetCrosshairPosition {
            x: 400.0,
            y: 250.0,
            time: 5000,
            price: 105.0,
        }]);
        let mock_provider = MockDataProvider::new();
        let provider = Box::new(mock_provider);
        let mut ctrl = ChartController::new(provider, handler);

        ctrl.handle_input(InteractionCommand::UpdateCrosshair {
            screen_x: 400.0,
            screen_y: 250.0,
        });

        assert!(ctrl.state().crosshair.active);
        assert_eq!(ctrl.state().crosshair.screen_x, 400.0);
        assert_eq!(ctrl.state().crosshair.time, 5000);
    }

    #[test]
    fn handle_input_deactivates_crosshair() {
        let handler = Box::new(MockInteractionHandler::new());
        handler.push_response(vec![ViewportCommand::DeactivateCrosshair]);
        let mock_provider = MockDataProvider::new();
        let provider = Box::new(mock_provider);
        let mut ctrl = ChartController::new(provider, handler);

        // First activate
        ctrl.state.crosshair.active = true;
        ctrl.handle_input(InteractionCommand::DeactivateCrosshair);
        assert!(!ctrl.state().crosshair.active);
    }

    #[test]
    fn handle_input_updates_viewport() {
        let handler = Box::new(MockInteractionHandler::new());
        handler.push_response(vec![ViewportCommand::SetTimeRange {
            start: 5000,
            end: 10000,
        }]);
        let mock_provider = MockDataProvider::new();
        let provider = Box::new(mock_provider);
        let mut ctrl = ChartController::new(provider, handler);

        ctrl.handle_input(InteractionCommand::PanBy { time_delta: 5000 });
        assert_eq!(ctrl.state().viewport.time_start, 5000);
        assert_eq!(ctrl.state().viewport.time_end, 10000);
    }

    #[test]
    fn data_provider_start_stop() {
        let (mut ctrl, _tx) = make_controller();
        assert!(ctrl.start_data_provider().is_ok());
        assert!(ctrl.stop_data_provider().is_ok());
    }

    #[test]
    fn add_and_clear_indicator_overlays() {
        let (mut ctrl, _tx) = make_controller();
        let overlay: TimeSeries<f64, CHART_CAPACITY> = TimeSeries::new();
        ctrl.add_indicator_overlay("SMA(14)".to_string(), overlay);
        assert_eq!(ctrl.indicator_overlays.len(), 1);
        ctrl.clear_indicator_overlays();
        assert!(ctrl.indicator_overlays.is_empty());
    }
}
