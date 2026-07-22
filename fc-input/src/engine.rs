//! Interaction engine — state machine that converts raw [`InputEvent`]s
//! into high-level [`ChartCommand`]s.
//!
//! The engine tracks mouse/keyboard state and produces a stream of commands
//! that a chart controller executes. It is deliberately decoupled from any
//! rendering or input data layer.

use crate::{
    InputEvent, KeyCode, KeyEvent, MouseButton, MouseButtonEvent, MouseMoveEvent,
    ModifierState, PinchEvent, TouchEvent, WheelEvent,
};

// ---------------------------------------------------------------------------
// DrawingTool
// ---------------------------------------------------------------------------

/// Available drawing tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingTool {
    Select,
    TrendLine,
    Arrow,
    Ray,
    Segment,
    Rectangle,
    Ellipse,
    HorizontalLine,
    VerticalLine,
    FibonacciRetracement,
    FibonacciExtension,
    Pitchfork,
    Path,
    Text,
    Image,
    Label,
}

// ---------------------------------------------------------------------------
// ChartCommand
// ---------------------------------------------------------------------------

/// High-level chart commands produced by the [`InteractionEngine`].
///
/// The host application or chart controller receives these and applies them.
#[derive(Debug, Clone, PartialEq)]
pub enum ChartCommand {
    // Viewport
    ZoomAtCursor {
        factor: f64,
        screen_x: f64,
        screen_y: f64,
    },
    ZoomReset,
    Pan {
        time_delta: i64,
        price_delta: f64,
    },
    PanReset,
    SetViewport {
        time_start: u64,
        time_end: u64,
        price_min: f64,
        price_max: f64,
    },

    // Crosshair
    UpdateCrosshair {
        screen_x: f64,
        screen_y: f64,
    },
    DeactivateCrosshair,

    // Selection
    SelectDrawing {
        screen_x: f64,
        screen_y: f64,
    },
    DeselectAll,
    DeleteSelected,
    MoveSelected {
        delta_timestamp: u64,
        delta_price: f64,
    },

    // Drawing
    StartDrawing {
        tool: DrawingTool,
    },
    CancelDrawing,
    PlaceDrawingPoint {
        screen_x: f64,
        screen_y: f64,
    },

    // Timeframe
    SwitchTimeframe {
        timeframe: String,
    },

    // Request
    RequestRedraw,
}

// ---------------------------------------------------------------------------
// InteractionMode
// ---------------------------------------------------------------------------

/// Internal interaction mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InteractionMode {
    Navigate,
    Draw,
}

// ---------------------------------------------------------------------------
// InteractionEngine
// ---------------------------------------------------------------------------

/// State machine that consumes [`InputEvent`]s and produces [`ChartCommand`]s.
pub struct InteractionEngine {
    /// Current interaction mode.
    mode: InteractionMode,
    /// Whether the mouse is pressed (dragging).
    mouse_down: bool,
    /// Last mouse position.
    last_mouse: Option<(f64, f64)>,
    /// Drag start position.
    drag_start: Option<(f64, f64)>,
    /// Active drawing tool (if any).
    active_tool: Option<DrawingTool>,
    /// Whether Shift is held (axis-locked zoom/pan).
    shift_held: bool,
    /// Whether Ctrl is held (modifier combos).
    ctrl_held: bool,
}

impl InteractionEngine {
    /// Create a new engine in Navigate mode.
    pub fn new() -> Self {
        Self {
            mode: InteractionMode::Navigate,
            mouse_down: false,
            last_mouse: None,
            drag_start: None,
            active_tool: None,
            shift_held: false,
            ctrl_held: false,
        }
    }

    /// Set the active drawing tool directly.
    pub fn set_tool(&mut self, tool: Option<DrawingTool>) {
        self.active_tool = tool;
        self.mode = match tool {
            Some(_) => InteractionMode::Draw,
            None => InteractionMode::Navigate,
        };
    }

    /// Get the current active tool.
    pub fn active_tool(&self) -> Option<DrawingTool> {
        self.active_tool
    }

    /// Check if currently in drawing mode.
    pub fn is_drawing(&self) -> bool {
        self.mode == InteractionMode::Draw
    }

