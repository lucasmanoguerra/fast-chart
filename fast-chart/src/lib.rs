//! # fast-chart
//!
//! Application layer for the fast-chart trading library.
//! Contains the ChartController, ports, and layout management.
//!
//! This crate bridges domain types with rendering and data-provider
//! adapters through well-defined port traits.
//!
//! `fast-chart` is the single gateway to domain types — the app
//! crate should never import `fc-types` directly.
//!
//! ```rust
//! use fast_chart::{
//!     // Render types (top-level re-exports)
//!     DrawCommand, DrawLayer, LineStyle as RenderLineStyle,
//!     RendererBackend, RenderContext, CoordinatePipeline,
//!     ScreenPoint, WorldPoint, Rect, SeriesHit, SeriesRenderer,
//!     // Domain types (gateway re-exports)
//!     Bar, Tick, Viewport, TimeSeries, LineStyle,
//! };
//! ```

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
// Domain re-exports (gateway pattern)
// App should never need to import fc-types directly
// ---------------------------------------------------------------------------
pub use fc_types::{
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

pub use fc_types::indicators::{Adx, Atr, Bollinger, Cci, Ema, HeikinAshi, Ichimoku, Kagi, Macd, ParabolicSar, Renko, Rsi, Sma, Stochastic, Supertrend, Vwap, WilliamsR};

// Drawing types
pub use fc_types::drawing::{
    Arrow, ChartPoint, DrawingId, Ellipse, FibonacciExtension, FibonacciRetracement,
    HorizontalLine, ImageDrawing, LabelDrawing, Path, Pitchfork, Rectangle, Segment, TextDrawing, TrendLine, VerticalLine,
};

// ---------------------------------------------------------------------------
// Primary re-exports — the ergonomic public API
// ---------------------------------------------------------------------------

pub use app::chart_controller::{ChartController, ChartState};
pub use app::frame_counter::FrameCounter;
pub use app::layout::{GridLayout, HorizontalSplit, LayoutEngine, VerticalStack};
pub use app::layout_manager::LayoutManager;
pub use app::pane::divider::{DividerCursor, PaneDivider};
pub use app::pane::Pane;
pub use app::viewport_management::ViewportManager;
pub use ports::data_provider::{DataEvent, DataProvider};
pub use ports::interaction::{InteractionCommand, InteractionHandler, ViewportCommand};
pub use ports::render::ChartRenderer;

// ---------------------------------------------------------------------------
// Render re-exports — the rendering contract API
// ---------------------------------------------------------------------------
pub use render::{
    CoordinatePipeline, DrawCommand, DrawLayer, Drawing, DrawingBounds, HitResult,
    RendererBackend, RenderContext, ScreenPoint, SeriesHit, SeriesRenderer, WorldPoint, Rect,
};
pub use render::LineStyle as RenderLineStyle;
