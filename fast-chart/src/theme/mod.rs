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
//! use fast_chart::theme::{ChartTheme, ThemeToken, Rgba};
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

// ---------------------------------------------------------------------------
// Rgba
// ---------------------------------------------------------------------------

/// A named color as RGBA with channels in `[0.0, 1.0]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba(pub f64, pub f64, pub f64, pub f64);

impl Rgba {
    /// Create a new RGBA color.
    pub const fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self(r, g, b, a)
    }

    /// Fully opaque convenience constructor.
    pub const fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self(r, g, b, 1.0)
    }

    /// From `0xRRGGBBAA` hex integer.
    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 24) & 0xFF) as f64 / 255.0;
        let g = ((hex >> 16) & 0xFF) as f64 / 255.0;
        let b = ((hex >> 8) & 0xFF) as f64 / 255.0;
        let a = (hex & 0xFF) as f64 / 255.0;
        Self(r, g, b, a)
    }
}

impl Default for Rgba {
    fn default() -> Self {
        Self(0.0, 0.0, 0.0, 1.0)
    }
}

// ---------------------------------------------------------------------------
// LineStyle
// ---------------------------------------------------------------------------

/// Line rendering style for themed lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    /// Solid line.
    Solid,
    /// Dashed line.
    Dashed,
    /// Dotted line.
    Dotted,
}

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
// Hot-swap API
// ---------------------------------------------------------------------------

impl ChartTheme {
    /// Set a color by token at runtime (hot-swap).
    pub fn set_color(&mut self, token: ThemeToken, color: Rgba) {
        match token {
            ThemeToken::Background => self.background = color,
            ThemeToken::PaneBackground => self.pane_background = color,
            ThemeToken::GridLine => self.grid_line = color,
            ThemeToken::TextPrimary => self.text_primary = color,
            ThemeToken::TextSecondary => self.text_secondary = color,
            ThemeToken::PriceScaleBackground => self.price_scale_background = color,
            ThemeToken::PriceScaleText => self.price_scale_text = color,
            ThemeToken::PriceScaleBorder => self.price_scale_border = color,
            ThemeToken::TimeScaleBackground => self.time_scale_background = color,
            ThemeToken::TimeScaleText => self.time_scale_text = color,
            ThemeToken::TimeScaleBorder => self.time_scale_border = color,
            ThemeToken::Bullish => self.bullish = color,
            ThemeToken::BullishFill => self.bullish_fill = color,
            ThemeToken::Bearish => self.bearish = color,
            ThemeToken::BearishFill => self.bearish_fill = color,
            ThemeToken::LineColor => self.line_color = color,
            ThemeToken::AreaFill => self.area_fill = color,
            ThemeToken::VolumeBullish => self.volume_bullish = color,
            ThemeToken::VolumeBearish => self.volume_bearish = color,
            ThemeToken::CrosshairLine => self.crosshair_line = color,
            ThemeToken::CrosshairLabelBg => self.crosshair_label_bg = color,
            ThemeToken::CrosshairLabelText => self.crosshair_label_text = color,
            ThemeToken::SelectionBorder => self.selection_border = color,
            ThemeToken::SelectionFill => self.selection_fill = color,
            ThemeToken::HoverBorder => self.hover_border = color,
            ThemeToken::MarkerUp => self.marker_up = color,
            ThemeToken::MarkerDown => self.marker_down = color,
            ThemeToken::MarkerNeutral => self.marker_neutral = color,
            ThemeToken::DrawingLine => self.drawing_line = color,
            ThemeToken::DrawingFill => self.drawing_fill = color,
            ThemeToken::DrawingText => self.drawing_text = color,
            ThemeToken::Divider => self.divider = color,
            ThemeToken::Watermark => self.watermark = color,
        }
    }

