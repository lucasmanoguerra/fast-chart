//! Platform-agnostic input events for the chart interaction system.
//!
//! `InputEvent` is the raw, device-independent vocabulary that adapters
//! (winit, web, etc.) translate platform-specific events into. The
//! `InteractionEngine` consumes these to produce `ChartCommand`s.

// ---------------------------------------------------------------------------
// InputEvent — platform-agnostic input vocabulary
// ---------------------------------------------------------------------------

use bitflags::bitflags;

bitflags! {
    /// Bitflags for modifier keys — compact storage and fast membership tests.
    ///
    /// Prefer this over `ModifierState` when storing or comparing modifiers
    /// in hot paths (e.g., keyboard shortcut matching).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ModifierFlags: u8 {
        const SHIFT  = 0b0001;
        const CTRL   = 0b0010;
        const ALT    = 0b0100;
        const SUPER  = 0b1000;
    }
}

impl ModifierFlags {
    /// Convert from `ModifierState` struct.
    pub fn from_state(state: ModifierState) -> Self {
        let mut flags = Self::empty();
        if state.shift { flags |= Self::SHIFT; }
        if state.ctrl { flags |= Self::CTRL; }
        if state.alt { flags |= Self::ALT; }
        if state.super_key { flags |= Self::SUPER; }
        flags
    }
}

/// A platform-agnostic input event.
///
/// Adapters convert platform-specific events (winit, web, touch) into these.
/// The `InteractionEngine` consumes them to produce `ChartCommand`s.
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    // --- Mouse ---
    MouseMove(MouseMoveEvent),
    MouseDown(MouseButtonEvent),
    MouseUp(MouseButtonEvent),
    Wheel(WheelEvent),
    MouseDoubleClick(MouseButtonEvent),

    // --- Touch ---
    TouchStart(TouchEvent),
    TouchMove(TouchEvent),
    TouchEnd(TouchEvent),
    Pinch(PinchEvent),

    // --- Keyboard ---
    KeyDown(KeyEvent),
    KeyUp(KeyEvent),

    // --- Stylus ---
    StylusMove(StylusEvent),
    StylusDown(StylusEvent),
    StylusUp(StylusEvent),
}

// ---------------------------------------------------------------------------
// Event payloads
// ---------------------------------------------------------------------------

/// Button that triggered a mouse event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

/// State of modifier keys during an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

/// Mouse cursor position and button info.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseMoveEvent {
    pub x: f64,
    pub y: f64,
    pub modifiers: ModifierState,
}

/// Mouse button press/release event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseButtonEvent {
    pub button: MouseButton,
    pub x: f64,
    pub y: f64,
    pub modifiers: ModifierState,
}

/// Mouse wheel / trackpad scroll event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelEvent {
    pub delta_x: f64,
    pub delta_y: f64,
    pub x: f64,
    pub y: f64,
    pub modifiers: ModifierState,
}

/// Touch contact event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchEvent {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub pressure: Option<f32>,
}

/// Two-finger pinch gesture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PinchEvent {
    pub center_x: f64,
    pub center_y: f64,
    pub distance: f64,
    pub scale: f64,
}

/// Keyboard key event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: KeyCode,
    pub modifiers: ModifierState,
    pub repeat: bool,
}

/// Platform-independent key codes.
///
/// Covers the keys relevant for chart interaction (navigation, shortcuts,
/// drawing tool selection). Not an exhaustive keyboard map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
    PageUp,
    PageDown,
    Plus,
    Minus,
    Equal,
    Escape,
    Delete,
    Backspace,
    Enter,
    Key1, Key2, Key3, Key4, Key5,
    Key6, Key7, Key8, Key9, Key0,
    Letter(char),
    Shift,
    Control,
    Alt,
    Super,
    F1, F2, F3, F4, F5,
    F6, F7, F8, F9, F10, F11, F12,
    Unknown,
}

/// Stylus/pen event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StylusEvent {
    pub x: f64,
    pub y: f64,
    pub pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub modifiers: ModifierState,
}

// ---------------------------------------------------------------------------
// Convenience constructors
// ---------------------------------------------------------------------------

impl InputEvent {
    pub fn mouse_move(x: f64, y: f64) -> Self {
        InputEvent::MouseMove(MouseMoveEvent {
            x, y, modifiers: ModifierState::default(),
        })
    }

