use crate::ports::interaction::{InteractionCommand, InteractionHandler, ViewportCommand};
use super::chart_state::ChartState;
use fc_primitives::invalidation::InvalidationLevel;
use fc_primitives::kinetic::KineticScroll;

/// Service that processes user interaction commands and applies the
/// resulting viewport mutations. Extracted from
/// [`ChartController`](super::chart_controller::ChartController) to
/// respect the Single Responsibility Principle.
pub(crate) struct ViewportInteractionService {
    interaction: Box<dyn InteractionHandler>,
    kinetic: KineticScroll,
}

impl ViewportInteractionService {
    pub(crate) fn new(interaction: Box<dyn InteractionHandler>) -> Self {
        Self {
            interaction,
            kinetic: KineticScroll::new(0.95),
        }
    }

    /// Forward an interaction command to the handler and apply the
    /// resulting viewport commands to chart state.
    pub(crate) fn handle_input(&self, command: InteractionCommand, state: &mut ChartState, kinetic: &mut KineticScroll) {
        log::trace!("handle_input: processing command");
        let commands = self.interaction.handle_event(command);

        for cmd in commands {
            match cmd {
                ViewportCommand::SetTimeRange { start, end } => {
                    state.viewport.time_start = start;
                    state.viewport.time_end = end;
                    state.mark_dirty(InvalidationLevel::Full);
                }
                ViewportCommand::SetValueRange { min, max } => {
                    state.viewport.value_min = min;
                    state.viewport.value_max = max;
                    state.mark_dirty(InvalidationLevel::Full);
                }
                ViewportCommand::SetCrosshairPosition {
                    x,
                    y,
                    time,
                    price,
                } => {
                    state.crosshair.screen_x = x;
                    state.crosshair.screen_y = y;
                    state.crosshair.time = time;
                    state.crosshair.price = price;
                    state.crosshair.active = true;
                    state.mark_dirty(InvalidationLevel::Cursor);
                }
                ViewportCommand::DeactivateCrosshair => {
                    state.crosshair.deactivate();
                    state.mark_dirty(InvalidationLevel::Cursor);
                }
                ViewportCommand::RequestRedraw => {
                    state.mark_dirty(InvalidationLevel::Full);
                }
                ViewportCommand::ZoomAtCursor { factor, screen_x } => {
                    kinetic.stop();
                    state.viewport.zoom(factor, screen_x);
                    state.mark_dirty(InvalidationLevel::Full);
                }
                ViewportCommand::PanBy { time_delta } => {
                    kinetic.start(time_delta as f64);
                    state.viewport.pan(time_delta);
                    state.mark_dirty(InvalidationLevel::Full);
                }
            }
        }
    }

    /// Access the kinetic scroll state.
    pub(crate) fn kinetic(&self) -> &KineticScroll {
        &self.kinetic
    }

    /// Access the kinetic scroll state mutably.
    pub(crate) fn kinetic_mut(&mut self) -> &mut KineticScroll {
        &mut self.kinetic
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::interaction::{InteractionCommand, InteractionHandler, ViewportCommand};
    use super::super::chart_state::ChartState;
    use std::cell::RefCell;

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

    #[test]
    fn handle_input_sets_crosshair() {
        let handler = Box::new(MockInteractionHandler::new());
        handler.push_response(vec![ViewportCommand::SetCrosshairPosition {
            x: 400.0,
            y: 250.0,
            time: 5000,
            price: 105.0,
        }]);
        let svc = ViewportInteractionService::new(handler);
        let mut state = ChartState::new();
        let mut kinetic = KineticScroll::new(0.95);

        svc.handle_input(
            InteractionCommand::UpdateCrosshair {
                screen_x: 400.0,
                screen_y: 250.0,
            },
            &mut state,
            &mut kinetic,
        );

        assert!(state.crosshair.active);
        assert_eq!(state.crosshair.screen_x, 400.0);
        assert_eq!(state.crosshair.time, 5000);
    }

    #[test]
    fn handle_input_updates_viewport() {
        let handler = Box::new(MockInteractionHandler::new());
        handler.push_response(vec![ViewportCommand::SetTimeRange {
            start: 5000,
            end: 10000,
        }]);
        let svc = ViewportInteractionService::new(handler);
        let mut state = ChartState::new();
        let mut kinetic = KineticScroll::new(0.95);

        svc.handle_input(
            InteractionCommand::PanBy { time_delta: 5000 },
            &mut state,
            &mut kinetic,
        );

        assert_eq!(state.viewport.time_start, 5000);
        assert_eq!(state.viewport.time_end, 10000);
    }

    #[test]
    fn handle_input_zoom_at_cursor() {
        let handler = Box::new(MockInteractionHandler::new());
        handler.push_response(vec![ViewportCommand::ZoomAtCursor {
            factor: 2.0,
            screen_x: 400.0,
        }]);
        let svc = ViewportInteractionService::new(handler);
        let mut state = ChartState::new();
        let mut kinetic = KineticScroll::new(0.95);

        svc.handle_input(
            InteractionCommand::PanBy { time_delta: 500 },
            &mut state,
            &mut kinetic,
        );

        assert!(state.invalidation.level() > InvalidationLevel::Nothing);
    }
}
