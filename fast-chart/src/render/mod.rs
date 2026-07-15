//! Rendering abstractions for the chart library.
//!
//! This module defines the universal render commands, layer system,
//! coordinate pipeline, and renderer backend trait.

pub mod commands;

pub use commands::{DrawCommand, LineStyle};
