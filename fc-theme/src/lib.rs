//! Chart theming system with design tokens.
//!
//! Built-in themes: [`ChartTheme::dark`], [`ChartTheme::light`].
//! Custom themes via [`ChartThemeBuilder`].
//!
//! # Hot-swap
//!
//! Colors can be changed at runtime via [`ChartTheme::set_color`] and
//! [`ChartTheme::get_color`]. For thread-safe sharing, use [`ThemeHandle`].
//!
//! ```rust
//! use fc_theme::{ChartTheme, ThemeToken, Rgba};
//!
//! let mut theme = ChartTheme::dark();
//! // Initial config — done via builder or presets
//!
//! // Hot-swap at runtime
//! theme.set_color(ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0));
//! assert_eq!(theme.get_color(ThemeToken::Bullish), Rgba::rgb(0.0, 1.0, 0.0));
//! ```

use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

// Re-export Rgba from fc-primitives (canonical location)
pub use fc_primitives::color::Rgba;

// Re-export LineStyle from fc-primitives (canonical location)
pub use fc_primitives::LineStyle;

// ---------------------------------------------------------------------------
// ThemeToken — type-safe token identifiers for hot-swap
// ---------------------------------------------------------------------------

/// Type-safe token identifier for theme colors.
///
/// Used with [`ChartTheme::set_color`] and [`ChartTheme::get_color`] for
/// runtime color changes without rebuilding the entire theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeToken {
    // Background
    Background,
    PaneBackground,

    // Grid
    GridLine,

    // Text
    TextPrimary,
    TextSecondary,

    // Price scale
    PriceScaleBackground,
    PriceScaleText,
    PriceScaleBorder,

    // Time scale
    TimeScaleBackground,
    TimeScaleText,
    TimeScaleBorder,

    // Series
    Bullish,
    BullishFill,
    Bearish,
    BearishFill,
    LineColor,
    AreaFill,
    VolumeBullish,
    VolumeBearish,

    // Crosshair
    CrosshairLine,
    CrosshairLabelBg,
    CrosshairLabelText,

    // Selection
    SelectionBorder,
    SelectionFill,

    // Hover
    HoverBorder,

    // Markers
    MarkerUp,
    MarkerDown,
    MarkerNeutral,

    // Drawings
    DrawingLine,
    DrawingFill,
    DrawingText,

    // Divider
    Divider,

    // Watermark
    Watermark,
}

// ---------------------------------------------------------------------------
// ChartTheme
// ---------------------------------------------------------------------------

/// Complete chart theme with all design tokens.
///
/// Colors can be configured at init (via builder or presets) and changed at
/// runtime via [`set_color`](ChartTheme::set_color) for hot-swap.
///
/// # Examples
///
/// ```
/// use fc_theme::{ChartTheme, ThemeToken, Rgba};
///
/// let mut dark = ChartTheme::dark();
/// let light = ChartTheme::light();
/// assert_ne!(dark.background, light.background);
///
/// dark.set_color(ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0));
/// assert_eq!(dark.get_color(ThemeToken::Bullish), Rgba::rgb(0.0, 1.0, 0.0));
/// ```
#[derive(Debug, Clone)]
pub struct ChartTheme {
    // Background
    pub background: Rgba,
    pub pane_background: Rgba,

    // Grid
    pub grid_line: Rgba,
    pub grid_line_style: LineStyle,

    // Text
    pub text_primary: Rgba,
    pub text_secondary: Rgba,
    pub text_font_size: f64,

    // Price scale
    pub price_scale_background: Rgba,
    pub price_scale_text: Rgba,
    pub price_scale_border: Rgba,

    // Time scale
    pub time_scale_background: Rgba,
    pub time_scale_text: Rgba,
    pub time_scale_border: Rgba,

    // Series
    pub bullish: Rgba,
    pub bullish_fill: Rgba,
    pub bearish: Rgba,
    pub bearish_fill: Rgba,
    pub line_color: Rgba,
    pub area_fill: Rgba,
    pub volume_bullish: Rgba,
    pub volume_bearish: Rgba,

