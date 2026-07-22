//! # fc-domain
//!
//! Domain types for the fast-chart trading library.
//! Depends on `fc-primitives` for fundamental building blocks (Bar, TimeSeries, scales, etc.)
//!
//! This crate provides chart-level domain logic:
//! crosshair magnet, drawing tools, indicator trait and implementations,
//! markers, price lines, price scales, and viewport management.

pub mod crosshair;
pub mod drawing;
pub mod indicator;
pub mod indicators;
pub mod marker;
pub mod price_line;
pub mod price_scale;
pub mod viewport;

// Re-exports
pub use crosshair::{Crosshair, MagnetMode};
pub use drawing::{
    Arrow, ChartPoint, Drawing, DrawingId, DrawingSet, Ellipse, FibonacciExtension,
    FibonacciRetracement, HorizontalLine, ImageDrawing, LabelDrawing, Path, Pitchfork, Rectangle,
    Segment, TextDrawing, TrendLine, VerticalLine,
};
pub use indicator::{Indicator, OverlayMode};
pub use marker::{Marker, MarkerId, MarkerPosition, MarkerSet, MarkerShape};
pub use price_line::{LabelPosition, LineStyle, PriceLine, PriceLineId, PriceLineSet};
pub use price_scale::{
    DefaultPriceFormatter, PriceFormatter, PriceScale, PriceScaleId, PriceScaleMode,
    PriceScaleOptions,
};
pub use viewport::Viewport;
