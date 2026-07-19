//! The rendering engine for fast-chart financial charts.
//!
//! This crate defines the universal render primitives, layer system,
//! coordinate pipeline, render passes, and the core rendering traits.
//! It is backend-agnostic and produces [`DrawCommand`] values that can
//! be executed by any graphics adapter.

use smallvec::SmallVec;

// ---------------------------------------------------------------------------
// Submodule declarations
// ---------------------------------------------------------------------------

pub mod backend;
pub mod commands;
pub mod context;
pub mod coordinates;
pub mod dirty;
pub mod drawing;
pub mod drawing_interaction;
pub mod drawing_manager;
pub mod indicator_renderer;
pub mod layers;
pub mod passes;
pub mod pipeline;
pub mod pixel_perfect;
pub mod series_renderer;

// ---------------------------------------------------------------------------
// Public re-exports — the ergonomic API for this crate
// ---------------------------------------------------------------------------

pub use backend::RendererBackend;
pub use commands::{DrawCommand, LineStyle};
pub use context::RenderContext;
pub use coordinates::{CoordinatePipeline, ScreenPoint, WorldPoint};
pub use dirty::{DirtyRegion, DirtyRegionTracker, ScreenRect};
pub use drawing::{Drawing, DrawingBounds, HitResult};
pub use drawing_interaction::{DrawingAction, DrawingInteraction, DrawingMode};
pub use drawing_manager::DrawingManager;
pub use indicator_renderer::IndicatorRenderer;
pub use layers::DrawLayer;
pub use passes::{PassTracker, RenderPass};
pub use pipeline::{FrameStats, PassBatch, RenderPipeline, z_index_to_pass};
pub use series_renderer::{Rect, SeriesHit, SeriesRenderer};

/// A small, stack-allocated buffer for draw commands within a single pane.
///
/// Most panes produce fewer than 32 draw commands. `SmallVec` avoids heap
/// allocation for the common case while still supporting arbitrary counts.
pub type DrawBuffer = SmallVec<[DrawCommand; 32]>;
