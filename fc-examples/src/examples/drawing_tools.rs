//! Drawing tools: horizontal line, vertical line, and Fibonacci retracement.
//!
//! Demonstrates: constructing drawing objects with builder-style chaining,
//! reading level prices from Fibonacci, and the drawing API.

use fc_core::{ChartPoint, FibonacciRetracement, HorizontalLine, LineStyle, VerticalLine};

pub fn run() {
    // Horizontal line at a support level
    let support = HorizontalLine::new("support-49500", 49_500.0)
        .with_color([0.26, 0.65, 0.96, 0.8])
        .with_width(2.0)
        .with_style(LineStyle::Dashed);
    println!("H-line id: {}", support.id.0);
    println!("H-line price: {}", support.price);

    // Vertical line at a specific timestamp
    let session_start = VerticalLine::new("session-open", 1_700_000_000_000)
        .with_color([1.0, 0.84, 0.0, 0.6])
        .with_width(1.5)
        .with_style(LineStyle::Dotted);
    println!("V-line timestamp: {}", session_start.timestamp);

    // Fibonacci retracement between swing low and swing high
    let fib = FibonacciRetracement::new(
        "fib-swing",
        ChartPoint::new(1_700_000_000_000, 48_000.0),
        ChartPoint::new(1_700_006_000_000, 52_000.0),
    )
    .with_color([0.8, 0.4, 1.0, 0.7])
    .with_width(1.0)
    .with_style(LineStyle::Dashed)
    .with_levels(vec![0.0, 0.382, 0.5, 0.618, 1.0]);

    println!("\nFibonacci levels:");
    for (level, price) in fib.level_prices() {
        println!("  {level:.1}%: ${price:.2}", level = level * 100.0);
    }

    println!("\nAll drawings created successfully.");
}
