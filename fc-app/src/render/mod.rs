//! Rendering engine re-exported from `fc-render`.
//!
//! Rendering types, traits, and commands are defined in `fc-render` and
//! re-exported here so downstream crates (`fc-renderer-wgpu`, examples,
//! tests) can access them through `fc_app::render::*` without depending
//! directly on `fc-render`.

// ---------------------------------------------------------------------------
// Module re-exports — consumers access these as `fc_app::render::commands::…`
// ---------------------------------------------------------------------------

pub use fc_render::backend;
pub use fc_render::commands;
pub use fc_render::coordinates;
pub use fc_render::dirty;
pub use fc_render::drawing_interaction;
pub use fc_render::drawing_manager;
pub use fc_render::indicator_renderer;
pub use fc_render::layers;
pub use fc_render::passes;
pub use fc_render::pixel_perfect;
pub use fc_render::pipeline;
pub use fc_render::renderable_drawing;
pub use fc_render::series_renderer;

// ---------------------------------------------------------------------------
// Top-level re-exports — ergonomic access to the most-used types
// ---------------------------------------------------------------------------

pub use fc_render::{
    CoordinatePipeline, DirtyRegion, DirtyRegionTracker, DrawCommand, DrawLayer, DrawBuffer,
    DrawingAction, DrawingBounds, DrawingInteraction, DrawingMode, Drawing, FrameStats,
    HitResult, IndicatorRenderer, LineStyle, PassBatch, PassTracker, Rect, RenderContext,
    RenderPass, RenderPipeline, RenderableDrawing, RendererBackend, ScreenPoint, ScreenRect,
    SeriesHit, SeriesRenderer, WorldPoint, z_index_to_pass,
};

// ---------------------------------------------------------------------------
// Local session module (delegates to fc-sessions).
// ---------------------------------------------------------------------------

#[cfg(feature = "sessions")]
pub mod session;