    // Crosshair
    pub crosshair_line: Rgba,
    pub crosshair_label_bg: Rgba,
    pub crosshair_label_text: Rgba,

    // Selection
    pub selection_border: Rgba,
    pub selection_fill: Rgba,

    // Hover
    pub hover_border: Rgba,

    // Markers
    pub marker_up: Rgba,
    pub marker_down: Rgba,
    pub marker_neutral: Rgba,

    // Drawings
    pub drawing_line: Rgba,
    pub drawing_fill: Rgba,
    pub drawing_text: Rgba,

    // Divider
    pub divider: Rgba,

    // Watermark
    pub watermark: Rgba,
}

// ---------------------------------------------------------------------------
// Hot-swap API — generated from single table via macro
// ---------------------------------------------------------------------------
//
// To add a new token:
//   1. Add variant to ThemeToken enum
//   2. Add `pub field_name: Rgba` to ChartTheme struct
//   3. Add one line below: (Variant, field_name, "string_name")
//
// set_color, get_color, and parse_token are all generated from this table.

macro_rules! theme_color_map {
    ( $( ($variant:ident, $field:ident, $str:literal) ),* $(,)? ) => {
        impl ChartTheme {
            /// Set a color by token at runtime (hot-swap).
            pub fn set_color(&mut self, token: ThemeToken, color: Rgba) {
                match token {
                    $(ThemeToken::$variant => self.$field = color,)*
                }
            }

            /// Get a color by token.
            pub fn get_color(&self, token: ThemeToken) -> Rgba {
                match token {
                    $(ThemeToken::$variant => self.$field,)*
                }
            }

            /// Batch set multiple colors at once (hot-swap).
            pub fn set_colors(&mut self, updates: &[(ThemeToken, Rgba)]) {
                for &(token, color) in updates {
                    self.set_color(token, color);
                }
            }
        }

        /// Parse a token name string into a `ThemeToken`.
        #[inline]
        pub fn parse_token(name: &str) -> Option<ThemeToken> {
            Some(match name {
                $($str => ThemeToken::$variant,)*
                _ => return None,
            })
        }
    };
}

theme_color_map! {
    (Background,       background,         "background"),
    (PaneBackground,   pane_background,    "pane_background"),
    (GridLine,         grid_line,          "grid_line"),
    (TextPrimary,      text_primary,       "text_primary"),
    (TextSecondary,    text_secondary,     "text_secondary"),
    (PriceScaleBackground, price_scale_background, "price_scale_background"),
    (PriceScaleText,   price_scale_text,   "price_scale_text"),
    (PriceScaleBorder, price_scale_border, "price_scale_border"),
    (TimeScaleBackground, time_scale_background, "time_scale_background"),
    (TimeScaleText,    time_scale_text,    "time_scale_text"),
    (TimeScaleBorder,  time_scale_border,  "time_scale_border"),
    (Bullish,          bullish,            "bullish"),
    (BullishFill,      bullish_fill,       "bullish_fill"),
    (Bearish,          bearish,            "bearish"),
    (BearishFill,      bearish_fill,       "bearish_fill"),
    (LineColor,        line_color,         "line_color"),
    (AreaFill,         area_fill,          "area_fill"),
    (VolumeBullish,    volume_bullish,     "volume_bullish"),
    (VolumeBearish,    volume_bearish,     "volume_bearish"),
    (CrosshairLine,    crosshair_line,     "crosshair_line"),
    (CrosshairLabelBg, crosshair_label_bg, "crosshair_label_bg"),
    (CrosshairLabelText, crosshair_label_text, "crosshair_label_text"),
    (SelectionBorder,  selection_border,   "selection_border"),
    (SelectionFill,    selection_fill,     "selection_fill"),
    (HoverBorder,      hover_border,       "hover_border"),
    (MarkerUp,         marker_up,          "marker_up"),
    (MarkerDown,       marker_down,        "marker_down"),
    (MarkerNeutral,    marker_neutral,     "marker_neutral"),
    (DrawingLine,      drawing_line,       "drawing_line"),
    (DrawingFill,      drawing_fill,       "drawing_fill"),
    (DrawingText,      drawing_text,       "drawing_text"),
    (Divider,          divider,            "divider"),
    (Watermark,        watermark,          "watermark"),
}