    /// Process an input event and return chart commands.
    pub fn handle(&mut self, event: InputEvent) -> Vec<ChartCommand> {
        match event {
            InputEvent::MouseMove(evt) => self.handle_mouse_move(evt),
            InputEvent::MouseDown(evt) => self.handle_mouse_down(evt),
            InputEvent::MouseUp(evt) => self.handle_mouse_up(evt),
            InputEvent::Wheel(evt) => self.handle_wheel(evt),
            InputEvent::MouseDoubleClick(_) => Vec::new(),
            InputEvent::TouchStart(evt) => self.handle_touch_start(evt),
            InputEvent::TouchMove(evt) => self.handle_touch_move(evt),
            InputEvent::TouchEnd(evt) => self.handle_touch_end(evt),
            InputEvent::Pinch(evt) => self.handle_pinch(evt),
            InputEvent::KeyDown(evt) => self.handle_key_down(evt),
            InputEvent::KeyUp(evt) => self.handle_key_up(evt),
            InputEvent::StylusMove(evt) => {
                self.handle_mouse_move(MouseMoveEvent {
                    x: evt.x, y: evt.y,
                    modifiers: evt.modifiers,
                })
            }
            InputEvent::StylusDown(evt) => {
                self.handle_mouse_down(MouseButtonEvent {
                    button: MouseButton::Left,
                    x: evt.x, y: evt.y,
                    modifiers: evt.modifiers,
                })
            }
            InputEvent::StylusUp(evt) => {
                self.handle_mouse_up(MouseButtonEvent {
                    button: MouseButton::Left,
                    x: evt.x, y: evt.y,
                    modifiers: evt.modifiers,
                })
            }
        }
    }

    // -- private handlers ---------------------------------------------------

    fn handle_mouse_move(&mut self, evt: MouseMoveEvent) -> Vec<ChartCommand> {
        self.last_mouse = Some((evt.x, evt.y));

        if self.mouse_down {
            if let Some(start) = self.drag_start {
                let dx = evt.x - start.0;
                let dy = evt.y - start.1;

                if self.mode == InteractionMode::Draw
                    && self.active_tool == Some(DrawingTool::Select)
                    && (dx.abs() > 0.5 || dy.abs() > 0.5)
                {
                    return vec![
                        ChartCommand::MoveSelected {
                            delta_timestamp: dx.round() as i64 as u64,
                            delta_price: dy,
                        },
                        ChartCommand::RequestRedraw,
                    ];
                }
            }
        }

        vec![
            ChartCommand::UpdateCrosshair {
                screen_x: evt.x,
                screen_y: evt.y,
            },
            ChartCommand::RequestRedraw,
        ]
    }

    fn handle_mouse_down(&mut self, evt: MouseButtonEvent) -> Vec<ChartCommand> {
        if evt.button != MouseButton::Left {
            return Vec::new();
        }

        self.mouse_down = true;
        self.drag_start = Some((evt.x, evt.y));
        self.last_mouse = Some((evt.x, evt.y));

        self.sync_modifiers(evt.modifiers);

        if let Some(tool) = self.active_tool {
            match tool {
                DrawingTool::Select => {
                    vec![
                        ChartCommand::SelectDrawing {
                            screen_x: evt.x,
                            screen_y: evt.y,
                        },
                        ChartCommand::RequestRedraw,
                    ]
                }
                _ => {
                    vec![
                        ChartCommand::PlaceDrawingPoint {
                            screen_x: evt.x,
                            screen_y: evt.y,
                        },
                        ChartCommand::RequestRedraw,
                    ]
                }
            }
        } else {
            vec![ChartCommand::RequestRedraw]
        }
    }

    fn handle_mouse_up(&mut self, evt: MouseButtonEvent) -> Vec<ChartCommand> {
        if evt.button != MouseButton::Left {
            return Vec::new();
        }

        self.mouse_down = false;
        self.drag_start = None;
        self.sync_modifiers(evt.modifiers);

        Vec::new()
    }

