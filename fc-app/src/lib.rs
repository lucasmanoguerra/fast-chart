//! Application layer for the fast-chart trading library.
//!
//! Contains the [`ChartController`], ports, and layout management.
//! Users should depend on `fc-primitives`, `fc-domain`, `fc-render`, etc.
//! directly for domain types.

// ---------------------------------------------------------------------------
// Internal modules
// ---------------------------------------------------------------------------
#[cfg(feature = "animation")]
pub mod animation;
pub mod app;
pub mod builder;
pub mod cache;
#[cfg(feature = "file-watcher")]
pub mod config_watcher;
pub mod input;
pub mod ports;
pub mod render;
pub mod series;
pub mod theme;

// ---------------------------------------------------------------------------
// Application re-exports
// ---------------------------------------------------------------------------
pub use app::chart_controller::ChartController;
pub use app::chart_state::ChartState;
pub use app::frame_counter::FrameCounter;
pub use app::layout::{GridLayout, HorizontalSplit, LayoutEngine, VerticalStack};
pub use app::layout_manager::LayoutManager;
pub use app::pane::divider::PaneDivider;
pub use app::pane::Pane;
pub use app::viewport_management::ViewportManager;
pub use builder::ChartBuilder;

// ---------------------------------------------------------------------------
// Port re-exports
// ---------------------------------------------------------------------------
pub use ports::data_provider::{DataError, DataEvent, DataProvider};
pub use ports::interaction::{InteractionCommand, InteractionHandler, ViewportCommand};
pub use ports::render::{ChartRenderer, FrameState, RenderError};
