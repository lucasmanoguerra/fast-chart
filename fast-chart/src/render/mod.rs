//! Rendering abstractions for the chart library.
//!
//! This module defines the universal render commands, layer system,
//! coordinate pipeline, and renderer backend trait.

pub mod backend;
pub mod commands;
pub mod coordinates;
pub mod layers;
pub mod series_renderer;

pub use backend::RendererBackend;
pub use commands::{DrawCommand, LineStyle};
pub use coordinates::{CoordinatePipeline, ScreenPoint, WorldPoint};
pub use layers::DrawLayer;
pub use series_renderer::{Rect, SeriesHit, SeriesRenderer};
