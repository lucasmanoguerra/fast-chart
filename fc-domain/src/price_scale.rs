// Price scale types for per-pane coordinate mapping.
//
// Each pane can hold multiple price scales (Left, Right, named Overlays),
// each maintaining its own value range and formatter for independent
// price-to-pixel mapping.
// ---------------------------------------------------------------------------
// PriceScaleId
// ---------------------------------------------------------------------------
/// Identifies a price scale within a pane.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PriceScaleId {
    Left,
    Right,
    Overlay(String),
}

impl PriceScaleId {
    /// Create an overlay identifier from a name string.
    pub fn overlay(name: &str) -> Self {
        Self::Overlay(name.to_string())
    }
}

// ---------------------------------------------------------------------------
// PriceScaleMode
// ---------------------------------------------------------------------------

/// Price scale mode — controls how price values map to pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum PriceScaleMode {
    /// Auto mode — scale adjusts automatically to visible data.
    Auto,
    /// Manual mode — user controls the visible range.
    Manual,
    /// Locked mode — range is fixed and cannot be changed by user or auto-fit.
    Locked,
    /// Linear scale (default). Equal price increments → equal pixel distances.
    #[default]
    Normal,
    /// Logarithmic scale. Useful for crypto and wide-range assets.
    Logarithmic,
    /// Percentage change from the first visible bar.
    Percentage,
    /// Indexed mode — prices shown as percentage change from index 0.
    Indexed,
    /// Inverted mode — higher prices at the bottom.
    Inverted,
}


// ---------------------------------------------------------------------------
// PriceScaleOptions
// ---------------------------------------------------------------------------

/// Configuration options for a single price scale.
#[derive(Debug, Clone)]
pub struct PriceScaleOptions {
    /// Whether the scale axis is drawn.
    pub visible: bool,
    /// When true the scale auto-fits to the visible data range.
    pub auto_scale: bool,
    /// Mapping mode (Normal, Logarithmic, Percentage).
    pub mode: PriceScaleMode,
    /// Extra padding fraction applied during auto-fit (0.0 – 0.1).
    /// Default: 0.05 (5 %).
    pub scale_offset: f64,
}

impl Default for PriceScaleOptions {
    fn default() -> Self {
        Self {
            visible: true,
            auto_scale: true,
            mode: PriceScaleMode::Normal,
            scale_offset: 0.05,
        }
    }
}

// ---------------------------------------------------------------------------
// PriceScale
// ---------------------------------------------------------------------------

/// A price scale that maps prices to y-coordinates within a pane.
///
/// Each scale maintains its own `value_min` / `value_max` range. When
/// `auto_scale` is enabled, calling [`auto_fit`] adjusts the range to
/// the visible data with configurable padding.
#[derive(Debug, Clone)]
pub struct PriceScale {
    pub id: PriceScaleId,
    pub options: PriceScaleOptions,
    pub value_min: f64,
    pub value_max: f64,
    /// Extra margin above the top of the range (in price units).
    pub margin_top: f64,
    /// Extra margin below the bottom of the range (in price units).
    pub margin_bottom: f64,
}

impl PriceScale {
    /// Create a new price scale with the given identity and default options.
    pub fn new(id: PriceScaleId, options: PriceScaleOptions) -> Self {
        Self {
            id,
            options,
            value_min: 0.0,
            value_max: 100.0,
            margin_top: 0.0,
            margin_bottom: 0.0,
        }
    }

    /// Apply auto-fit with `scale_offset` padding plus explicit margins.
    ///
    /// No-op when `auto_scale` is `false` or mode is `Locked`.
    pub fn auto_fit(&mut self, visible_data_min: f64, visible_data_max: f64) {
        if !self.options.auto_scale || self.options.mode == PriceScaleMode::Locked {
            return;
        }
        let range = visible_data_max - visible_data_min;
        if range.abs() < f64::EPSILON {
            self.value_min = visible_data_min - 1.0 - self.margin_bottom;
            self.value_max = visible_data_max + 1.0 + self.margin_top;
            return;
        }
        let pad = range * self.options.scale_offset;
        self.value_min = visible_data_min - pad - self.margin_bottom;
        self.value_max = visible_data_max + pad + self.margin_top;
    }

    /// Returns `true` when `price` falls within the current range.
    pub fn contains(&self, price: f64) -> bool {
        price >= self.value_min && price <= self.value_max
    }

    /// Change the scale mode.
    pub fn set_mode(&mut self, mode: PriceScaleMode) {
        self.options.mode = mode;
    }

    /// Returns `true` when the range can be modified (not Locked).
    pub fn is_editable(&self) -> bool {
        self.options.mode != PriceScaleMode::Locked
    }
}

// ---------------------------------------------------------------------------
// PriceFormatter trait + DefaultPriceFormatter
// ---------------------------------------------------------------------------

