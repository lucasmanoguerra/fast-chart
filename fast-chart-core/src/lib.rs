//! # fast-chart-core
//!
//! Application layer for the fast-chart trading library.
//! Contains the ChartController, ports, and layout management.
//!
//! This crate bridges domain types with rendering and data-provider
//! adapters through well-defined port traits.

// ---------------------------------------------------------------------------
// Internal modules
// ---------------------------------------------------------------------------
pub mod app;
pub mod ports;

// ---------------------------------------------------------------------------
// Primary re-exports — the ergonomic public API
// ---------------------------------------------------------------------------

pub use app::chart_controller::{ChartController, ChartState};
pub use app::layout_manager::LayoutManager;
pub use app::pane::Pane;
pub use app::viewport_management::ViewportManager;
pub use ports::data_provider::{DataEvent, DataProvider};
pub use ports::interaction::{InteractionCommand, InteractionHandler, ViewportCommand};
pub use ports::render::ChartRenderer;