    pub fn mouse_move_with_modifiers(x: f64, y: f64, modifiers: ModifierState) -> Self {
        InputEvent::MouseMove(MouseMoveEvent { x, y, modifiers })
    }

    pub fn mouse_left_down(x: f64, y: f64) -> Self {
        InputEvent::MouseDown(MouseButtonEvent {
            button: MouseButton::Left, x, y, modifiers: ModifierState::default(),
        })
    }

    pub fn mouse_left_up(x: f64, y: f64) -> Self {
        InputEvent::MouseUp(MouseButtonEvent {
            button: MouseButton::Left, x, y, modifiers: ModifierState::default(),
        })
    }

    pub fn mouse_double_click(x: f64, y: f64) -> Self {
        InputEvent::MouseDoubleClick(MouseButtonEvent {
            button: MouseButton::Left, x, y, modifiers: ModifierState::default(),
        })
    }

    pub fn wheel(delta_x: f64, delta_y: f64, x: f64, y: f64) -> Self {
        InputEvent::Wheel(WheelEvent {
            delta_x, delta_y, x, y, modifiers: ModifierState::default(),
        })
    }

    pub fn wheel_with_modifiers(
        delta_x: f64, delta_y: f64, x: f64, y: f64, modifiers: ModifierState,
    ) -> Self {
        InputEvent::Wheel(WheelEvent { delta_x, delta_y, x, y, modifiers })
    }

    pub fn key_down(key: KeyCode) -> Self {
        InputEvent::KeyDown(KeyEvent {
            key, modifiers: ModifierState::default(), repeat: false,
        })
    }

    pub fn key_down_with_modifiers(key: KeyCode, modifiers: ModifierState) -> Self {
        InputEvent::KeyDown(KeyEvent { key, modifiers, repeat: false })
    }

    pub fn key_up(key: KeyCode) -> Self {
        InputEvent::KeyUp(KeyEvent {
            key, modifiers: ModifierState::default(), repeat: false,
        })
    }

    pub fn touch_start(id: u64, x: f64, y: f64) -> Self {
        InputEvent::TouchStart(TouchEvent { id, x, y, pressure: None })
    }

    pub fn touch_move(id: u64, x: f64, y: f64) -> Self {
        InputEvent::TouchMove(TouchEvent { id, x, y, pressure: None })
    }

    pub fn touch_end(id: u64, x: f64, y: f64) -> Self {
        InputEvent::TouchEnd(TouchEvent { id, x, y, pressure: None })
    }

    pub fn pinch(center_x: f64, center_y: f64, distance: f64, scale: f64) -> Self {
        InputEvent::Pinch(PinchEvent { center_x, center_y, distance, scale })
    }

    pub fn stylus_move(x: f64, y: f64, pressure: f32) -> Self {
        InputEvent::StylusMove(StylusEvent {
            x, y, pressure, tilt_x: 0.0, tilt_y: 0.0, modifiers: ModifierState::default(),
        })
    }
}

