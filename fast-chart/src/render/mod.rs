//! Rendering abstractions for the chart library.
//!
//! This module defines the universal render commands, layer system,
//! coordinate pipeline, and renderer backend trait.

pub mod backend;
pub mod commands;
pub mod context;
pub mod coordinates;
pub mod indicator_renderer;
pub mod layers;
pub mod series_renderer;

pub use backend::RendererBackend;
pub use commands::{DrawCommand, LineStyle};
pub use context::RenderContext;
pub use coordinates::{CoordinatePipeline, ScreenPoint, WorldPoint};
pub use indicator_renderer::IndicatorRenderer;
pub use layers::DrawLayer;
pub use series_renderer::{Rect, SeriesHit, SeriesRenderer};
