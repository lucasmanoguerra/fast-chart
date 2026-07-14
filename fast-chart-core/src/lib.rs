//! # fast-chart-core
//!
//! Application layer for the fast-chart trading library.
//! Contains the ChartController, ports, and layout management.
//!
//! This crate bridges domain types with rendering and data-provider
//! adapters through well-defined port traits.
//!
//! `fast-chart-core` is the single gateway to domain types — the app
//! crate should never import `fast-chart-domain` directly.

// ---------------------------------------------------------------------------
// Internal modules
// ---------------------------------------------------------------------------
pub mod app;
pub mod ports;

// ---------------------------------------------------------------------------
// Domain re-exports (gateway pattern)
// App should never need to import fast-chart-domain directly
// ---------------------------------------------------------------------------
pub use fast_chart_domain::{
    // Core data types
    Bar, Tick, Viewport,
    // Series
    TimeSeries, SeriesType,
    // Indicators
    Indicator,
    // Crosshair & Magnet
    Crosshair, MagnetMode,
    // Markers
    Marker, MarkerId, MarkerPosition, MarkerSet, MarkerShape,
    // Price lines
    PriceLine, PriceLineId, PriceLineSet, LabelPosition, LineStyle,
    // Price scales
    PriceScale, PriceScaleId, PriceScaleMode, PriceScaleOptions,
    DefaultPriceFormatter, PriceFormatter,
    // Invalidation
    InvalidationLevel, InvalidationMask, PaneBitmask,
    // Kinetic
    KineticScroll,
    // Localization
    EnglishLocalizer, SpanishLocalizer, Localizer,
    // Scales
    LinearScale, TimeScale,
    // Error
    ChartError,
};

pub use fast_chart_domain::indicators::{Bollinger, Ema, Ichimoku, Macd, Rsi, Sma, Stochastic};

// ---------------------------------------------------------------------------
// Primary re-exports — the ergonomic public API
// ---------------------------------------------------------------------------

pub use app::chart_controller::{ChartController, ChartState};
pub use app::frame_counter::FrameCounter;
pub use app::layout_manager::LayoutManager;
pub use app::pane::Pane;
pub use app::viewport_management::ViewportManager;
pub use ports::data_provider::{DataEvent, DataProvider};
pub use ports::interaction::{InteractionCommand, InteractionHandler, ViewportCommand};
pub use ports::render::ChartRenderer;