// ---------------------------------------------------------------------------
// Built-in themes
// ---------------------------------------------------------------------------

impl ChartTheme {
    /// Dark theme (default for trading terminals).
    pub fn dark() -> Self {
        Self {
            background: Rgba::from_hex(0x1A1A2EFF),
            pane_background: Rgba::from_hex(0x16213EFF),

            grid_line: Rgba::from_hex(0xFFFFFF1A),
            grid_line_style: LineStyle::Solid,

            text_primary: Rgba::from_hex(0xE0E0E0FF),
            text_secondary: Rgba::from_hex(0xA0A0A0FF),
            text_font_size: 12.0,

            price_scale_background: Rgba::from_hex(0x1A1A2EFF),
            price_scale_text: Rgba::from_hex(0xA0A0A0FF),
            price_scale_border: Rgba::from_hex(0xFFFFFF1A),

            time_scale_background: Rgba::from_hex(0x1A1A2EFF),
            time_scale_text: Rgba::from_hex(0xA0A0A0FF),
            time_scale_border: Rgba::from_hex(0xFFFFFF1A),

            bullish: Rgba::from_hex(0x26A69AFF),
            bullish_fill: Rgba::from_hex(0x26A69A40),
            bearish: Rgba::from_hex(0xEF5350FF),
            bearish_fill: Rgba::from_hex(0xEF535040),
            line_color: Rgba::from_hex(0x42A5F5FF),
            area_fill: Rgba::from_hex(0x42A5F520),
            volume_bullish: Rgba::from_hex(0x26A69A40),
            volume_bearish: Rgba::from_hex(0xEF535040),

            crosshair_line: Rgba::from_hex(0xFFFFFF60),
            crosshair_label_bg: Rgba::from_hex(0x42A5F5FF),
            crosshair_label_text: Rgba::from_hex(0xFFFFFFFF),

            selection_border: Rgba::from_hex(0x42A5F5FF),
            selection_fill: Rgba::from_hex(0x42A5F520),

            hover_border: Rgba::from_hex(0x42A5F580),

            marker_up: Rgba::from_hex(0x26A69AFF),
            marker_down: Rgba::from_hex(0xEF5350FF),
            marker_neutral: Rgba::from_hex(0xA0A0A0FF),

            drawing_line: Rgba::from_hex(0x42A5F5FF),
            drawing_fill: Rgba::from_hex(0x42A5F520),
            drawing_text: Rgba::from_hex(0xE0E0E0FF),

            divider: Rgba::from_hex(0xFFFFFF1A),

            watermark: Rgba::from_hex(0xFFFFFF12),
        }
    }

    /// Light theme.
    pub fn light() -> Self {
        Self {
            background: Rgba::from_hex(0xFFFFFFFF),
            pane_background: Rgba::from_hex(0xF8F9FAFF),

            grid_line: Rgba::from_hex(0x00000012),
            grid_line_style: LineStyle::Solid,

            text_primary: Rgba::from_hex(0x333333FF),
            text_secondary: Rgba::from_hex(0x666666FF),
            text_font_size: 12.0,

            price_scale_background: Rgba::from_hex(0xF8F9FAFF),
            price_scale_text: Rgba::from_hex(0x666666FF),
            price_scale_border: Rgba::from_hex(0x00000012),

            time_scale_background: Rgba::from_hex(0xF8F9FAFF),
            time_scale_text: Rgba::from_hex(0x666666FF),
            time_scale_border: Rgba::from_hex(0x00000012),

            bullish: Rgba::from_hex(0x26A69AFF),
            bullish_fill: Rgba::from_hex(0x26A69A40),
            bearish: Rgba::from_hex(0xEF5350FF),
            bearish_fill: Rgba::from_hex(0xEF535040),
            line_color: Rgba::from_hex(0x1976D2FF),
            area_fill: Rgba::from_hex(0x1976D220),
            volume_bullish: Rgba::from_hex(0x26A69A40),
            volume_bearish: Rgba::from_hex(0xEF535040),

            crosshair_line: Rgba::from_hex(0x00000040),
            crosshair_label_bg: Rgba::from_hex(0x1976D2FF),
            crosshair_label_text: Rgba::from_hex(0xFFFFFFFF),

            selection_border: Rgba::from_hex(0x1976D2FF),
            selection_fill: Rgba::from_hex(0x1976D220),

            hover_border: Rgba::from_hex(0x1976D280),

            marker_up: Rgba::from_hex(0x26A69AFF),
            marker_down: Rgba::from_hex(0xEF5350FF),
            marker_neutral: Rgba::from_hex(0x666666FF),

            drawing_line: Rgba::from_hex(0x1976D2FF),
            drawing_fill: Rgba::from_hex(0x1976D220),
            drawing_text: Rgba::from_hex(0x333333FF),

            divider: Rgba::from_hex(0x00000012),

            watermark: Rgba::from_hex(0x00000008),
        }
    }