/// Trait for formatting price values to display strings.
pub trait PriceFormatter: Send + Sync {
    /// Full-precision format for axis labels (e.g. "105.20").
    fn format(&self, price: f64) -> String;

    /// Compact format for crosshair tooltips (e.g. "1.2K").
    fn format_short(&self, price: f64) -> String;
}

/// Default formatter with configurable decimal places.
///
/// When `decimal_places` is `None` the formatter auto-detects precision:
///   - prices >= 1.0  → 2 decimals
///   - prices < 0.01  → 5 decimals
///   - otherwise      → 4 decimals
#[derive(Debug, Clone)]
pub struct DefaultPriceFormatter {
    pub decimal_places: Option<usize>,
}

impl DefaultPriceFormatter {
    pub fn new(decimal_places: Option<usize>) -> Self {
        Self { decimal_places }
    }
}

impl Default for DefaultPriceFormatter {
    fn default() -> Self {
        Self::new(None)
    }
}

/// Auto-detect the appropriate number of decimal places for a price.
fn auto_decimals(price: f64) -> usize {
    if price >= 1.0 {
        2
    } else if price < 0.01 {
        5
    } else {
        4
    }
}

impl PriceFormatter for DefaultPriceFormatter {
    fn format(&self, price: f64) -> String {
        if price.is_nan() {
            return "NaN".to_string();
        }
        if price.is_infinite() {
            return if price > 0.0 { "∞" } else { "-∞" }.to_string();
        }
        let prec = self.decimal_places.unwrap_or_else(|| auto_decimals(price));
        format!("{:.prec$}", price, prec = prec)
    }