pub mod crosshair;
pub mod engine;
pub mod gesture;
pub mod keyboard;
pub mod pan;
pub mod zoom;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_event_mouse_move() {
        let e = InputEvent::mouse_move(100.0, 200.0);
        match e {
            InputEvent::MouseMove(evt) => {
                assert_eq!(evt.x, 100.0);
                assert_eq!(evt.y, 200.0);
                assert_eq!(evt.modifiers, ModifierState::default());
            }
            _ => panic!("Expected MouseMove"),
        }
    }

    #[test]
    fn input_event_mouse_left_click() {
        let down = InputEvent::mouse_left_down(50.0, 75.0);
        let up = InputEvent::mouse_left_up(50.0, 75.0);
        assert_ne!(down, up);
        assert_eq!(down, InputEvent::MouseDown(MouseButtonEvent {
            button: MouseButton::Left, x: 50.0, y: 75.0, modifiers: ModifierState::default(),
        }));
    }

    #[test]
    fn input_event_wheel() {
        let e = InputEvent::wheel(0.0, -3.0, 400.0, 300.0);
        match e {
            InputEvent::Wheel(evt) => {
                assert_eq!(evt.delta_x, 0.0);
                assert_eq!(evt.delta_y, -3.0);
                assert_eq!(evt.x, 400.0);
                assert_eq!(evt.y, 300.0);
            }
            _ => panic!("Expected Wheel"),
        }
    }

    #[test]
    fn input_event_key_down() {
        let e = InputEvent::key_down(KeyCode::Escape);
        match e {
            InputEvent::KeyDown(evt) => {
                assert_eq!(evt.key, KeyCode::Escape);
                assert!(!evt.repeat);
            }
            _ => panic!("Expected KeyDown"),
        }
    }

    #[test]
    fn input_event_key_with_modifiers() {
        let mods = ModifierState { shift: true, ctrl: true, alt: false, super_key: false };
        let e = InputEvent::key_down_with_modifiers(KeyCode::Letter('z'), mods);
        match e {
            InputEvent::KeyDown(evt) => {
                assert_eq!(evt.key, KeyCode::Letter('z'));
                assert!(evt.modifiers.shift);
                assert!(evt.modifiers.ctrl);
                assert!(!evt.modifiers.alt);
            }
            _ => panic!("Expected KeyDown"),
        }
    }

    #[test]
    fn input_event_touch() {
        let start = InputEvent::touch_start(1, 100.0, 200.0);
        let mov = InputEvent::touch_move(1, 110.0, 210.0);
        let end = InputEvent::touch_end(1, 110.0, 210.0);
        assert_ne!(start, mov);
        assert_ne!(mov, end);
    }

    #[test]
    fn input_event_pinch() {
        let e = InputEvent::pinch(400.0, 300.0, 200.0, 1.5);
        match e {
            InputEvent::Pinch(evt) => {
                assert_eq!(evt.center_x, 400.0);
                assert_eq!(evt.center_y, 300.0);
                assert_eq!(evt.distance, 200.0);
                assert_eq!(evt.scale, 1.5);
            }
            _ => panic!("Expected Pinch"),
        }
    }

    #[test]
    fn input_event_stylus() {
        let e = InputEvent::stylus_move(100.0, 200.0, 0.75);
        match e {
            InputEvent::StylusMove(evt) => {
                assert_eq!(evt.x, 100.0);
                assert_eq!(evt.pressure, 0.75);
            }
            _ => panic!("Expected StylusMove"),
        }
    }

    #[test]
    fn input_event_double_click() {
        let e = InputEvent::mouse_double_click(100.0, 200.0);
        assert!(matches!(e, InputEvent::MouseDoubleClick(_)));
    }

    #[test]
    fn modifier_state_default() {
        let m = ModifierState::default();
        assert!(!m.shift);
        assert!(!m.ctrl);
        assert!(!m.alt);
        assert!(!m.super_key);
    }

    #[test]
    fn modifier_state_equality() {
        let a = ModifierState { shift: true, ctrl: false, alt: false, super_key: false };
        let b = ModifierState { shift: true, ctrl: false, alt: false, super_key: false };
        let c = ModifierState { shift: false, ctrl: true, alt: false, super_key: false };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn mouse_button_variants() {
        assert_eq!(MouseButton::Left, MouseButton::Left);
        assert_ne!(MouseButton::Left, MouseButton::Right);
        assert_ne!(MouseButton::Middle, MouseButton::Other(5));
        assert_eq!(MouseButton::Other(5), MouseButton::Other(5));
    }

    #[test]
    fn key_code_clone_and_hash() {
        let k1 = KeyCode::Letter('a');
        let k2 = k1;
        let mut set = std::collections::HashSet::new();
        set.insert(k1);
        set.insert(k2);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn input_event_clone() {
        let e = InputEvent::mouse_move(1.0, 2.0);
        let cloned = e.clone();
        assert_eq!(e, cloned);
    }

    #[test]
    fn input_event_with_modifiers() {
        let mods = ModifierState { shift: false, ctrl: true, alt: false, super_key: false };
        let e = InputEvent::mouse_move_with_modifiers(100.0, 200.0, mods);
        match e {
            InputEvent::MouseMove(evt) => assert!(evt.modifiers.ctrl),
            _ => panic!("Expected MouseMove"),
        }
    }

    #[test]
    fn wheel_with_modifiers() {
        let mods = ModifierState { shift: true, ctrl: false, alt: false, super_key: false };
        let e = InputEvent::wheel_with_modifiers(0.0, -1.0, 400.0, 300.0, mods);
        match e {
            InputEvent::Wheel(evt) => assert!(evt.modifiers.shift),
            _ => panic!("Expected Wheel"),
        }
    }
}