    fn handle_wheel(&mut self, evt: WheelEvent) -> Vec<ChartCommand> {
        self.sync_modifiers(evt.modifiers);

        if self.shift_held {
            let mut cmds = Vec::new();

            if evt.delta_x.abs() > 0.01 {
                cmds.push(ChartCommand::Pan {
                    time_delta: evt.delta_x.round() as i64,
                    price_delta: 0.0,
                });
            }
            if evt.delta_y.abs() > 0.01 {
                cmds.push(ChartCommand::Pan {
                    time_delta: 0,
                    price_delta: evt.delta_y,
                });
            }

            if cmds.is_empty() {
                return Vec::new();
            }

            cmds.push(ChartCommand::RequestRedraw);
            return cmds;
        }

        // Vertical scroll → zoom. Negative delta_y = scroll up = zoom in.
        let factor = if evt.delta_y < 0.0 { 1.1 } else { 1.0 / 1.1 };

        vec![
            ChartCommand::ZoomAtCursor {
                factor,
                screen_x: evt.x,
                screen_y: evt.y,
            },
            ChartCommand::RequestRedraw,
        ]
    }

    fn handle_touch_start(&mut self, evt: TouchEvent) -> Vec<ChartCommand> {
        self.mouse_down = true;
        self.drag_start = Some((evt.x, evt.y));
        self.last_mouse = Some((evt.x, evt.y));

        if let Some(tool) = self.active_tool {
            match tool {
                DrawingTool::Select => {
                    vec![
                        ChartCommand::SelectDrawing {
                            screen_x: evt.x,
                            screen_y: evt.y,
                        },
                        ChartCommand::RequestRedraw,
                    ]
                }
                _ => {
                    vec![
                        ChartCommand::PlaceDrawingPoint {
                            screen_x: evt.x,
                            screen_y: evt.y,
                        },
                        ChartCommand::RequestRedraw,
                    ]
                }
            }
        } else {
            vec![ChartCommand::RequestRedraw]
        }
    }

    fn handle_touch_move(&mut self, evt: TouchEvent) -> Vec<ChartCommand> {
        self.last_mouse = Some((evt.x, evt.y));

        if self.mouse_down {
            if let Some(start) = self.drag_start {
                let dx = evt.x - start.0;
                let dy = evt.y - start.1;

                if self.mode == InteractionMode::Draw
                    && self.active_tool == Some(DrawingTool::Select)
                    && (dx.abs() > 0.5 || dy.abs() > 0.5)
                {
                    return vec![
                        ChartCommand::MoveSelected {
                            delta_timestamp: dx.round() as i64 as u64,
                            delta_price: dy,
                        },
                        ChartCommand::RequestRedraw,
                    ];
                }
            }
        }

        vec![
            ChartCommand::UpdateCrosshair {
                screen_x: evt.x,
                screen_y: evt.y,
            },
            ChartCommand::RequestRedraw,
        ]
    }

    fn handle_touch_end(&mut self, _evt: TouchEvent) -> Vec<ChartCommand> {
        self.mouse_down = false;
        self.drag_start = None;
        Vec::new()
    }

    fn handle_pinch(&mut self, evt: PinchEvent) -> Vec<ChartCommand> {
        vec![
            ChartCommand::ZoomAtCursor {
                factor: evt.scale,
                screen_x: evt.center_x,
                screen_y: evt.center_y,
            },
            ChartCommand::RequestRedraw,
        ]
    }