    /// Get a color by token.
    pub fn get_color(&self, token: ThemeToken) -> Rgba {
        match token {
            ThemeToken::Background => self.background,
            ThemeToken::PaneBackground => self.pane_background,
            ThemeToken::GridLine => self.grid_line,
            ThemeToken::TextPrimary => self.text_primary,
            ThemeToken::TextSecondary => self.text_secondary,
            ThemeToken::PriceScaleBackground => self.price_scale_background,
            ThemeToken::PriceScaleText => self.price_scale_text,
            ThemeToken::PriceScaleBorder => self.price_scale_border,
            ThemeToken::TimeScaleBackground => self.time_scale_background,
            ThemeToken::TimeScaleText => self.time_scale_text,
            ThemeToken::TimeScaleBorder => self.time_scale_border,
            ThemeToken::Bullish => self.bullish,
            ThemeToken::BullishFill => self.bullish_fill,
            ThemeToken::Bearish => self.bearish,
            ThemeToken::BearishFill => self.bearish_fill,
            ThemeToken::LineColor => self.line_color,
            ThemeToken::AreaFill => self.area_fill,
            ThemeToken::VolumeBullish => self.volume_bullish,
            ThemeToken::VolumeBearish => self.volume_bearish,
            ThemeToken::CrosshairLine => self.crosshair_line,
            ThemeToken::CrosshairLabelBg => self.crosshair_label_bg,
            ThemeToken::CrosshairLabelText => self.crosshair_label_text,
            ThemeToken::SelectionBorder => self.selection_border,
            ThemeToken::SelectionFill => self.selection_fill,
            ThemeToken::HoverBorder => self.hover_border,
            ThemeToken::MarkerUp => self.marker_up,
            ThemeToken::MarkerDown => self.marker_down,
            ThemeToken::MarkerNeutral => self.marker_neutral,
            ThemeToken::DrawingLine => self.drawing_line,
            ThemeToken::DrawingFill => self.drawing_fill,
            ThemeToken::DrawingText => self.drawing_text,
            ThemeToken::Divider => self.divider,
            ThemeToken::Watermark => self.watermark,
        }
    }

    /// Batch set multiple colors at once (hot-swap).
    pub fn set_colors(&mut self, updates: &[(ThemeToken, Rgba)]) {
        for &(token, color) in updates {
            self.set_color(token, color);
        }
    }
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
    pub fn with(mut self, name: &str, color: Rgba) -> Self {
        if let Some(token) = parse_token(name) {
            self.overrides.insert(token, color);
        }
        self
    }