    /// Apply a preset: "dark" or "light".
    pub fn preset(name: &str) -> Self {
        match name {
            "light" => Self::light(),
            _ => Self::dark(),
        }
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Builder for custom themes — initial configuration.
///
/// # Examples
///
/// ```
/// use fc_theme::{ChartThemeBuilder, ThemeToken, Rgba};
///
/// let theme = ChartThemeBuilder::new()
///     .with("background", Rgba::rgb(0.1, 0.1, 0.1))
///     .with_token(ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0))
///     .build();
///
/// assert_eq!(theme.background, Rgba::rgb(0.1, 0.1, 0.1));
/// assert_eq!(theme.bullish, Rgba::rgb(0.0, 1.0, 0.0));
/// ```
pub struct ChartThemeBuilder {
    theme: ChartTheme,
    overrides: HashMap<ThemeToken, Rgba>,
}

impl ChartThemeBuilder {
    /// Create a new builder starting from the dark theme.
    pub fn new() -> Self {
        Self {
            theme: ChartTheme::dark(),
            overrides: HashMap::new(),
        }
    }

    /// Create a builder starting from an existing theme.
    pub fn from_theme(theme: ChartTheme) -> Self {
        Self {
            theme,
            overrides: HashMap::new(),
        }
    }

    /// Override a specific token by name (string).
    #[inline]
    pub fn with(mut self, name: &str, color: Rgba) -> Self {
        if let Some(token) = parse_token(name) {
            self.overrides.insert(token, color);
        }
        self
    }

    /// Override a specific token (type-safe).
    #[inline]
    pub fn with_token(mut self, token: ThemeToken, color: Rgba) -> Self {
        self.overrides.insert(token, color);
        self
    }

    /// Build the final theme, applying any overrides.
    pub fn build(mut self) -> ChartTheme {
        for (token, color) in self.overrides {
            self.theme.set_color(token, color);
        }
        self.theme
    }
}

impl Default for ChartThemeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ThemeError
// ---------------------------------------------------------------------------

/// Errors that can occur when accessing a shared theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeError {
    /// The RwLock protecting the theme was poisoned (a holder panicked).
    LockPoisoned,
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeError::LockPoisoned => write!(f, "theme lock poisoned"),
        }
    }
}

impl std::error::Error for ThemeError {}

// ---------------------------------------------------------------------------
// ThemeHandle — thread-safe shared theme for hot-swap
// ---------------------------------------------------------------------------

/// Thread-safe handle to a shared theme.
///
/// Allows the UI thread and renderer to share the same theme. Call
/// [`set`](ThemeHandle::set) to hot-swap the entire theme, or
/// [`set_color`](ThemeHandle::set_color) for individual tokens.
///
/// ```rust
/// use fc_theme::{ThemeHandle, ChartTheme, ThemeToken, Rgba};
///
/// let handle = ThemeHandle::new(ChartTheme::dark());
/// // From UI thread — hot-swap a single color
/// handle.set_color(ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0));
/// // From renderer — read the current theme
/// let theme = handle.read().unwrap();
/// assert_eq!(theme.get_color(ThemeToken::Bullish), Rgba::rgb(0.0, 1.0, 0.0));
/// ```
pub struct ThemeHandle {
    inner: Arc<RwLock<ChartTheme>>,
}