    fn handle_key_down(&mut self, evt: KeyEvent) -> Vec<ChartCommand> {
        self.sync_modifiers(evt.modifiers);

        match evt.key {
            KeyCode::Escape => {
                let mut cmds = Vec::new();
                if self.mode == InteractionMode::Draw {
                    cmds.push(ChartCommand::CancelDrawing);
                    self.set_tool(None);
                }
                cmds.push(ChartCommand::DeselectAll);
                cmds.push(ChartCommand::RequestRedraw);
                cmds
            }
            KeyCode::Delete | KeyCode::Backspace => {
                vec![
                    ChartCommand::DeleteSelected,
                    ChartCommand::RequestRedraw,
                ]
            }
            KeyCode::Plus | KeyCode::Equal => {
                vec![
                    ChartCommand::ZoomAtCursor {
                        factor: 1.2,
                        screen_x: self.last_mouse.unwrap_or((0.0, 0.0)).0,
                        screen_y: self.last_mouse.unwrap_or((0.0, 0.0)).1,
                    },
                    ChartCommand::RequestRedraw,
                ]
            }
            KeyCode::Minus => {
                vec![
                    ChartCommand::ZoomAtCursor {
                        factor: 1.0 / 1.2,
                        screen_x: self.last_mouse.unwrap_or((0.0, 0.0)).0,
                        screen_y: self.last_mouse.unwrap_or((0.0, 0.0)).1,
                    },
                    ChartCommand::RequestRedraw,
                ]
            }
            KeyCode::ArrowLeft => {
                vec![
                    ChartCommand::Pan { time_delta: -60, price_delta: 0.0 },
                    ChartCommand::RequestRedraw,
                ]
            }
            KeyCode::ArrowRight => {
                vec![
                    ChartCommand::Pan { time_delta: 60, price_delta: 0.0 },
                    ChartCommand::RequestRedraw,
                ]
            }
            KeyCode::ArrowUp => {
                vec![
                    ChartCommand::Pan { time_delta: 0, price_delta: -1.0 },
                    ChartCommand::RequestRedraw,
                ]
            }
            KeyCode::ArrowDown => {
                vec![
                    ChartCommand::Pan { time_delta: 0, price_delta: 1.0 },
                    ChartCommand::RequestRedraw,
                ]
            }
            KeyCode::Key1 => self.start_drawing_tool(DrawingTool::TrendLine),
            KeyCode::Key2 => self.start_drawing_tool(DrawingTool::Arrow),
            KeyCode::Key3 => self.start_drawing_tool(DrawingTool::Ray),
            KeyCode::Key4 => self.start_drawing_tool(DrawingTool::Segment),
            KeyCode::Key5 => self.start_drawing_tool(DrawingTool::Rectangle),
            KeyCode::Key6 => self.start_drawing_tool(DrawingTool::Ellipse),
            KeyCode::Key7 => self.start_drawing_tool(DrawingTool::HorizontalLine),
            KeyCode::Key8 => self.start_drawing_tool(DrawingTool::VerticalLine),
            KeyCode::Key9 => self.start_drawing_tool(DrawingTool::FibonacciRetracement),
            _ => Vec::new(),
        }
    }

    fn handle_key_up(&mut self, evt: KeyEvent) -> Vec<ChartCommand> {
        self.sync_modifiers(evt.modifiers);
        Vec::new()
    }

    fn start_drawing_tool(&mut self, tool: DrawingTool) -> Vec<ChartCommand> {
        self.set_tool(Some(tool));
        vec![
            ChartCommand::StartDrawing { tool },
            ChartCommand::RequestRedraw,
        ]
    }

    fn sync_modifiers(&mut self, mods: ModifierState) {
        self.shift_held = mods.shift;
        self.ctrl_held = mods.ctrl;
    }
}