    /// Override a specific token (type-safe).
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

/// Parse a token name string into a `ThemeToken`.
fn parse_token(name: &str) -> Option<ThemeToken> {
    Some(match name {
        "background" => ThemeToken::Background,
        "pane_background" => ThemeToken::PaneBackground,
        "grid_line" => ThemeToken::GridLine,
        "text_primary" => ThemeToken::TextPrimary,
        "text_secondary" => ThemeToken::TextSecondary,
        "price_scale_background" => ThemeToken::PriceScaleBackground,
        "price_scale_text" => ThemeToken::PriceScaleText,
        "price_scale_border" => ThemeToken::PriceScaleBorder,
        "time_scale_background" => ThemeToken::TimeScaleBackground,
        "time_scale_text" => ThemeToken::TimeScaleText,
        "time_scale_border" => ThemeToken::TimeScaleBorder,
        "bullish" => ThemeToken::Bullish,
        "bullish_fill" => ThemeToken::BullishFill,
        "bearish" => ThemeToken::Bearish,
        "bearish_fill" => ThemeToken::BearishFill,
        "line_color" => ThemeToken::LineColor,
        "area_fill" => ThemeToken::AreaFill,
        "volume_bullish" => ThemeToken::VolumeBullish,
        "volume_bearish" => ThemeToken::VolumeBearish,
        "crosshair_line" => ThemeToken::CrosshairLine,
        "crosshair_label_bg" => ThemeToken::CrosshairLabelBg,
        "crosshair_label_text" => ThemeToken::CrosshairLabelText,
        "selection_border" => ThemeToken::SelectionBorder,
        "selection_fill" => ThemeToken::SelectionFill,
        "hover_border" => ThemeToken::HoverBorder,
        "marker_up" => ThemeToken::MarkerUp,
        "marker_down" => ThemeToken::MarkerDown,
        "marker_neutral" => ThemeToken::MarkerNeutral,
        "drawing_line" => ThemeToken::DrawingLine,
        "drawing_fill" => ThemeToken::DrawingFill,
        "drawing_text" => ThemeToken::DrawingText,
        "divider" => ThemeToken::Divider,
        "watermark" => ThemeToken::Watermark,
        _ => return None,
    })
}

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
/// use fast_chart::theme::{ThemeHandle, ChartTheme, ThemeToken, Rgba};
///
/// let handle = ThemeHandle::new(ChartTheme::dark());
/// // From UI thread — hot-swap a single color
/// handle.set_color(ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0));
/// // From renderer — read the current theme
/// let theme = handle.read();
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
    pub fn read(&self) -> RwLockReadGuard<'_, ChartTheme> {
        self.inner.read().expect("theme lock poisoned")
    }

    /// Write access to the current theme.
    pub fn write(&self) -> RwLockWriteGuard<'_, ChartTheme> {
        self.inner.write().expect("theme lock poisoned")
    }

    /// Clone the current theme snapshot.
    pub fn snapshot(&self) -> ChartTheme {
        self.read().clone()
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

    #[test]
    fn rgba_new() {
        let c = Rgba::new(0.1, 0.2, 0.3, 0.4);
        assert_eq!(c, Rgba(0.1, 0.2, 0.3, 0.4));
    }

    #[test]
    fn rgba_from_hex() {
        let c = Rgba::from_hex(0xFF0000FF);
        assert!((c.0 - 1.0).abs() < f64::EPSILON);
        assert!((c.1 - 0.0).abs() < f64::EPSILON);
        assert!((c.2 - 0.0).abs() < f64::EPSILON);
        assert!((c.3 - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rgba_rgb_convenience() {
        let c = Rgba::rgb(0.5, 0.6, 0.7);
        assert_eq!(c, Rgba(0.5, 0.6, 0.7, 1.0));
    }

    #[test]
    fn rgba_default_is_black_opaque() {
        let c = Rgba::default();
        assert_eq!(c, Rgba(0.0, 0.0, 0.0, 1.0));
    }

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

    #[test]
    fn light_theme_defaults() {
        let t = ChartTheme::light();
        assert_ne!(t.background, Rgba::default());
        assert_ne!(t.bullish, t.bearish);
    }

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

    #[test]
    fn preset_dark() {
        let t = ChartTheme::preset("dark");
        assert_eq!(t.background, ChartTheme::dark().background);
    }

    #[test]
    fn preset_light() {
        let t = ChartTheme::preset("light");
        assert_eq!(t.background, ChartTheme::light().background);
    }

    #[test]
    fn preset_unknown_falls_back_to_dark() {
        let t = ChartTheme::preset("unknown");
        assert_eq!(t.background, ChartTheme::dark().background);
    }

    // --- Builder tests ---

    #[test]
    fn builder_new_starts_with_dark() {
        let b = ChartThemeBuilder::new();
        let theme = b.build();
        let dark = ChartTheme::dark();
        assert_eq!(theme.background, dark.background);
    }

    #[test]
    fn builder_override_string() {
        let red = Rgba::new(1.0, 0.0, 0.0, 1.0);
        let theme = ChartThemeBuilder::new()
            .with("background", red)
            .build();
        assert_eq!(theme.background, red);
    }

    #[test]
    fn builder_override_type_safe() {
        let green = Rgba::new(0.0, 1.0, 0.0, 1.0);
        let theme = ChartThemeBuilder::new()
            .with_token(ThemeToken::Bullish, green)
            .build();
        assert_eq!(theme.bullish, green);
    }

    #[test]
    fn builder_from_theme() {
        let custom = ChartTheme::light();
        let theme = ChartThemeBuilder::from_theme(custom.clone())
            .with("background", Rgba::new(0.0, 0.0, 0.0, 1.0))
            .build();
        assert_eq!(theme.background, Rgba::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(theme.bullish, custom.bullish);
    }

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

    #[test]
    fn theme_clone() {
        let t1 = ChartTheme::dark();
        let t2 = t1.clone();
        assert_eq!(t1.background, t2.background);
        assert_eq!(t1.bullish, t2.bullish);
        assert_eq!(t1.bearish, t2.bearish);
    }

    #[test]
    fn line_style_variants() {
        assert_ne!(LineStyle::Solid, LineStyle::Dashed);
        assert_ne!(LineStyle::Dashed, LineStyle::Dotted);
        assert_ne!(LineStyle::Solid, LineStyle::Dotted);
    }

    #[test]
    fn rgba_equality() {
        assert_eq!(Rgba(0.1, 0.2, 0.3, 0.4), Rgba(0.1, 0.2, 0.3, 0.4));
        assert_ne!(Rgba(0.1, 0.2, 0.3, 0.4), Rgba(0.1, 0.2, 0.3, 0.5));
    }

    #[test]
    fn builder_unknown_token_is_ignored() {
        let theme = ChartThemeBuilder::new()
            .with("nonexistent_token", Rgba::new(1.0, 0.0, 0.0, 1.0))
            .build();
        let dark = ChartTheme::dark();
        assert_eq!(theme.background, dark.background);
    }

    // --- Hot-swap tests ---

    #[test]
    fn hot_swap_set_color() {
        let mut theme = ChartTheme::dark();
        let original = theme.bullish;
        let new_color = Rgba::rgb(0.5, 0.5, 0.5);
        theme.set_color(ThemeToken::Bullish, new_color);
        assert_eq!(theme.get_color(ThemeToken::Bullish), new_color);
        assert_ne!(theme.get_color(ThemeToken::Bullish), original);
    }

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

    // --- ThemeHandle tests ---

    #[test]
    fn handle_new_and_read() {
        let handle = ThemeHandle::new(ChartTheme::dark());
        let snap = handle.snapshot();
        assert_eq!(snap.background, ChartTheme::dark().background);
    }

    #[test]
    fn handle_hot_swap_color() {
        let handle = ThemeHandle::new(ChartTheme::dark());
        handle.set_color(ThemeToken::Bullish, Rgba::rgb(1.0, 1.0, 1.0));
        let snap = handle.snapshot();
        assert_eq!(snap.get_color(ThemeToken::Bullish), Rgba::rgb(1.0, 1.0, 1.0));
    }

    #[test]
    fn handle_hot_swap_entire_theme() {
        let handle = ThemeHandle::new(ChartTheme::dark());
        handle.set(ChartTheme::light());
        let snap = handle.snapshot();
        assert_eq!(snap.background, ChartTheme::light().background);
    }

    #[test]
    fn handle_clone_shares_state() {
        let h1 = ThemeHandle::new(ChartTheme::dark());
        let h2 = h1.clone();
        h1.set_color(ThemeToken::Bullish, Rgba::rgb(0.5, 0.5, 0.5));
        let snap = h2.snapshot();
        assert_eq!(snap.get_color(ThemeToken::Bullish), Rgba::rgb(0.5, 0.5, 0.5));
    }

    #[test]
    fn handle_write_access() {
        let handle = ThemeHandle::new(ChartTheme::dark());
        {
            let mut theme = handle.write();
            theme.background = Rgba::rgb(0.0, 0.0, 0.0);
        }
        let snap = handle.snapshot();
        assert_eq!(snap.background, Rgba::rgb(0.0, 0.0, 0.0));
    }
}
