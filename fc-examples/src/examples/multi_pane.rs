//! Multi-pane chart: candles + volume + RSI stacked vertically.
//!
//! Demonstrates: configuring a chart with three panes using the builder API.

use fc_core::builder::{ChartBuilder, PaneConfig};
use fc_core::theme::ChartTheme;
use fc_core::Bar;

pub fn run() {
    let config = ChartBuilder::new()
        .theme(ChartTheme::dark())
        .title("ETH/USDT Multi-Pane")
        .width(1920.0)
        .height(1080.0)
        .pane(|_| PaneConfig::new("candles"))
        .pane(|_| PaneConfig::new("volume"))
        .pane(|_| PaneConfig::new("RSI(14)"))
        .build();

    assert_eq!(config.panes.len(), 3);
    assert_eq!(config.panes[0].name, "candles");
    assert_eq!(config.panes[1].name, "volume");
    assert_eq!(config.panes[2].name, "RSI(14)");

    let bars = generate_sample_bars(200);
    let mut total_volume: u64 = 0;
    for bar in &bars {
        total_volume += bar.volume;
    }

    println!("Pane count: {}", config.panes.len());
    for pane in &config.panes {
        println!("  - {}", pane.name);
    }
    println!("Total volume across {} bars: {total_volume}", bars.len());
}

fn generate_sample_bars(n: usize) -> Vec<Bar> {
    let mut bars = Vec::with_capacity(n);
    let mut prev = 1800.0;
    for i in 0..n {
        let ts = 1_700_000_000_000 + i as u64 * 60_000;
        let open = prev;
        let close = open + (i as f64 * 0.3 - 30.0);
        let high = open.max(close) + 5.0;
        let low = (open.min(close) - 5.0).max(0.01);
        let bar = Bar::new(ts, open, high, low, close, 8000).unwrap();
        prev = bar.close;
        bars.push(bar);
    }
    bars
}