impl Default for InteractionEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ModifierState;

    fn mods(shift: bool, ctrl: bool) -> ModifierState {
        ModifierState { shift, ctrl, alt: false, super_key: false }
    }

    // Clasificación: determinística — verifica que update() avanza el tiempo y produce valor interpolado
    #[test]
    fn mouse_move_produces_update_crosshair() {
        let mut eng = InteractionEngine::new();
        let cmds = eng.handle(InputEvent::mouse_move(100.0, 200.0));

        assert_eq!(cmds.len(), 2);
        assert_eq!(
            cmds[0],
            ChartCommand::UpdateCrosshair {
                screen_x: 100.0,
                screen_y: 200.0,
            }
        );
        assert_eq!(cmds[1], ChartCommand::RequestRedraw);
    }

    // Clasificación: determinística — verifica mouse_down_and_up_cycle
    #[test]
    fn mouse_down_and_up_cycle() {
        let mut eng = InteractionEngine::new();
        let down_cmds = eng.handle(InputEvent::mouse_left_down(50.0, 60.0));
        assert!(down_cmds.contains(&ChartCommand::RequestRedraw));

        let up_cmds = eng.handle(InputEvent::mouse_left_up(50.0, 60.0));
        assert!(up_cmds.is_empty());
    }

    // Clasificación: determinística — verifica wheel_produces_zoom_in
    #[test]
    fn wheel_produces_zoom_in() {
        let mut eng = InteractionEngine::new();
        let cmds = eng.handle(InputEvent::wheel(0.0, -3.0, 400.0, 300.0));

        assert!(cmds.contains(&ChartCommand::ZoomAtCursor {
            factor: 1.1,
            screen_x: 400.0,
            screen_y: 300.0,
        }));
    }

    // Clasificación: determinística — verifica wheel_produces_zoom_out
    #[test]
    fn wheel_produces_zoom_out() {
        let mut eng = InteractionEngine::new();
        let cmds = eng.handle(InputEvent::wheel(0.0, 3.0, 400.0, 300.0));

        let expected_factor = 1.0 / 1.1;
        assert!(cmds.iter().any(|c| matches!(
            c,
            ChartCommand::ZoomAtCursor { factor, .. } if (*factor - expected_factor).abs() < 1e-10
        )));
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn wheel_with_shift_produces_pan() {
        let mut eng = InteractionEngine::new();
        let evt = InputEvent::wheel_with_modifiers(
            0.0, -5.0, 400.0, 300.0, mods(true, false),
        );
        let cmds = eng.handle(evt);

        assert!(cmds.iter().any(|c| matches!(
            c,
            ChartCommand::Pan { price_delta, .. } if (*price_delta - (-5.0)).abs() < 1e-10
        )));
    }

    // Clasificación: determinística — verifica escape_cancels_drawing_and_deselects
    #[test]
    fn escape_cancels_drawing_and_deselects() {
        let mut eng = InteractionEngine::new();
        eng.set_tool(Some(DrawingTool::TrendLine));
        assert!(eng.is_drawing());

        let cmds = eng.handle(InputEvent::key_down(KeyCode::Escape));

        assert!(cmds.contains(&ChartCommand::CancelDrawing));
        assert!(cmds.contains(&ChartCommand::DeselectAll));
        assert!(!eng.is_drawing());
        assert_eq!(eng.active_tool(), None);
    }

    // Clasificación: determinística — verifica escape_without_tool_only_deselects
    #[test]
    fn escape_without_tool_only_deselects() {
        let mut eng = InteractionEngine::new();
        let cmds = eng.handle(InputEvent::key_down(KeyCode::Escape));

        assert!(!cmds.contains(&ChartCommand::CancelDrawing));
        assert!(cmds.contains(&ChartCommand::DeselectAll));
    }

    // Clasificación: determinística — verifica delete_produces_delete_selected
    #[test]
    fn delete_produces_delete_selected() {
        let mut eng = InteractionEngine::new();
        let cmds = eng.handle(InputEvent::key_down(KeyCode::Delete));

        assert!(cmds.contains(&ChartCommand::DeleteSelected));
    }

    // Clasificación: determinística — verifica backspace_produces_delete_selected
    #[test]
    fn backspace_produces_delete_selected() {
        let mut eng = InteractionEngine::new();
        let cmds = eng.handle(InputEvent::key_down(KeyCode::Backspace));

        assert!(cmds.contains(&ChartCommand::DeleteSelected));
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn arrow_right_produces_pan() {
        let mut eng = InteractionEngine::new();
        let cmds = eng.handle(InputEvent::key_down(KeyCode::ArrowRight));

        assert!(cmds.contains(&ChartCommand::Pan {
            time_delta: 60,
            price_delta: 0.0,
        }));
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn arrow_left_produces_pan_negative() {
        let mut eng = InteractionEngine::new();
        let cmds = eng.handle(InputEvent::key_down(KeyCode::ArrowLeft));

        assert!(cmds.contains(&ChartCommand::Pan {
            time_delta: -60,
            price_delta: 0.0,
        }));
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn arrow_up_produces_pan_price() {
        let mut eng = InteractionEngine::new();
        let cmds = eng.handle(InputEvent::key_down(KeyCode::ArrowUp));

        assert!(cmds.contains(&ChartCommand::Pan {
            time_delta: 0,
            price_delta: -1.0,
        }));
    }

    // Clasificación: determinística — verifica key_1_starts_trend_line
    #[test]
    fn key_1_starts_trend_line() {
        let mut eng = InteractionEngine::new();
        let cmds = eng.handle(InputEvent::key_down(KeyCode::Key1));

        assert!(cmds.contains(&ChartCommand::StartDrawing {
            tool: DrawingTool::TrendLine,
        }));
        assert!(eng.is_drawing());
        assert_eq!(eng.active_tool(), Some(DrawingTool::TrendLine));
    }

    // Clasificación: determinística — verifica key_9_starts_fibonacci_retracement
    #[test]
    fn key_9_starts_fibonacci_retracement() {
        let mut eng = InteractionEngine::new();
        let cmds = eng.handle(InputEvent::key_down(KeyCode::Key9));

        assert!(cmds.contains(&ChartCommand::StartDrawing {
            tool: DrawingTool::FibonacciRetracement,
        }));
    }

    // Clasificación: determinística — verifica shift_tracking_across_key_down_up
    #[test]
    fn shift_tracking_across_key_down_up() {
        let mut eng = InteractionEngine::new();

        let down_mods = ModifierState { shift: true, ctrl: false, alt: false, super_key: false };
        eng.handle(InputEvent::key_down_with_modifiers(
            KeyCode::Shift, down_mods,
        ));

        let cmds = eng.handle(InputEvent::wheel_with_modifiers(
            0.0, -5.0, 400.0, 300.0,
            ModifierState { shift: true, ctrl: false, alt: false, super_key: false },
        ));
        assert!(cmds.iter().any(|c| matches!(c, ChartCommand::Pan { .. })));

        // Release shift
        eng.handle(InputEvent::key_up(KeyCode::Shift));
        let cmds2 = eng.handle(InputEvent::wheel(0.0, -3.0, 400.0, 300.0));
        assert!(cmds2.iter().any(|c| matches!(c, ChartCommand::ZoomAtCursor { .. })));
    }

    // Clasificación: determinística — verifica drag_cycle_mouse_down_move_up
    #[test]
    fn drag_cycle_mouse_down_move_up() {
        let mut eng = InteractionEngine::new();
        eng.handle(InputEvent::mouse_left_down(100.0, 100.0));
        let move_cmds = eng.handle(InputEvent::mouse_move(150.0, 150.0));
        assert!(move_cmds.iter().any(|c| matches!(c, ChartCommand::UpdateCrosshair { .. })));
        let up_cmds = eng.handle(InputEvent::mouse_left_up(150.0, 150.0));
        assert!(up_cmds.is_empty());
    }

    // Clasificación: determinística — verifica detección de gesto pinch con dos dedos
    #[test]
    fn pinch_produces_zoom() {
        let mut eng = InteractionEngine::new();
        let cmds = eng.handle(InputEvent::pinch(400.0, 300.0, 200.0, 1.5));
        assert!(cmds.contains(&ChartCommand::ZoomAtCursor {
            factor: 1.5,
            screen_x: 400.0,
            screen_y: 300.0,
        }));
    }

    // Clasificación: determinística — verifica set_tool_active_tool_is_drawing
    #[test]
    fn set_tool_active_tool_is_drawing() {
        let mut eng = InteractionEngine::new();
        assert_eq!(eng.active_tool(), None);
        assert!(!eng.is_drawing());

        eng.set_tool(Some(DrawingTool::Rectangle));
        assert_eq!(eng.active_tool(), Some(DrawingTool::Rectangle));
        assert!(eng.is_drawing());

        eng.set_tool(None);
        assert_eq!(eng.active_tool(), None);
        assert!(!eng.is_drawing());
    }

    // Clasificación: determinística — verifica mouse_down_with_select_tool_emits_select_drawing
    #[test]
    fn mouse_down_with_select_tool_emits_select_drawing() {
        let mut eng = InteractionEngine::new();
        eng.set_tool(Some(DrawingTool::Select));
        let cmds = eng.handle(InputEvent::mouse_left_down(200.0, 300.0));
        assert!(cmds.contains(&ChartCommand::SelectDrawing {
            screen_x: 200.0,
            screen_y: 300.0,
        }));
    }

    // Clasificación: determinística — verifica mouse_down_with_drawing_tool_emits_place_point
    #[test]
    fn mouse_down_with_drawing_tool_emits_place_point() {
        let mut eng = InteractionEngine::new();
        eng.set_tool(Some(DrawingTool::TrendLine));
        let cmds = eng.handle(InputEvent::mouse_left_down(100.0, 200.0));
        assert!(cmds.contains(&ChartCommand::PlaceDrawingPoint {
            screen_x: 100.0,
            screen_y: 200.0,
        }));
    }

    // Clasificación: determinística — verifica mouse_down_right_button_is_noop
    #[test]
    fn mouse_down_right_button_is_noop() {
        let mut eng = InteractionEngine::new();
        let cmds = eng.handle(InputEvent::MouseDown(MouseButtonEvent {
            button: MouseButton::Right,
            x: 100.0, y: 200.0,
            modifiers: ModifierState::default(),
        }));
        assert!(cmds.is_empty());
    }

    // Clasificación: determinística — verifica plus_key_zooms_in
    #[test]
    fn plus_key_zooms_in() {
        let mut eng = InteractionEngine::new();
        eng.handle(InputEvent::mouse_move(500.0, 400.0));
        let cmds = eng.handle(InputEvent::key_down(KeyCode::Plus));
        assert!(cmds.iter().any(|c| matches!(
            c,
            ChartCommand::ZoomAtCursor { factor: 1.2, screen_x: 500.0, screen_y: 400.0 }
        )));
    }

    // Clasificación: determinística — verifica minus_key_zooms_out
    #[test]
    fn minus_key_zooms_out() {
        let mut eng = InteractionEngine::new();
        eng.handle(InputEvent::mouse_move(500.0, 400.0));
        let cmds = eng.handle(InputEvent::key_down(KeyCode::Minus));
        let expected = 1.0 / 1.2;
        assert!(cmds.iter().any(|c| matches!(c, ChartCommand::ZoomAtCursor { factor, .. } if (*factor - expected).abs() < 1e-10)));
    }

    // Clasificación: determinística — verifica double_click_is_noop
    #[test]
    fn double_click_is_noop() {
        let mut eng = InteractionEngine::new();
        let cmds = eng.handle(InputEvent::mouse_double_click(100.0, 200.0));
        assert!(cmds.is_empty());
    }

    // Clasificación: determinística — verifica stylus_move_produces_crosshair
    #[test]
    fn stylus_move_produces_crosshair() {
        let mut eng = InteractionEngine::new();
        let cmds = eng.handle(InputEvent::stylus_move(150.0, 250.0, 0.5));
        assert!(cmds.iter().any(|c| matches!(c, ChartCommand::UpdateCrosshair { screen_x: 150.0, screen_y: 250.0 })));
    }

    // Clasificación: determinística — verifica touch_start_and_end_cycle
    #[test]
    fn touch_start_and_end_cycle() {
        let mut eng = InteractionEngine::new();
        let down_cmds = eng.handle(InputEvent::touch_start(1, 50.0, 60.0));
        assert!(down_cmds.contains(&ChartCommand::RequestRedraw));
        let up_cmds = eng.handle(InputEvent::touch_end(1, 50.0, 60.0));
        assert!(up_cmds.is_empty());
    }

    // Clasificación: determinística — verifica draw_select_drag_emits_move_selected
    #[test]
    fn draw_select_drag_emits_move_selected() {
        let mut eng = InteractionEngine::new();
        eng.set_tool(Some(DrawingTool::Select));
        eng.handle(InputEvent::mouse_left_down(100.0, 100.0));
        let cmds = eng.handle(InputEvent::mouse_move(110.0, 115.0));
        assert!(cmds.iter().any(|c| matches!(c, ChartCommand::MoveSelected { delta_timestamp: _, delta_price: _ })));
    }
}
