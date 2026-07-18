//! Fluent builder API for chart construction.
//!
//! ```rust
//! use fast_chart::builder::ChartBuilder;
//! use fast_chart::theme::ChartTheme;
//!
//! let chart = ChartBuilder::new()
//!     .theme(ChartTheme::dark())
//!     .width(1920.0)
//!     .height(1080.0)
//!     .build();
//! ```

use crate::theme::ChartTheme;

/// A pane configuration during build.
#[derive(Debug, Clone)]
pub struct PaneConfig {
    pub name: String,
}

impl PaneConfig {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_owned() }
    }
}

/// Fluent builder for chart construction.
pub struct ChartBuilder {
    theme: ChartTheme,
    width: f64,
    height: f64,
    panes: Vec<PaneConfig>,
    title: Option<String>,
}

impl ChartBuilder {
    pub fn new() -> Self {
        Self {
            theme: ChartTheme::dark(),
            width: 800.0,
            height: 600.0,
            panes: Vec::new(),
            title: None,
        }
    }

    /// Set the chart theme.
    pub fn theme(mut self, theme: ChartTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Set the chart width.
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Set the chart height.
    pub fn height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    /// Set the chart title.
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_owned());
        self
    }

    /// Add a pane with a closure.
    pub fn pane(mut self, f: impl FnOnce(PaneConfig) -> PaneConfig) -> Self {
        let pane = PaneConfig::new("default");
        self.panes.push(f(pane));
        self
    }

    /// Build the chart configuration.
    pub fn build(self) -> ChartConfig {
        ChartConfig {
            theme: self.theme,
            width: self.width,
            height: self.height,
            panes: self.panes,
            title: self.title,
        }
    }
}

impl Default for ChartBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Built chart configuration.
#[derive(Debug, Clone)]
pub struct ChartConfig {
    pub theme: ChartTheme,
    pub width: f64,
    pub height: f64,
    pub panes: Vec<PaneConfig>,
    pub title: Option<String>,
}

impl ChartConfig {
    /// Access theme mutably for hot-swap.
    pub fn theme_mut(&mut self) -> &mut ChartTheme {
        &mut self.theme
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Rgba, ThemeToken};

    #[test]
    fn builder_new() {
        let builder = ChartBuilder::new();
        let config = builder.build();
        assert_eq!(config.width, 800.0);
        assert_eq!(config.height, 600.0);
        assert!(config.panes.is_empty());
        assert!(config.title.is_none());
        assert_eq!(config.theme.background, ChartTheme::dark().background);
    }

    #[test]
    fn builder_theme() {
        let light = ChartTheme::light();
        let config = ChartBuilder::new().theme(light.clone()).build();
        assert_eq!(config.theme.background, light.background);
    }

    #[test]
    fn builder_dimensions() {
        let config = ChartBuilder::new()
            .width(1920.0)
            .height(1080.0)
            .build();
        assert_eq!(config.width, 1920.0);
        assert_eq!(config.height, 1080.0);
    }

    #[test]
    fn builder_title() {
        let config = ChartBuilder::new().title("BTC/USDT").build();
        assert_eq!(config.title.as_deref(), Some("BTC/USDT"));
    }

    #[test]
    fn builder_pane() {
        let config = ChartBuilder::new()
            .pane(|p| PaneConfig { name: "candles".to_owned() })
            .build();
        assert_eq!(config.panes.len(), 1);
        assert_eq!(config.panes[0].name, "candles");
    }

    #[test]
    fn builder_chained() {
        let config = ChartBuilder::new()
            .theme(ChartTheme::light())
            .width(2560.0)
            .height(1440.0)
            .title("ETH/USDT")
            .pane(|p| PaneConfig { name: "volume".to_owned() })
            .build();
        assert_eq!(config.width, 2560.0);
        assert_eq!(config.height, 1440.0);
        assert_eq!(config.title.as_deref(), Some("ETH/USDT"));
        assert_eq!(config.panes.len(), 1);
        assert_eq!(config.panes[0].name, "volume");
        assert_eq!(config.theme.background, ChartTheme::light().background);
    }

    #[test]
    fn builder_build_produces_config() {
        let config = ChartBuilder::new().build();
        assert!(config.width > 0.0);
        assert!(config.height > 0.0);
        assert!(!config.theme.background.0.is_nan());
    }

    #[test]
    fn config_theme_mut() {
        let mut config = ChartBuilder::new().build();
        let original = config.theme.bullish;
        config.theme_mut().set_color(ThemeToken::Bullish, Rgba::rgb(0.5, 0.5, 0.5));
        assert_eq!(config.theme.bullish, Rgba::rgb(0.5, 0.5, 0.5));
        assert_ne!(config.theme.bullish, original);
    }
}
