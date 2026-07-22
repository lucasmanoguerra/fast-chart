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
//! use fc_app::theme::{ChartTheme, ThemeToken, Rgba};
//!
//! let mut theme = ChartTheme::dark();
//! // Initial config — done via builder or presets
//!
//! // Hot-swap at runtime
//! theme.set_color(ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0));
//! assert_eq!(theme.get_color(ThemeToken::Bullish), Rgba::rgb(0.0, 1.0, 0.0));
//! ```

// Re-export everything from fc-theme
pub use fc_theme::*;
