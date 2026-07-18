//! Rendering abstractions for the chart library.
//!
//! This module defines the universal render commands, layer system,
//! coordinate pipeline, and renderer backend trait.

pub mod backend;
pub mod commands;
pub mod dirty;
pub mod passes;
pub mod pipeline;
pub mod context;
pub mod coordinates;
pub mod drawing;
pub mod drawing_interaction;
pub mod drawing_manager;
pub mod indicator_renderer;
pub mod layers;
pub mod pixel_perfect;
pub mod series_renderer;
pub mod session;

pub use backend::RendererBackend;
pub use commands::{DrawCommand, LineStyle};
pub use dirty::{DirtyRegion, DirtyRegionTracker, ScreenRect};
pub use passes::{PassTracker, RenderPass};
pub use pipeline::{FrameStats, PassBatch, RenderPipeline, z_index_to_pass};
pub use context::RenderContext;
pub use coordinates::{CoordinatePipeline, ScreenPoint, WorldPoint};
pub use drawing::{Drawing, DrawingBounds, HitResult};
pub use drawing_interaction::{DrawingAction, DrawingInteraction, DrawingMode};
pub use drawing_manager::DrawingManager;
pub use indicator_renderer::IndicatorRenderer;
pub use layers::DrawLayer;
pub use series_renderer::{Rect, SeriesHit, SeriesRenderer};
