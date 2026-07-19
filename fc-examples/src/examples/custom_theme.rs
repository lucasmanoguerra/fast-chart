//! Theme customization: dark → light → custom colors with hot-swap.
//!
//! Demonstrates: `ChartTheme` presets, `ChartThemeBuilder`, token-based
//! color overrides, and the `ThemeHandle` for thread-safe hot-swap.

use fc_core::theme::{ChartTheme, ChartThemeBuilder, Rgba, ThemeHandle, ThemeToken};

pub fn run() {
    // 1. Start from the dark preset
    let dark = ChartTheme::dark();
    println!("Dark background: {:?}", dark.background);

    // 2. Switch to light
    let light = ChartTheme::light();
    println!("Light background: {:?}", light.background);

    // 3. Build a fully custom theme from dark
    let custom = ChartThemeBuilder::from_theme(ChartTheme::dark())
        .with("background", Rgba::rgb(0.05, 0.05, 0.1))
        .with_token(ThemeToken::Bullish, Rgba::rgb(0.0, 0.9, 0.4))
        .with_token(ThemeToken::Bearish, Rgba::rgb(0.9, 0.2, 0.2))
        .with_token(ThemeToken::CrosshairLine, Rgba::rgb(1.0, 1.0, 0.0))
        .build();

    println!("Custom background: {:?}", custom.background);
    println!("Custom bullish:    {:?}", custom.bullish);

    // 4. Hot-swap at runtime via ThemeHandle (thread-safe)
    let handle = ThemeHandle::new(dark);
    handle.set_color(ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0));
    handle.set_color(ThemeToken::Bearish, Rgba::rgb(1.0, 0.0, 0.0));
    let snap = handle.snapshot();
    println!("After hot-swap bullish: {:?}", snap.bullish);

    // 5. Individual token updates
    handle.set_color(ThemeToken::GridLine, Rgba::rgb(0.3, 0.3, 0.3));
    handle.set_color(ThemeToken::TextPrimary, Rgba::rgb(0.9, 0.9, 0.9));
    let snap = handle.snapshot();
    println!("After update grid: {:?}", snap.grid_line);
    println!("After update text: {:?}", snap.text_primary);
}
