# Input System

## Platform-Agnostic Events

All input is expressed as `InputEvent` — a platform-agnostic vocabulary that
adapters (winit, web, etc.) translate platform-specific events into.

```rust
pub enum InputEvent {
    // Mouse
    MouseMove(MouseMoveEvent),
    MouseDown(MouseButtonEvent),
    MouseUp(MouseButtonEvent),
    Wheel(WheelEvent),
    MouseDoubleClick(MouseButtonEvent),

    // Touch
    TouchStart(TouchEvent),
    TouchMove(TouchEvent),
    TouchEnd(TouchEvent),
    Pinch(PinchEvent),

    // Keyboard
    KeyDown(KeyEvent),
    KeyUp(KeyEvent),

    // Stylus
    StylusMove(StylusEvent),
    StylusDown(StylusEvent),
    StylusUp(StylusEvent),
}
```

Each variant carries a typed payload with position, modifiers, pressure, etc.

---

## InteractionEngine

Consumes `InputEvent`s and produces `ChartCommand`s (viewport operations):

```rust
pub enum InteractionCommand {
    ZoomAtCursor { factor: f64, screen_x: f64 },
    PanBy { time_delta: i64 },
    UpdateCrosshair { screen_x: f64, screen_y: f64 },
    DeactivateCrosshair,
    SwitchTimeframe { timeframe: String },
    ResizePane { pane_index: usize, new_height: f64 },
}
```

---

## Sub-Modules

### zoom

Handles wheel zoom, pinch zoom, and axis zoom. Zoom is centered on the
cursor position by default.

### pan

Mouse drag panning, touch drag panning, momentum scrolling, and inertia.
Connected to `KineticScroll` for smooth deceleration.

### crosshair

Crosshair positioning with magnetic snapping to nearest OHLC price.
Supports normal, magnetic, hidden, and synced modes.

### gesture

Multi-touch gesture recognition for mobile/tablet. Pinch-to-zoom,
two-finger scroll, long-press context menu.

### keyboard

Keyboard shortcuts for chart navigation:
- Arrow keys: pan left/right
- `+`/`-`: zoom in/out
- `Home`/`End`: jump to start/end
- Number keys: drawing tool selection
- `Escape`: deselect/cancel

---

## Event Flow

```
Platform Event (winit/web)
  ↓  Adapter
InputEvent
  ↓  InteractionEngine
InteractionCommand
  ↓  ChartController
ViewportCommand
  ↓  ViewportManager
State Update → DirtyRegion → Redraw
```
