// ---------------------------------------------------------------------------
// DrawLayer — Z-index layer system for render ordering
// ---------------------------------------------------------------------------

use std::fmt;

/// Rendering layers ordered from back (lowest z-index) to front (highest).
///
/// Each layer has a fixed z-index range. Within a layer, draw commands
/// are sorted by their individual z-index. This ensures correct visual
/// ordering: grid lines behind candles behind crosshair behind tooltips.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawLayer {
    /// Background fill (z: 0–99)
    Background,
    /// Watermark text / logo (z: 100–199)
    Watermark,
    /// Grid lines (z: 200–299)
    Grid,
    /// Price scale axis (z: 300–399)
    PriceScale,
    /// Time scale axis (z: 400–499)
    TimeScale,
    /// Indicator overlays (z: 500–599)
    Indicators,
    /// Candlestick / OHLC bars (z: 600–699)
    Candles,
    /// Volume bars (z: 700–799)
    Volume,
    /// Custom series renderers (z: 800–899)
    CustomSeries,
    /// User drawing tools (z: 900–999)
    Drawings,
    /// Crosshair lines and labels (z: 1000–1099)
    Crosshair,
    /// Selection highlights (z: 1100–1199)
    Selection,
    /// Floating price labels (z: 1200–1299)
    FloatingLabels,
    /// Tooltip popups (z: 1300–1399)
    Tooltip,
    /// Cursor / mouse pointer (z: 1400–1499)
    Cursor,
}

impl DrawLayer {
    /// The z-index range start for this layer.
    pub fn z_start(self) -> i32 {
        match self {
            Self::Background => 0,
            Self::Watermark => 100,
            Self::Grid => 200,
            Self::PriceScale => 300,
            Self::TimeScale => 400,
            Self::Indicators => 500,
            Self::Candles => 600,
            Self::Volume => 700,
            Self::CustomSeries => 800,
            Self::Drawings => 900,
            Self::Crosshair => 1000,
            Self::Selection => 1100,
            Self::FloatingLabels => 1200,
            Self::Tooltip => 1300,
            Self::Cursor => 1400,
        }
    }

    /// The z-index range end (exclusive) for this layer.
    pub fn z_end(self) -> i32 {
        self.z_start() + 100
    }

    /// Midpoint z-index for this layer (convenience).
    pub fn z_mid(self) -> i32 {
        self.z_start() + 50
    }

    /// All layers in back-to-front order.
    pub fn all() -> &'static [DrawLayer] {
        &[
            Self::Background,
            Self::Watermark,
            Self::Grid,
            Self::PriceScale,
            Self::TimeScale,
            Self::Indicators,
            Self::Candles,
            Self::Volume,
            Self::CustomSeries,
            Self::Drawings,
            Self::Crosshair,
            Self::Selection,
            Self::FloatingLabels,
            Self::Tooltip,
            Self::Cursor,
        ]
    }

    /// Total number of layers.
    pub fn count() -> usize {
        Self::all().len()
    }
}

impl Default for DrawLayer {
    fn default() -> Self {
        Self::Candles
    }
}

impl fmt::Display for DrawLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Background => write!(f, "Background"),
            Self::Watermark => write!(f, "Watermark"),
            Self::Grid => write!(f, "Grid"),
            Self::PriceScale => write!(f, "PriceScale"),
            Self::TimeScale => write!(f, "TimeScale"),
            Self::Indicators => write!(f, "Indicators"),
            Self::Candles => write!(f, "Candles"),
            Self::Volume => write!(f, "Volume"),
            Self::CustomSeries => write!(f, "CustomSeries"),
            Self::Drawings => write!(f, "Drawings"),
            Self::Crosshair => write!(f, "Crosshair"),
            Self::Selection => write!(f, "Selection"),
            Self::FloatingLabels => write!(f, "FloatingLabels"),
            Self::Tooltip => write!(f, "Tooltip"),
            Self::Cursor => write!(f, "Cursor"),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_layers_have_unique_z_ranges() {
        let layers = DrawLayer::all();
        for (i, layer) in layers.iter().enumerate() {
            assert_eq!(
                layer.z_start(),
                i as i32 * 100,
                "{layer} z_start mismatch"
            );
            assert_eq!(layer.z_end(), layer.z_start() + 100, "{layer} z_end mismatch");
        }
    }

    #[test]
    fn layer_count() {
        assert_eq!(DrawLayer::count(), 15);
    }

    #[test]
    fn layers_are_ordered() {
        let layers = DrawLayer::all();
        for window in layers.windows(2) {
            assert!(
                window[0].z_end() <= window[1].z_start(),
                "overlapping layers: {} and {}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn default_is_candles() {
        assert_eq!(DrawLayer::default(), DrawLayer::Candles);
    }

    #[test]
    fn z_mid_is_center() {
        for &layer in DrawLayer::all() {
            assert_eq!(layer.z_mid(), layer.z_start() + 50, "{layer} z_mid mismatch");
        }
    }

    #[test]
    fn display_format() {
        assert_eq!(DrawLayer::Background.to_string(), "Background");
        assert_eq!(DrawLayer::Candles.to_string(), "Candles");
        assert_eq!(DrawLayer::Cursor.to_string(), "Cursor");
    }

    #[test]
    fn debug_format() {
        let dbg = format!("{:?}", DrawLayer::Grid);
        assert_eq!(dbg, "Grid");
    }

    #[test]
    fn hash_consistency() {
        use std::collections::HashSet;
        let set: HashSet<DrawLayer> = DrawLayer::all().iter().copied().collect();
        assert_eq!(set.len(), 15);
    }
}
