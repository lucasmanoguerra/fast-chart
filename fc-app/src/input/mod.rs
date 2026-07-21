//! Platform-agnostic input events for the chart interaction system.
//!
//! Re-exports the entire `fc-input` crate. All input types, events,
//! and the `InteractionEngine` live in that crate.
//!
//! `InputEvent` is the raw, device-independent vocabulary that adapters
//! (winit, web, etc.) translate platform-specific events into. The
//! `InteractionEngine` consumes these to produce `ChartCommand`s.

pub use fc_input::*;