    fn format_short(&self, price: f64) -> String {
        if price.is_nan() {
            return "NaN".to_string();
        }
        if price.is_infinite() {
            return if price > 0.0 { "∞" } else { "-∞" }.to_string();
        }
        if price.abs() >= 1_000.0 {
            let k = price / 1_000.0;
            format!("{:.1}K", k)
        } else {
            let prec = self.decimal_places.unwrap_or_else(|| auto_decimals(price));
            format!("{:.prec$}", price, prec = prec)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;


    // Clasificación: determinística — verifica left_and_right_are_distinct
    #[test]
    fn left_and_right_are_distinct() {
        assert_ne!(PriceScaleId::Left, PriceScaleId::Right);
    }

    // Clasificación: determinística — verifica overlay_equality_by_name
    #[test]
    fn overlay_equality_by_name() {
        let a = PriceScaleId::Overlay("RSI".into());
        let b = PriceScaleId::Overlay("RSI".into());
        assert_eq!(a, b);
    }

    // Clasificación: determinística — verifica overlay_inequality_by_name
    #[test]
    fn overlay_inequality_by_name() {
        let a = PriceScaleId::Overlay("RSI".into());
        let b = PriceScaleId::Overlay("MACD".into());
        assert_ne!(a, b);
    }


    // Clasificación: determinística — verifica auto_fit_padds_range
    #[test]
    fn auto_fit_padds_range() {
        let mut scale = PriceScale::new(PriceScaleId::Right, PriceScaleOptions::default());
        scale.auto_fit(100.0, 200.0);
        // range = 100, pad = 5
        assert!((scale.value_min - 95.0).abs() < f64::EPSILON);
        assert!((scale.value_max - 205.0).abs() < f64::EPSILON);
    }

    // Clasificación: determinística — verifica auto_fit_disabled_noop
    #[test]
    fn auto_fit_disabled_noop() {
        let mut scale = PriceScale::new(
            PriceScaleId::Left,
            PriceScaleOptions {
                auto_scale: false,
                ..Default::default()
            },
        );
        scale.auto_fit(100.0, 200.0);
        assert!((scale.value_min - 0.0).abs() < f64::EPSILON);
        assert!((scale.value_max - 100.0).abs() < f64::EPSILON);
    }

    // Clasificación: determinística — verifica auto_fit_zero_range
    #[test]
    fn auto_fit_zero_range() {
        let mut scale = PriceScale::new(PriceScaleId::Right, PriceScaleOptions::default());
        scale.auto_fit(50.0, 50.0);
        assert!((scale.value_min - 49.0).abs() < f64::EPSILON);
        assert!((scale.value_max - 51.0).abs() < f64::EPSILON);
    }

    // Clasificación: determinística — verifica auto_fit_locked_noop
    #[test]
    fn auto_fit_locked_noop() {
        let mut scale = PriceScale::new(
            PriceScaleId::Right,
            PriceScaleOptions {
                mode: PriceScaleMode::Locked,
                ..Default::default()
            },
        );
        scale.value_min = 10.0;
        scale.value_max = 90.0;
        scale.auto_fit(100.0, 200.0);
        assert!((scale.value_min - 10.0).abs() < f64::EPSILON);
        assert!((scale.value_max - 90.0).abs() < f64::EPSILON);
    }

    // Clasificación: determinística — verifica auto_fit_with_margins
    #[test]
    fn auto_fit_with_margins() {
        let mut scale = PriceScale::new(PriceScaleId::Right, PriceScaleOptions::default());
        scale.margin_top = 10.0;
        scale.margin_bottom = 5.0;
        scale.auto_fit(100.0, 200.0);
        // range = 100, pad = 5, margin_top = 10, margin_bottom = 5
        assert!((scale.value_min - 90.0).abs() < f64::EPSILON);
        assert!((scale.value_max - 215.0).abs() < f64::EPSILON);
    }


    // Clasificación: determinística — verifica set_mode_changes_mode
    #[test]
    fn set_mode_changes_mode() {
        let mut scale = PriceScale::new(PriceScaleId::Right, PriceScaleOptions::default());
        assert_eq!(scale.options.mode, PriceScaleMode::Normal);
        scale.set_mode(PriceScaleMode::Logarithmic);
        assert_eq!(scale.options.mode, PriceScaleMode::Logarithmic);
    }

    // Clasificación: determinística — verifica is_editable_normal
    #[test]
    fn is_editable_normal() {
        let scale = PriceScale::new(PriceScaleId::Right, PriceScaleOptions::default());
        assert!(scale.is_editable());
    }

    // Clasificación: determinística — verifica is_editable_locked
    #[test]
    fn is_editable_locked() {
        let scale = PriceScale::new(
            PriceScaleId::Right,
            PriceScaleOptions {
                mode: PriceScaleMode::Locked,
                ..Default::default()
            },
        );
        assert!(!scale.is_editable());
    }


    // Clasificación: determinística — verifica all_modes_exist
    #[test]
    fn all_modes_exist() {
        let _ = PriceScaleMode::Auto;
        let _ = PriceScaleMode::Manual;
        let _ = PriceScaleMode::Locked;
        let _ = PriceScaleMode::Normal;
        let _ = PriceScaleMode::Logarithmic;
        let _ = PriceScaleMode::Percentage;
        let _ = PriceScaleMode::Indexed;
        let _ = PriceScaleMode::Inverted;
    }

    // Clasificación: determinística — verifica que el modo por defecto es Normal
    #[test]
    fn default_mode_is_normal() {
        assert_eq!(PriceScaleMode::default(), PriceScaleMode::Normal);
    }


    // Clasificación: determinística — verifica contains_inside_range
    #[test]
    fn contains_inside_range() {
        let scale = PriceScale {
            id: PriceScaleId::Left,
            options: PriceScaleOptions::default(),
            value_min: 10.0,
            value_max: 20.0,
            margin_top: 0.0,
            margin_bottom: 0.0,
        };
        assert!(scale.contains(15.0));
        assert!(scale.contains(10.0));
        assert!(scale.contains(20.0));
        assert!(!scale.contains(9.9));
        assert!(!scale.contains(20.1));
    }


    // Clasificación: determinística — verifica default_format_auto_detects_precision
    #[test]
    fn default_format_auto_detects_precision() {
        let fmt = DefaultPriceFormatter::default();
        // >= 1.0 → 2 decimals
        assert_eq!(fmt.format(105.2), "105.20");
        // < 0.01 → 5 decimals
        assert_eq!(fmt.format(0.00523), "0.00523");
        // else → 4 decimals
        assert_eq!(fmt.format(0.5), "0.5000");
    }

    // Clasificación: determinística — verifica explicit_format
    #[test]
    fn explicit_format() {
        let fmt = DefaultPriceFormatter::new(Some(4));
        assert_eq!(fmt.format(105.2), "105.2000");
    }

    // Clasificación: determinística — verifica format_nan
    #[test]
    fn format_nan() {
        let fmt = DefaultPriceFormatter::default();
        assert_eq!(fmt.format(f64::NAN), "NaN");
    }

    // Clasificación: determinística — verifica format_infinity
    #[test]
    fn format_infinity() {
        let fmt = DefaultPriceFormatter::default();
        assert_eq!(fmt.format(f64::INFINITY), "∞");
        assert_eq!(fmt.format(f64::NEG_INFINITY), "-∞");
    }

    // Clasificación: determinística — verifica format_short_uses_k_suffix
    #[test]
    fn format_short_uses_k_suffix() {
        let fmt = DefaultPriceFormatter::default();
        assert_eq!(fmt.format_short(1500.0), "1.5K");
        assert_eq!(fmt.format_short(12_345.0), "12.3K");
    }

    // Clasificación: determinística — verifica format_short_small_prices
    #[test]
    fn format_short_small_prices() {
        let fmt = DefaultPriceFormatter::default();
        assert_eq!(fmt.format_short(50.5), "50.50");
    }
}
