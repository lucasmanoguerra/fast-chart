//! ChartBuilder fluent API: construct chart configurations declaratively.
//!
//! Demonstrates: builder chaining, theme selection, dimensions, titles,
//! pane configuration, and runtime theme hot-swap via `theme_mut()`.

use fc_core::builder::{ChartBuilder, PaneConfig};
use fc_core::theme::{ChartTheme, Rgba, ThemeToken};

pub fn run() {
    // Minimal builder — defaults to dark theme, 800×600
    let minimal = ChartBuilder::new().build();
    println!(
        "Minimal: {}×{}, title={:?}",
        minimal.width,
        minimal.height,
        minimal.title.as_deref().unwrap_or("(none)")
    );

    // Full configuration with chained methods
    let config = ChartBuilder::new()
        .theme(ChartTheme::light())
        .title("AAPL Daily")
        .width(2560.0)
        .height(1440.0)
        .pane(|_| PaneConfig::new("price"))
        .pane(|_| PaneConfig::new("volume"))
        .pane(|_| PaneConfig::new("MACD"))
        .build();

    println!("\nFull config:");
    println!("  Title:   {}", config.title.as_deref().unwrap());
    println!("  Size:    {}×{}", config.width, config.height);
    println!("  Theme:   background={:?}", config.theme.background);
    println!("  Panes:   {}", config.panes.len());

    // Runtime theme hot-swap via theme_mut()
    let mut config = ChartBuilder::new().theme(ChartTheme::dark()).build();
    let dark_bg = config.theme.background;
    config
        .theme_mut()
        .set_color(ThemeToken::Background, Rgba::rgb(0.02, 0.02, 0.05));
    config
        .theme_mut()
        .set_color(ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0));
    println!("\nHot-swap:");
    println!("  background before: {:?}", dark_bg);
    println!("  background after:  {:?}", config.theme.background);
    println!("  bullish after:     {:?}", config.theme.bullish);
}
