use fast_chart_core::ports::interaction::{InteractionCommand, InteractionHandler, ViewportCommand};
use fast_chart_domain::viewport::Viewport;

/// Stateless adapter mapping winit window events to `InteractionCommand`s.
pub struct WinitInteractionHandler;

impl WinitInteractionHandler {
    pub fn new() -> Self {
        Self
    }
}

impl InteractionHandler for WinitInteractionHandler {
    fn handle_event(&self, command: InteractionCommand) -> Vec<ViewportCommand> {
        match command {
            InteractionCommand::ZoomAtCursor { factor, screen_x } => {
                vec![ViewportCommand::ZoomAtCursor { factor, screen_x }]
            }
            InteractionCommand::PanBy { time_delta } => {
                vec![ViewportCommand::PanBy { time_delta }]
            }
            InteractionCommand::UpdateCrosshair { screen_x, screen_y } => {
                vec![ViewportCommand::SetCrosshairPosition {
                    x: screen_x,
                    y: screen_y,
                    time: 0,
                    price: 0.0,
                }]
            }
            InteractionCommand::DeactivateCrosshair => {
                vec![ViewportCommand::DeactivateCrosshair]
            }
            _ => vec![],
        }
    }
}

impl WinitInteractionHandler {
    /// Map a keyboard key string to a timeframe `InteractionCommand`.
    ///
    /// Returns `Some(InteractionCommand::SwitchTimeframe { .. })` for recognized keys,
    /// `None` for anything else.
    pub fn handle_key(key: &str) -> Option<InteractionCommand> {
        match key {
            "1" => Some(InteractionCommand::SwitchTimeframe {
                timeframe: "1m".to_string(),
            }),
            "5" => Some(InteractionCommand::SwitchTimeframe {
                timeframe: "5m".to_string(),
            }),
            "15" => Some(InteractionCommand::SwitchTimeframe {
                timeframe: "15m".to_string(),
            }),
            "6" => Some(InteractionCommand::SwitchTimeframe {
                timeframe: "1h".to_string(),
            }),
            "d" | "D" => Some(InteractionCommand::SwitchTimeframe {
                timeframe: "1d".to_string(),
            }),
            _ => None,
        }
    }
}

impl Default for WinitInteractionHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Map a winit `WindowEvent` to an `InteractionCommand` for the current viewport.
///
/// Returns `None` when the event has no chart interaction meaning.
pub fn winit_event_to_command(
    event: &winit::event::WindowEvent,
    screen_x: f32,
    _viewport: &Viewport,
) -> Option<InteractionCommand> {
    use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};

    match event {
        WindowEvent::CursorMoved { position, .. } => {
            Some(InteractionCommand::UpdateCrosshair {
                screen_x: position.x as f64,
                screen_y: position.y as f64,
            })
        }
        WindowEvent::MouseWheel {
            delta: MouseScrollDelta::LineDelta(_, dy),
            ..
        } => {
            let factor = if *dy > 0.0 { 1.1 } else { 0.9 };
            Some(InteractionCommand::ZoomAtCursor {
                factor,
                screen_x: screen_x as f64,
            })
        }
        WindowEvent::MouseInput {
            state: ElementState::Released,
            button: MouseButton::Left,
            ..
        } => Some(InteractionCommand::PanBy { time_delta: 0 }),
        WindowEvent::CursorLeft { .. } => Some(InteractionCommand::DeactivateCrosshair),
        _ => None,
    }
}