impl ThemeHandle {
    /// Create a new handle wrapping a theme.
    pub fn new(theme: ChartTheme) -> Self {
        Self {
            inner: Arc::new(RwLock::new(theme)),
        }
    }

    /// Hot-swap the entire theme.
    pub fn set(&self, theme: ChartTheme) {
        if let Ok(mut guard) = self.inner.write() {
            *guard = theme;
        }
    }

    /// Hot-swap a single color by token.
    pub fn set_color(&self, token: ThemeToken, color: Rgba) {
        if let Ok(mut guard) = self.inner.write() {
            guard.set_color(token, color);
        }
    }

    /// Read the current theme (blocks if write in progress).
    ///
    /// Returns an error if the lock is poisoned (a previous holder panicked
    /// while holding the write guard).
    pub fn read(&self) -> Result<RwLockReadGuard<'_, ChartTheme>, ThemeError> {
        self.inner.read().map_err(|_| ThemeError::LockPoisoned)
    }

    /// Write access to the current theme.
    ///
    /// Returns an error if the lock is poisoned.
    pub fn write(&self) -> Result<RwLockWriteGuard<'_, ChartTheme>, ThemeError> {
        self.inner.write().map_err(|_| ThemeError::LockPoisoned)
    }

    /// Clone the current theme snapshot.
    ///
    /// Returns the default dark theme if the lock is poisoned.
    pub fn snapshot(&self) -> ChartTheme {
        self.read()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| ChartTheme::dark())
    }
}

impl Clone for ThemeHandle {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl std::fmt::Debug for ThemeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let theme = self.snapshot();
        f.debug_struct("ThemeHandle")
            .field("background", &theme.background)
            .finish()
    }
}


// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Clasificación: determinística — verifica dark_theme_defaults
    #[test]
    fn dark_theme_defaults() {
        let t = ChartTheme::dark();
        assert_ne!(t.background, Rgba::default());
        assert_ne!(t.pane_background, Rgba::default());
        assert_ne!(t.bullish, t.bearish);
        assert_ne!(t.bullish_fill, t.bearish_fill);
        assert!(t.text_primary.3 > 0.0);
        assert!(t.text_secondary.3 > 0.0);
    }

    // Clasificación: determinística — verifica light_theme_defaults
    #[test]
    fn light_theme_defaults() {
        let t = ChartTheme::light();
        assert_ne!(t.background, Rgba::default());
        assert_ne!(t.bullish, t.bearish);
    }

    // Clasificación: determinística — verifica que dark y light son distinguibles
    #[test]
    fn dark_and_light_differ() {
        let d = ChartTheme::dark();
        let l = ChartTheme::light();
        assert_ne!(d.background, l.background);
        assert_ne!(d.pane_background, l.pane_background);
        assert_ne!(d.text_primary, l.text_primary);
        assert_ne!(d.grid_line, l.grid_line);
        assert_ne!(d.crosshair_line, l.crosshair_line);
        assert_ne!(d.watermark, l.watermark);
    }

    // Clasificación: determinística — verifica retorno de tema por nombre de preset
    #[test]
    fn preset_dark() {
        let t = ChartTheme::preset("dark");
        assert_eq!(t.background, ChartTheme::dark().background);
    }

    // Clasificación: determinística — verifica retorno de tema por nombre de preset
    #[test]
    fn preset_light() {
        let t = ChartTheme::preset("light");
        assert_eq!(t.background, ChartTheme::light().background);
    }

    // Clasificación: determinística — edge case: preset desconocido fallback a dark
    #[test]
    fn preset_unknown_falls_back_to_dark() {
        let t = ChartTheme::preset("unknown");
        assert_eq!(t.background, ChartTheme::dark().background);
    }


    // Clasificación: determinística — verifica que build() produce tema completo
    #[test]
    fn builder_new_starts_with_dark() {
        let b = ChartThemeBuilder::new();
        let theme = b.build();
        let dark = ChartTheme::dark();
        assert_eq!(theme.background, dark.background);
    }

    // Clasificación: determinística — verifica que build() produce tema completo
    #[test]
    fn builder_override_string() {
        let red = Rgba::new(1.0, 0.0, 0.0, 1.0);
        let theme = ChartThemeBuilder::new()
            .with("background", red)
            .build();
        assert_eq!(theme.background, red);
    }

    // Clasificación: determinística — verifica que build() produce tema completo
    #[test]
    fn builder_override_type_safe() {
        let green = Rgba::new(0.0, 1.0, 0.0, 1.0);
        let theme = ChartThemeBuilder::new()
            .with_token(ThemeToken::Bullish, green)
            .build();
        assert_eq!(theme.bullish, green);
    }

    // Clasificación: determinística — verifica que build() produce tema completo
    #[test]
    fn builder_from_theme() {
        let custom = ChartTheme::light();
        let theme = ChartThemeBuilder::from_theme(custom.clone())
            .with("background", Rgba::new(0.0, 0.0, 0.0, 1.0))
            .build();
        assert_eq!(theme.background, Rgba::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(theme.bullish, custom.bullish);
    }

    // Clasificación: determinística — verifica que build() produce tema completo
    #[test]
    fn builder_build_produces_complete_theme() {
        let theme = ChartThemeBuilder::new().build();
        assert!(!theme.background.0.is_nan());
        assert!(!theme.pane_background.0.is_nan());
        assert!(!theme.grid_line.0.is_nan());
        assert!(!theme.text_primary.0.is_nan());
        assert!(!theme.bullish.0.is_nan());
        assert!(!theme.bearish.0.is_nan());
        assert!(!theme.crosshair_line.0.is_nan());
        assert!(!theme.watermark.0.is_nan());
    }

    // Clasificación: determinística — verifica theme_clone
    #[test]
    fn theme_clone() {
        let t1 = ChartTheme::dark();
        let t2 = t1.clone();
        assert_eq!(t1.background, t2.background);
        assert_eq!(t1.bullish, t2.bullish);
        assert_eq!(t1.bearish, t2.bearish);
    }

    // Clasificación: determinística — verifica line_style_variants
    #[test]
    fn line_style_variants() {
        assert_ne!(LineStyle::Solid, LineStyle::Dashed);
        assert_ne!(LineStyle::Dashed, LineStyle::Dotted);
        assert_ne!(LineStyle::Solid, LineStyle::Dotted);
    }

    // Clasificación: determinística — verifica que build() produce tema completo
    #[test]
    fn builder_unknown_token_is_ignored() {
        let theme = ChartThemeBuilder::new()
            .with("nonexistent_token", Rgba::new(1.0, 0.0, 0.0, 1.0))
            .build();
        let dark = ChartTheme::dark();
        assert_eq!(theme.background, dark.background);
    }


    // Clasificación: determinística — verifica hot-swap de color a través del handle
    #[test]
    fn hot_swap_set_color() {
        let mut theme = ChartTheme::dark();
        let original = theme.bullish;
        let new_color = Rgba::rgb(0.5, 0.5, 0.5);
        theme.set_color(ThemeToken::Bullish, new_color);
        assert_eq!(theme.get_color(ThemeToken::Bullish), new_color);
        assert_ne!(theme.get_color(ThemeToken::Bullish), original);
    }

    // Clasificación: determinística — verifica hot_swap_batch_set
    #[test]
    fn hot_swap_batch_set() {
        let mut theme = ChartTheme::dark();
        theme.set_colors(&[
            (ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0)),
            (ThemeToken::Bearish, Rgba::rgb(1.0, 0.0, 0.0)),
            (ThemeToken::Background, Rgba::rgb(0.1, 0.1, 0.1)),
        ]);
        assert_eq!(theme.get_color(ThemeToken::Bullish), Rgba::rgb(0.0, 1.0, 0.0));
        assert_eq!(theme.get_color(ThemeToken::Bearish), Rgba::rgb(1.0, 0.0, 0.0));
        assert_eq!(theme.get_color(ThemeToken::Background), Rgba::rgb(0.1, 0.1, 0.1));
    }

    // Clasificación: determinística — round-trip SET/GET para todos los tokens
    #[test]
    fn hot_swap_all_tokens_round_trip() {
        let mut theme = ChartTheme::dark();
        let tokens = [
            ThemeToken::Background, ThemeToken::PaneBackground,
            ThemeToken::GridLine, ThemeToken::TextPrimary, ThemeToken::TextSecondary,
            ThemeToken::PriceScaleBackground, ThemeToken::PriceScaleText,
            ThemeToken::PriceScaleBorder, ThemeToken::TimeScaleBackground,
            ThemeToken::TimeScaleText, ThemeToken::TimeScaleBorder,
            ThemeToken::Bullish, ThemeToken::BullishFill,
            ThemeToken::Bearish, ThemeToken::BearishFill,
            ThemeToken::LineColor, ThemeToken::AreaFill,
            ThemeToken::VolumeBullish, ThemeToken::VolumeBearish,
            ThemeToken::CrosshairLine, ThemeToken::CrosshairLabelBg,
            ThemeToken::CrosshairLabelText, ThemeToken::SelectionBorder,
            ThemeToken::SelectionFill, ThemeToken::HoverBorder,
            ThemeToken::MarkerUp, ThemeToken::MarkerDown, ThemeToken::MarkerNeutral,
            ThemeToken::DrawingLine, ThemeToken::DrawingFill, ThemeToken::DrawingText,
            ThemeToken::Divider, ThemeToken::Watermark,
        ];
        let test_color = Rgba::rgb(0.42, 0.42, 0.42);
        for &token in &tokens {
            theme.set_color(token, test_color);
            assert_eq!(theme.get_color(token), test_color, "token {:?} failed", token);
        }
    }


    // Clasificación: determinística — verifica handle_new_and_read
    #[test]
    fn handle_new_and_read() {
        let handle = ThemeHandle::new(ChartTheme::dark());
        let snap = handle.snapshot();
        assert_eq!(snap.background, ChartTheme::dark().background);
    }

    // Clasificación: determinística — verifica handle_hot_swap_color
    #[test]
    fn handle_hot_swap_color() {
        let handle = ThemeHandle::new(ChartTheme::dark());
        handle.set_color(ThemeToken::Bullish, Rgba::rgb(1.0, 1.0, 1.0));
        let snap = handle.snapshot();
        assert_eq!(snap.get_color(ThemeToken::Bullish), Rgba::rgb(1.0, 1.0, 1.0));
    }

    // Clasificación: determinística — verifica handle_hot_swap_entire_theme
    #[test]
    fn handle_hot_swap_entire_theme() {
        let handle = ThemeHandle::new(ChartTheme::dark());
        handle.set(ChartTheme::light());
        let snap = handle.snapshot();
        assert_eq!(snap.background, ChartTheme::light().background);
    }

    // Clasificación: determinística — verifica que clone comparte estado via Arc
    #[test]
    fn handle_clone_shares_state() {
        let h1 = ThemeHandle::new(ChartTheme::dark());
        let h2 = h1.clone();
        h1.set_color(ThemeToken::Bullish, Rgba::rgb(0.5, 0.5, 0.5));
        let snap = h2.snapshot();
        assert_eq!(snap.get_color(ThemeToken::Bullish), Rgba::rgb(0.5, 0.5, 0.5));
    }

    // Clasificación: determinística — verifica handle_write_access
    #[test]
    fn handle_write_access() {
        let handle = ThemeHandle::new(ChartTheme::dark());
        {
            let mut theme = handle.write().unwrap();
            theme.background = Rgba::rgb(0.0, 0.0, 0.0);
        }
        let snap = handle.snapshot();
        assert_eq!(snap.background, Rgba::rgb(0.0, 0.0, 0.0));
    }
}
