//! Simple candlestick chart with a dark theme.
//!
//! Demonstrates: creating bars, choosing a theme, building the chart config,
//! and reading back the built configuration.

use fast_chart::builder::ChartBuilder;
use fast_chart::theme::ChartTheme;
use fast_chart::Bar;

pub fn run() {
    let theme = ChartTheme::dark();

    let config = ChartBuilder::new()
        .theme(theme)
        .title("BTC/USDT 1m")
        .width(1280.0)
        .height(720.0)
        .build();

    let mut bars = Vec::new();
    let mut prev_close = 50_000.0;
    for i in 0..100 {
        let ts = 1_700_000_000_000 + i * 60_000;
        let open = prev_close;
        let close = open + (i as f64 * 0.5 - 25.0);
        let high = open.max(close) + 10.0;
        let low = (open.min(close) - 10.0).max(0.01);
        let bar = Bar::new(ts, open, high, low, close, 5000).unwrap();
        prev_close = bar.close;
        bars.push(bar);
    }

    println!("Theme background: {:?}", config.theme.background);
    println!("Title: {:?}", config.title);
    println!("Bars created: {}", bars.len());
    println!("First bar: {:?}", bars.first().unwrap());
    println!("Last bar: {:?}", bars.last().unwrap());
}
