use super::*;
use crate::price_line::LineStyle;

// ---- ChartPoint ----

#[test]
fn chart_point_new() {
    let p = ChartPoint::new(1000, 50.5);
    assert_eq!(p.timestamp, 1000);
    assert_eq!(p.price, 50.5);
}

// ---- TrendLine ----

#[test]
fn trend_line_new_defaults() {
    let start = ChartPoint::new(100, 10.0);
    let end = ChartPoint::new(200, 20.0);
    let tl = TrendLine::new("tl1", start, end);

    assert_eq!(tl.id, DrawingId("tl1".to_string()));
    assert_eq!(tl.start.timestamp, 100);
    assert_eq!(tl.end.price, 20.0);
    assert_eq!(tl.color, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(tl.width, 1.0);
    assert_eq!(tl.style, LineStyle::Solid);
}

#[test]
fn trend_line_builder() {
    let tl = TrendLine::new(
        "tl2",
        ChartPoint::new(1, 5.0),
        ChartPoint::new(2, 10.0),
    )
    .with_color([1.0, 0.0, 0.0, 1.0])
    .with_width(2.5)
    .with_style(LineStyle::Dashed);

    assert_eq!(tl.color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(tl.width, 2.5);
    assert_eq!(tl.style, LineStyle::Dashed);
}

#[test]
fn trend_line_clone() {
    let tl = TrendLine::new("c", ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0));
    let cloned = tl.clone();
    assert_eq!(cloned.id, tl.id);
}

// ---- HorizontalLine ----

#[test]
fn horizontal_line_new_defaults() {
    let hl = HorizontalLine::new("hl1", 150.0);

    assert_eq!(hl.id, DrawingId("hl1".to_string()));
    assert_eq!(hl.price, 150.0);
    assert_eq!(hl.color, [0.5, 0.5, 0.5, 0.8]);
    assert_eq!(hl.width, 1.0);
    assert_eq!(hl.style, LineStyle::Solid);
    assert!(hl.extend_left);
    assert!(hl.extend_right);
}

#[test]
fn horizontal_line_builder() {
    let hl = HorizontalLine::new("hl2", 200.0)
        .with_color([0.0, 1.0, 0.0, 1.0])
        .with_width(3.0)
        .with_style(LineStyle::Dotted)
        .with_extend_left(false)
        .with_extend_right(false);

    assert_eq!(hl.color, [0.0, 1.0, 0.0, 1.0]);
    assert_eq!(hl.width, 3.0);
    assert_eq!(hl.style, LineStyle::Dotted);
    assert!(!hl.extend_left);
    assert!(!hl.extend_right);
}

// ---- VerticalLine ----

#[test]
fn vertical_line_new_defaults() {
    let vl = VerticalLine::new("vl1", 500);

    assert_eq!(vl.id, DrawingId("vl1".to_string()));
    assert_eq!(vl.timestamp, 500);
    assert_eq!(vl.color, [0.5, 0.5, 0.5, 0.8]);
    assert_eq!(vl.width, 1.0);
    assert_eq!(vl.style, LineStyle::Solid);
}

#[test]
fn vertical_line_builder() {
    let vl = VerticalLine::new("vl2", 600)
        .with_color([1.0, 1.0, 0.0, 1.0])
        .with_width(1.5)
        .with_style(LineStyle::Dashed);

    assert_eq!(vl.color, [1.0, 1.0, 0.0, 1.0]);
    assert_eq!(vl.width, 1.5);
    assert_eq!(vl.style, LineStyle::Dashed);
}

// ---- Rectangle ----

#[test]
fn rectangle_new_defaults() {
    let tl = ChartPoint::new(100, 200.0);
    let br = ChartPoint::new(300, 100.0);
    let rect = Rectangle::new("r1", tl, br);

    assert_eq!(rect.id, DrawingId("r1".to_string()));
    assert_eq!(rect.top_left.timestamp, 100);
    assert_eq!(rect.bottom_right.price, 100.0);
    assert_eq!(rect.color, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(rect.width, 1.0);
    assert_eq!(rect.style, LineStyle::Solid);
    assert!(rect.fill_color.is_none());
}

#[test]
fn rectangle_builder() {
    let rect = Rectangle::new(
        "r2",
        ChartPoint::new(10, 50.0),
        ChartPoint::new(20, 30.0),
    )
    .with_color([1.0, 0.0, 0.0, 0.5])
    .with_width(2.0)
    .with_style(LineStyle::Dotted)
    .with_fill([0.0, 1.0, 0.0, 0.3]);

    assert_eq!(rect.color, [1.0, 0.0, 0.0, 0.5]);
    assert_eq!(rect.width, 2.0);
    assert_eq!(rect.style, LineStyle::Dotted);
    assert_eq!(rect.fill_color, Some([0.0, 1.0, 0.0, 0.3]));
}

#[test]
fn rectangle_width_ts() {
    let rect = Rectangle::new(
        "r3",
        ChartPoint::new(500, 10.0),
        ChartPoint::new(800, 20.0),
    );
    assert_eq!(rect.width_ts(), 300);
}

#[test]
fn rectangle_width_ts_reversed_corners() {
    // top_left has a later timestamp than bottom_right — should still work
    let rect = Rectangle::new(
        "r4",
        ChartPoint::new(800, 20.0),
        ChartPoint::new(500, 10.0),
    );
    assert_eq!(rect.width_ts(), 300);
}

#[test]
fn rectangle_height_price() {
    let rect = Rectangle::new(
        "r5",
        ChartPoint::new(1, 150.0),
        ChartPoint::new(2, 80.0),
    );
    assert!((rect.height_price() - 70.0).abs() < f64::EPSILON);
}

#[test]
fn rectangle_height_price_reversed_corners() {
    let rect = Rectangle::new(
        "r6",
        ChartPoint::new(1, 80.0),
        ChartPoint::new(2, 150.0),
    );
    assert!((rect.height_price() - 70.0).abs() < f64::EPSILON);
}

#[test]
fn rectangle_zero_size() {
    let rect = Rectangle::new("r7", ChartPoint::new(100, 50.0), ChartPoint::new(100, 50.0));
    assert_eq!(rect.width_ts(), 0);
    assert!((rect.height_price()).abs() < f64::EPSILON);
}

#[test]
fn rectangle_clone() {
    let rect = Rectangle::new("rc", ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0));
    let cloned = rect.clone();
    assert_eq!(cloned.id, rect.id);
}

// ---- FibonacciRetracement ----

#[test]
fn fibonacci_new_defaults() {
    let start = ChartPoint::new(100, 100.0);
    let end = ChartPoint::new(200, 200.0);
    let fib = FibonacciRetracement::new("f1", start, end);

    assert_eq!(fib.id, DrawingId("f1".to_string()));
    assert_eq!(fib.start.price, 100.0);
    assert_eq!(fib.end.price, 200.0);
    assert_eq!(fib.levels, vec![0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0]);
    assert_eq!(fib.color, [0.5, 0.5, 0.5, 0.8]);
    assert_eq!(fib.width, 1.0);
    assert_eq!(fib.style, LineStyle::Dashed);
}

#[test]
fn fibonacci_builder() {
    let fib = FibonacciRetracement::new(
        "f2",
        ChartPoint::new(0, 50.0),
        ChartPoint::new(1, 100.0),
    )
    .with_color([0.0, 0.0, 1.0, 1.0])
    .with_width(2.0)
    .with_style(LineStyle::Solid)
    .with_levels(vec![0.0, 0.5, 1.0]);

    assert_eq!(fib.color, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(fib.width, 2.0);
    assert_eq!(fib.style, LineStyle::Solid);
    assert_eq!(fib.levels, vec![0.0, 0.5, 1.0]);
}

#[test]
fn fibonacci_price_at_level() {
    let fib = FibonacciRetracement::new(
        "f3",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 200.0),
    );

    // range = 100.0
    assert!((fib.price_at_level(0.0) - 100.0).abs() < f64::EPSILON);
    assert!((fib.price_at_level(0.5) - 150.0).abs() < f64::EPSILON);
    assert!((fib.price_at_level(1.0) - 200.0).abs() < f64::EPSILON);
    assert!((fib.price_at_level(0.382) - 138.2).abs() < 1e-10);
    assert!((fib.price_at_level(0.618) - 161.8).abs() < 1e-10);
}

#[test]
fn fibonacci_price_at_level_downtrend() {
    // start price > end price (downtrend)
    let fib = FibonacciRetracement::new(
        "f4",
        ChartPoint::new(0, 200.0),
        ChartPoint::new(1, 100.0),
    );

    // range = -100.0
    assert!((fib.price_at_level(0.0) - 200.0).abs() < f64::EPSILON);
    assert!((fib.price_at_level(0.5) - 150.0).abs() < f64::EPSILON);
    assert!((fib.price_at_level(1.0) - 100.0).abs() < f64::EPSILON);
}

#[test]
fn fibonacci_level_prices_count() {
    let fib = FibonacciRetracement::new(
        "f5",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(1, 100.0),
    );
    let prices = fib.level_prices();
    assert_eq!(prices.len(), 7); // default levels count
}

#[test]
fn fibonacci_level_prices_custom() {
    let fib = FibonacciRetracement::new(
        "f6",
        ChartPoint::new(0, 50.0),
        ChartPoint::new(1, 150.0),
    )
    .with_levels(vec![0.0, 0.5, 1.0]);

    let prices = fib.level_prices();
    assert_eq!(prices.len(), 3);
    assert!((prices[0].1 - 50.0).abs() < f64::EPSILON);
    assert!((prices[1].1 - 100.0).abs() < f64::EPSILON);
    assert!((prices[2].1 - 150.0).abs() < f64::EPSILON);
}

#[test]
fn fibonacci_zero_range() {
    let fib = FibonacciRetracement::new(
        "f7",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 100.0),
    );
    // All levels should return the same price
    let prices = fib.level_prices();
    for &(_, price) in &prices {
        assert!((price - 100.0).abs() < f64::EPSILON);
    }
}

#[test]
fn fibonacci_clone() {
    let fib = FibonacciRetracement::new(
        "fc",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(1, 100.0),
    );
    let cloned = fib.clone();
    assert_eq!(cloned.id, fib.id);
    assert_eq!(cloned.levels, fib.levels);
}

// ---- FibonacciExtension ----

#[test]
fn fibonacci_extension_new_defaults() {
    let a = ChartPoint::new(100, 100.0);
    let b = ChartPoint::new(200, 200.0);
    let c = ChartPoint::new(250, 160.0);
    let ext = FibonacciExtension::new("fe1", a, b, c);

    assert_eq!(ext.id, DrawingId("fe1".to_string()));
    assert_eq!(ext.point_a.price, 100.0);
    assert_eq!(ext.point_b.price, 200.0);
    assert_eq!(ext.point_c.price, 160.0);
    assert_eq!(
        ext.levels,
        vec![0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0, 1.272, 1.618]
    );
    assert_eq!(ext.color, [0.5, 0.5, 0.5, 0.8]);
    assert_eq!(ext.width, 1.0);
    assert_eq!(ext.style, LineStyle::Dashed);
}

#[test]
fn fibonacci_extension_builder() {
    let ext = FibonacciExtension::new(
        "fe2",
        ChartPoint::new(0, 50.0),
        ChartPoint::new(1, 100.0),
        ChartPoint::new(2, 80.0),
    )
    .with_color([0.0, 0.0, 1.0, 1.0])
    .with_width(2.0)
    .with_style(LineStyle::Solid)
    .with_levels(vec![0.0, 0.5, 1.0, 1.618]);

    assert_eq!(ext.color, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(ext.width, 2.0);
    assert_eq!(ext.style, LineStyle::Solid);
    assert_eq!(ext.levels, vec![0.0, 0.5, 1.0, 1.618]);
}

#[test]
fn fibonacci_extension_price_at_level() {
    // A=100, B=200, C=160 → ab_range=100
    // level 0.0  → 160 + 100*0.0   = 160.0
    // level 0.5  → 160 + 100*0.5   = 210.0
    // level 1.0  → 160 + 100*1.0   = 260.0
    // level 1.618→ 160 + 100*1.618 = 321.8
    let ext = FibonacciExtension::new(
        "fe3",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 200.0),
        ChartPoint::new(2, 160.0),
    );

    assert!((ext.price_at_level(0.0) - 160.0).abs() < f64::EPSILON);
    assert!((ext.price_at_level(0.5) - 210.0).abs() < f64::EPSILON);
    assert!((ext.price_at_level(1.0) - 260.0).abs() < f64::EPSILON);
    assert!((ext.price_at_level(1.618) - 321.8).abs() < 1e-10);
}

#[test]
fn fibonacci_extension_price_at_level_downtrend() {
    // A=200, B=100, C=140 → ab_range=-100
    // level 0.0  → 140 + (-100)*0.0 = 140.0
    // level 1.0  → 140 + (-100)*1.0 = 40.0
    // level 1.618→ 140 + (-100)*1.618 = -21.8
    let ext = FibonacciExtension::new(
        "fe4",
        ChartPoint::new(0, 200.0),
        ChartPoint::new(1, 100.0),
        ChartPoint::new(2, 140.0),
    );

    assert!((ext.price_at_level(0.0) - 140.0).abs() < f64::EPSILON);
    assert!((ext.price_at_level(1.0) - 40.0).abs() < f64::EPSILON);
    assert!((ext.price_at_level(1.618) - (-21.8)).abs() < 1e-10);
}

#[test]
fn fibonacci_extension_level_prices_count() {
    let ext = FibonacciExtension::new(
        "fe5",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(1, 100.0),
        ChartPoint::new(2, 50.0),
    );
    let prices = ext.level_prices();
    assert_eq!(prices.len(), 9); // default extension levels count
}

#[test]
fn fibonacci_extension_level_prices_custom() {
    let ext = FibonacciExtension::new(
        "fe6",
        ChartPoint::new(0, 50.0),
        ChartPoint::new(1, 150.0),
        ChartPoint::new(2, 100.0),
    )
    .with_levels(vec![0.0, 1.0, 1.618]);

    let prices = ext.level_prices();
    assert_eq!(prices.len(), 3);
    // ab_range = 100, C = 100
    assert!((prices[0].1 - 100.0).abs() < f64::EPSILON); // 100 + 100*0
    assert!((prices[1].1 - 200.0).abs() < f64::EPSILON); // 100 + 100*1
    assert!((prices[2].1 - 261.8).abs() < 1e-10); // 100 + 100*1.618
}

#[test]
fn fibonacci_extension_zero_range() {
    let ext = FibonacciExtension::new(
        "fe7",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 100.0),
        ChartPoint::new(2, 100.0),
    );
    // All levels should return C.price = 100.0
    let prices = ext.level_prices();
    for &(_, price) in &prices {
        assert!((price - 100.0).abs() < f64::EPSILON);
    }
}

#[test]
fn fibonacci_extension_clone() {
    let ext = FibonacciExtension::new(
        "fec",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(1, 100.0),
        ChartPoint::new(2, 50.0),
    );
    let cloned = ext.clone();
    assert_eq!(cloned.id, ext.id);
    assert_eq!(cloned.levels, ext.levels);
    assert_eq!(cloned.point_a, ext.point_a);
}

// ---- Pitchfork ----

#[test]
fn pitchfork_new_defaults() {
    let a = ChartPoint::new(100, 100.0);
    let b = ChartPoint::new(200, 200.0);
    let c = ChartPoint::new(200, 120.0);
    let pf = Pitchfork::new("pf1", a, b, c);

    assert_eq!(pf.id, DrawingId("pf1".to_string()));
    assert_eq!(pf.point_a.price, 100.0);
    assert_eq!(pf.point_b.price, 200.0);
    assert_eq!(pf.point_c.price, 120.0);
    assert_eq!(pf.color, [0.5, 0.5, 0.5, 0.8]);
    assert_eq!(pf.width, 1.0);
    assert_eq!(pf.style, LineStyle::Solid);
}

#[test]
fn pitchfork_builder() {
    let pf = Pitchfork::new(
        "pf2",
        ChartPoint::new(0, 50.0),
        ChartPoint::new(1, 100.0),
        ChartPoint::new(2, 60.0),
    )
    .with_color([1.0, 0.0, 0.0, 1.0])
    .with_width(2.5)
    .with_style(LineStyle::Dashed);

    assert_eq!(pf.color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(pf.width, 2.5);
    assert_eq!(pf.style, LineStyle::Dashed);
}

#[test]
fn pitchfork_median_at_a() {
    // At A's timestamp, median should be A's price
    let a = ChartPoint::new(0, 100.0);
    let b = ChartPoint::new(100, 200.0);
    let c = ChartPoint::new(100, 120.0);
    let pf = Pitchfork::new("pf3", a, b, c);

    let price = pf.median_price_at(0);
    assert!((price - 100.0).abs() < f64::EPSILON);
}

#[test]
fn pitchfork_median_at_midpoint() {
    // B and C both at timestamp 100 → midpoint ts = 100
    // midpoint price = (200 + 120) / 2 = 160
    let a = ChartPoint::new(0, 100.0);
    let b = ChartPoint::new(100, 200.0);
    let c = ChartPoint::new(100, 120.0);
    let pf = Pitchfork::new("pf4", a, b, c);

    let price = pf.median_price_at(100);
    assert!((price - 160.0).abs() < f64::EPSILON);
}

#[test]
fn pitchfork_median_interpolation() {
    // Linear interpolation: at t=50 (halfway between 0 and 100),
    // median = 100 + (160 - 100) * 0.5 = 130
    let a = ChartPoint::new(0, 100.0);
    let b = ChartPoint::new(100, 200.0);
    let c = ChartPoint::new(100, 120.0);
    let pf = Pitchfork::new("pf5", a, b, c);

    let price = pf.median_price_at(50);
    assert!((price - 130.0).abs() < f64::EPSILON);
}

#[test]
fn pitchfork_upper_at_a() {
    // At A's timestamp, upper = A.price + (B.price - midpoint)
    // midpoint = (200+120)/2 = 160, offset = 200-160 = 40
    // at t=0: upper = 100 + 0 + 40 = 140
    let a = ChartPoint::new(0, 100.0);
    let b = ChartPoint::new(100, 200.0);
    let c = ChartPoint::new(100, 120.0);
    let pf = Pitchfork::new("pf6", a, b, c);

    let price = pf.upper_price_at(0);
    assert!((price - 140.0).abs() < f64::EPSILON);
}

#[test]
fn pitchfork_upper_at_midpoint() {
    // At midpoint: median = 160, upper = 160 + 40 = 200
    let a = ChartPoint::new(0, 100.0);
    let b = ChartPoint::new(100, 200.0);
    let c = ChartPoint::new(100, 120.0);
    let pf = Pitchfork::new("pf7", a, b, c);

    let price = pf.upper_price_at(100);
    assert!((price - 200.0).abs() < f64::EPSILON);
}

#[test]
fn pitchfork_lower_at_a() {
    // At A's timestamp: lower = A.price + 0 + (C.price - midpoint)
    // midpoint = 160, offset = 120 - 160 = -40
    // at t=0: lower = 100 + 0 + (-40) = 60
    let a = ChartPoint::new(0, 100.0);
    let b = ChartPoint::new(100, 200.0);
    let c = ChartPoint::new(100, 120.0);
    let pf = Pitchfork::new("pf8", a, b, c);

    let price = pf.lower_price_at(0);
    assert!((price - 60.0).abs() < f64::EPSILON);
}

#[test]
fn pitchfork_lower_at_midpoint() {
    // At midpoint: lower = 160 + (-40) = 120
    let a = ChartPoint::new(0, 100.0);
    let b = ChartPoint::new(100, 200.0);
    let c = ChartPoint::new(100, 120.0);
    let pf = Pitchfork::new("pf9", a, b, c);

    let price = pf.lower_price_at(100);
    assert!((price - 120.0).abs() < f64::EPSILON);
}

#[test]
fn pitchfork_asymmetric_b_c() {
    // B and C at different timestamps
    // B at t=80, C at t=120 → midpoint ts = 100
    // midpoint price = (200 + 80) / 2 = 140
    let a = ChartPoint::new(0, 100.0);
    let b = ChartPoint::new(80, 200.0);
    let c = ChartPoint::new(120, 80.0);
    let pf = Pitchfork::new("pf10", a, b, c);

    // At t=100 (midpoint), median should be midpoint price
    let median = pf.median_price_at(100);
    assert!((median - 140.0).abs() < f64::EPSILON);

    // Upper offset = 200 - 140 = 60
    let upper = pf.upper_price_at(100);
    assert!((upper - 200.0).abs() < f64::EPSILON);

    // Lower offset = 80 - 140 = -60
    let lower = pf.lower_price_at(100);
    assert!((lower - 80.0).abs() < f64::EPSILON);
}

#[test]
fn pitchfork_past_a() {
    // Before A: saturating_sub clamps t to 0, factor = 0
    // median = A.price = 100
    let a = ChartPoint::new(100, 100.0);
    let b = ChartPoint::new(200, 200.0);
    let c = ChartPoint::new(200, 120.0);
    let pf = Pitchfork::new("pf11", a, b, c);

    let price = pf.median_price_at(0);
    assert!((price - 100.0).abs() < f64::EPSILON);
}

#[test]
fn pitchfork_zero_span() {
    // A and midpoint share the same timestamp → denominator = 0
    let a = ChartPoint::new(100, 100.0);
    let b = ChartPoint::new(100, 200.0);
    let c = ChartPoint::new(100, 120.0);
    let pf = Pitchfork::new("pf12", a, b, c);

    // factor = 0, median = A.price = 100
    let price = pf.median_price_at(100);
    assert!((price - 100.0).abs() < f64::EPSILON);
}

#[test]
fn pitchfork_clone() {
    let pf = Pitchfork::new(
        "pfc",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(1, 100.0),
        ChartPoint::new(2, 50.0),
    );
    let cloned = pf.clone();
    assert_eq!(cloned.id, pf.id);
    assert_eq!(cloned.point_a, pf.point_a);
}

// ---- DrawingSet ----

#[test]
fn drawing_set_starts_empty() {
    let set = DrawingSet::new();
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);
}

#[test]
fn drawing_set_add_trend_line() {
    let mut set = DrawingSet::new();
    set.add_trend_line(TrendLine::new(
        "t1",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(1, 1.0),
    ));
    assert_eq!(set.len(), 1);
    assert!(!set.is_empty());
    assert!(set.get_trend_line(&DrawingId("t1".to_string())).is_some());
}

#[test]
fn drawing_set_add_horizontal_line() {
    let mut set = DrawingSet::new();
    set.add_horizontal_line(HorizontalLine::new("h1", 100.0));
    assert_eq!(set.len(), 1);
    assert!(set.get_horizontal_line(&DrawingId("h1".to_string())).is_some());
}

#[test]
fn drawing_set_add_vertical_line() {
    let mut set = DrawingSet::new();
    set.add_vertical_line(VerticalLine::new("v1", 42));
    assert_eq!(set.len(), 1);
    assert!(set.get_vertical_line(&DrawingId("v1".to_string())).is_some());
}

#[test]
fn drawing_set_remove_trend_line() {
    let mut set = DrawingSet::new();
    set.add_trend_line(TrendLine::new(
        "t1",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(1, 1.0),
    ));
    assert!(set.remove(&DrawingId("t1".to_string())));
    assert_eq!(set.len(), 0);
    assert!(set.get_trend_line(&DrawingId("t1".to_string())).is_none());
}

#[test]
fn drawing_set_remove_horizontal_line() {
    let mut set = DrawingSet::new();
    set.add_horizontal_line(HorizontalLine::new("h1", 100.0));
    assert!(set.remove(&DrawingId("h1".to_string())));
    assert_eq!(set.len(), 0);
}

#[test]
fn drawing_set_remove_vertical_line() {
    let mut set = DrawingSet::new();
    set.add_vertical_line(VerticalLine::new("v1", 42));
    assert!(set.remove(&DrawingId("v1".to_string())));
    assert_eq!(set.len(), 0);
}

#[test]
fn drawing_set_remove_nonexistent() {
    let mut set = DrawingSet::new();
    set.add_horizontal_line(HorizontalLine::new("h1", 100.0));
    assert!(!set.remove(&DrawingId("nope".to_string())));
    assert_eq!(set.len(), 1);
}

#[test]
fn drawing_set_remove_from_mixed() {
    let mut set = DrawingSet::new();
    set.add_trend_line(TrendLine::new(
        "t1",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(1, 1.0),
    ));
    set.add_horizontal_line(HorizontalLine::new("h1", 50.0));
    set.add_vertical_line(VerticalLine::new("v1", 10));
    assert_eq!(set.len(), 3);

    assert!(set.remove(&DrawingId("h1".to_string())));
    assert_eq!(set.len(), 2);
    assert!(set.get_horizontal_line(&DrawingId("h1".to_string())).is_none());
    assert!(set.get_trend_line(&DrawingId("t1".to_string())).is_some());
    assert!(set.get_vertical_line(&DrawingId("v1".to_string())).is_some());
}

#[test]
fn drawing_set_len_counts_all_types() {
    let mut set = DrawingSet::new();
    set.add_trend_line(TrendLine::new(
        "t1",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(1, 1.0),
    ));
    set.add_trend_line(TrendLine::new(
        "t2",
        ChartPoint::new(2, 2.0),
        ChartPoint::new(3, 3.0),
    ));
    set.add_horizontal_line(HorizontalLine::new("h1", 100.0));
    set.add_horizontal_line(HorizontalLine::new("h2", 200.0));
    set.add_horizontal_line(HorizontalLine::new("h3", 300.0));
    set.add_vertical_line(VerticalLine::new("v1", 1));

    assert_eq!(set.len(), 6);
    assert_eq!(set.all_trend_lines().len(), 2);
    assert_eq!(set.all_horizontal_lines().len(), 3);
    assert_eq!(set.all_vertical_lines().len(), 1);
}

#[test]
fn drawing_set_is_empty_after_removing_last() {
    let mut set = DrawingSet::new();
    set.add_vertical_line(VerticalLine::new("v1", 1));
    assert!(!set.is_empty());
    set.remove(&DrawingId("v1".to_string()));
    assert!(set.is_empty());
}

// ---- DrawingSet: Rectangle ----

#[test]
fn drawing_set_add_rectangle() {
    let mut set = DrawingSet::new();
    set.add_rectangle(Rectangle::new(
        "r1",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 50.0),
    ));
    assert_eq!(set.len(), 1);
    assert!(set.get_rectangle(&DrawingId("r1".to_string())).is_some());
}

#[test]
fn drawing_set_remove_rectangle() {
    let mut set = DrawingSet::new();
    set.add_rectangle(Rectangle::new(
        "r1",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 50.0),
    ));
    assert!(set.remove(&DrawingId("r1".to_string())));
    assert_eq!(set.len(), 0);
}

#[test]
fn drawing_set_all_rectangles() {
    let mut set = DrawingSet::new();
    set.add_rectangle(Rectangle::new(
        "r1",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 50.0),
    ));
    set.add_rectangle(Rectangle::new(
        "r2",
        ChartPoint::new(2, 200.0),
        ChartPoint::new(3, 150.0),
    ));
    assert_eq!(set.all_rectangles().len(), 2);
}

// ---- DrawingSet: FibonacciRetracement ----

#[test]
fn drawing_set_add_fibonacci() {
    let mut set = DrawingSet::new();
    set.add_fibonacci_retracement(FibonacciRetracement::new(
        "f1",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 200.0),
    ));
    assert_eq!(set.len(), 1);
    assert!(set
        .get_fibonacci_retracement(&DrawingId("f1".to_string()))
        .is_some());
}

#[test]
fn drawing_set_remove_fibonacci() {
    let mut set = DrawingSet::new();
    set.add_fibonacci_retracement(FibonacciRetracement::new(
        "f1",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 200.0),
    ));
    assert!(set.remove(&DrawingId("f1".to_string())));
    assert_eq!(set.len(), 0);
}

#[test]
fn drawing_set_mixed_with_new_types() {
    let mut set = DrawingSet::new();
    set.add_trend_line(TrendLine::new(
        "t1",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(1, 1.0),
    ));
    set.add_rectangle(Rectangle::new(
        "r1",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 50.0),
    ));
    set.add_fibonacci_retracement(FibonacciRetracement::new(
        "f1",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 200.0),
    ));
    assert_eq!(set.len(), 3);

    assert!(set.remove(&DrawingId("r1".to_string())));
    assert_eq!(set.len(), 2);
    assert!(set.get_trend_line(&DrawingId("t1".to_string())).is_some());
    assert!(set
        .get_fibonacci_retracement(&DrawingId("f1".to_string()))
        .is_some());
}

// ---- DrawingSet: FibonacciExtension ----

#[test]
fn drawing_set_add_fibonacci_extension() {
    let mut set = DrawingSet::new();
    set.add_fibonacci_extension(FibonacciExtension::new(
        "fe1",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 200.0),
        ChartPoint::new(2, 150.0),
    ));
    assert_eq!(set.len(), 1);
    assert!(set
        .get_fibonacci_extension(&DrawingId("fe1".to_string()))
        .is_some());
}

#[test]
fn drawing_set_remove_fibonacci_extension() {
    let mut set = DrawingSet::new();
    set.add_fibonacci_extension(FibonacciExtension::new(
        "fe1",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 200.0),
        ChartPoint::new(2, 150.0),
    ));
    assert!(set.remove(&DrawingId("fe1".to_string())));
    assert_eq!(set.len(), 0);
}

#[test]
fn drawing_set_all_fibonacci_extensions() {
    let mut set = DrawingSet::new();
    set.add_fibonacci_extension(FibonacciExtension::new(
        "fe1",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 200.0),
        ChartPoint::new(2, 150.0),
    ));
    set.add_fibonacci_extension(FibonacciExtension::new(
        "fe2",
        ChartPoint::new(3, 50.0),
        ChartPoint::new(4, 100.0),
        ChartPoint::new(5, 80.0),
    ));
    assert_eq!(set.all_fibonacci_extensions().len(), 2);
}

// ---- DrawingSet: Pitchfork ----

#[test]
fn drawing_set_add_pitchfork() {
    let mut set = DrawingSet::new();
    set.add_pitchfork(Pitchfork::new(
        "pf1",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 200.0),
        ChartPoint::new(2, 120.0),
    ));
    assert_eq!(set.len(), 1);
    assert!(set
        .get_pitchfork(&DrawingId("pf1".to_string()))
        .is_some());
}

#[test]
fn drawing_set_remove_pitchfork() {
    let mut set = DrawingSet::new();
    set.add_pitchfork(Pitchfork::new(
        "pf1",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 200.0),
        ChartPoint::new(2, 120.0),
    ));
    assert!(set.remove(&DrawingId("pf1".to_string())));
    assert_eq!(set.len(), 0);
}

#[test]
fn drawing_set_all_pitchforks() {
    let mut set = DrawingSet::new();
    set.add_pitchfork(Pitchfork::new(
        "pf1",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 200.0),
        ChartPoint::new(2, 120.0),
    ));
    set.add_pitchfork(Pitchfork::new(
        "pf2",
        ChartPoint::new(3, 50.0),
        ChartPoint::new(4, 100.0),
        ChartPoint::new(5, 60.0),
    ));
    assert_eq!(set.all_pitchforks().len(), 2);
}

#[test]
fn drawing_set_mixed_all_types() {
    let mut set = DrawingSet::new();
    set.add_trend_line(TrendLine::new(
        "t1",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(1, 1.0),
    ));
    set.add_fibonacci_extension(FibonacciExtension::new(
        "fe1",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 200.0),
        ChartPoint::new(2, 150.0),
    ));
    set.add_pitchfork(Pitchfork::new(
        "pf1",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1, 200.0),
        ChartPoint::new(2, 120.0),
    ));
    assert_eq!(set.len(), 3);

    assert!(set.remove(&DrawingId("fe1".to_string())));
    assert_eq!(set.len(), 2);
    assert!(set.get_trend_line(&DrawingId("t1".to_string())).is_some());
    assert!(set
        .get_pitchfork(&DrawingId("pf1".to_string()))
        .is_some());
}

// ---- Ellipse ----

#[test]
fn ellipse_new_defaults() {
    let center = ChartPoint::new(500, 150.0);
    let e = Ellipse::new("e1", center, 100.0, 50.0);

    assert_eq!(e.id, DrawingId("e1".to_string()));
    assert_eq!(e.center.timestamp, 500);
    assert_eq!(e.center.price, 150.0);
    assert_eq!(e.radius_x, 100.0);
    assert_eq!(e.radius_y, 50.0);
    assert_eq!(e.color, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(e.width, 1.0);
    assert_eq!(e.style, LineStyle::Solid);
    assert!(e.fill_color.is_none());
}

#[test]
fn ellipse_builder() {
    let e = Ellipse::new("e2", ChartPoint::new(100, 200.0), 30.0, 20.0)
        .with_color([1.0, 0.0, 0.0, 0.8])
        .with_width(2.5)
        .with_style(LineStyle::Dashed)
        .with_fill([0.0, 1.0, 0.0, 0.3]);

    assert_eq!(e.color, [1.0, 0.0, 0.0, 0.8]);
    assert_eq!(e.width, 2.5);
    assert_eq!(e.style, LineStyle::Dashed);
    assert_eq!(e.fill_color, Some([0.0, 1.0, 0.0, 0.3]));
}

#[test]
fn ellipse_contains_center() {
    let e = Ellipse::new("e3", ChartPoint::new(100, 100.0), 50.0, 30.0);
    assert!(e.contains(ChartPoint::new(100, 100.0)));
}

#[test]
fn ellipse_contains_inside() {
    let e = Ellipse::new("e4", ChartPoint::new(100, 100.0), 50.0, 30.0);
    // well inside
    assert!(e.contains(ChartPoint::new(110, 105.0)));
}

#[test]
fn ellipse_contains_outside() {
    let e = Ellipse::new("e5", ChartPoint::new(100, 100.0), 50.0, 30.0);
    // far outside
    assert!(!e.contains(ChartPoint::new(200, 200.0)));
}

#[test]
fn ellipse_contains_on_boundary() {
    // point exactly on the boundary: (dx/rx)^2 + (dy/ry)^2 == 1
    let e = Ellipse::new("e6", ChartPoint::new(100, 100.0), 50.0, 30.0);
    // rightmost point: (150, 100.0)
    assert!(e.contains(ChartPoint::new(150, 100.0)));
    // topmost point: (100, 130.0)
    assert!(e.contains(ChartPoint::new(100, 130.0)));
}

#[test]
fn ellipse_contains_beyond_boundary() {
    let e = Ellipse::new("e7", ChartPoint::new(100, 100.0), 50.0, 30.0);
    // just outside rightmost
    assert!(!e.contains(ChartPoint::new(151, 100.0)));
    // just outside topmost
    assert!(!e.contains(ChartPoint::new(100, 131.0)));
}

#[test]
fn ellipse_bounding_box() {
    let e = Ellipse::new("e8", ChartPoint::new(500, 100.0), 200.0, 50.0);
    let (min, max) = e.bounding_box();

    assert_eq!(min.timestamp, 300);
    assert!((min.price - 50.0).abs() < f64::EPSILON);
    assert_eq!(max.timestamp, 700);
    assert!((max.price - 150.0).abs() < f64::EPSILON);
}

#[test]
fn ellipse_bounding_box_center_at_zero() {
    // saturating_sub prevents underflow
    let e = Ellipse::new("e9", ChartPoint::new(10, 50.0), 100.0, 20.0);
    let (min, _max) = e.bounding_box();
    assert_eq!(min.timestamp, 0); // saturating_sub: 10 - 100 -> 0
}

#[test]
fn ellipse_zero_radii() {
    let e = Ellipse::new("e10", ChartPoint::new(100, 100.0), 0.0, 0.0);
    // center on boundary: (0/0)^2 + (0/0)^2 = NaN, which is not <= 1.0
    assert!(!e.contains(ChartPoint::new(100, 100.0)));
    // bounding box collapses to a point
    let (min, max) = e.bounding_box();
    assert_eq!(min, max);
}

#[test]
fn ellipse_clone() {
    let e = Ellipse::new("ec", ChartPoint::new(100, 100.0), 50.0, 30.0);
    let cloned = e.clone();
    assert_eq!(cloned.id, e.id);
    assert_eq!(cloned.center, e.center);
    assert_eq!(cloned.radius_x, e.radius_x);
}

// ---- Path ----

#[test]
fn path_new_defaults() {
    let points = vec![ChartPoint::new(0, 0.0), ChartPoint::new(10, 20.0)];
    let p = Path::new("p1", points);

    assert_eq!(p.id, DrawingId("p1".to_string()));
    assert_eq!(p.points.len(), 2);
    assert_eq!(p.color, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(p.width, 1.0);
    assert_eq!(p.style, LineStyle::Solid);
    assert!(!p.closed);
}

#[test]
fn path_builder() {
    let p = Path::new("p2", vec![ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0)])
        .with_color([0.0, 0.0, 1.0, 1.0])
        .with_width(3.0)
        .with_style(LineStyle::Dotted)
        .with_closed(true);

    assert_eq!(p.color, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(p.width, 3.0);
    assert_eq!(p.style, LineStyle::Dotted);
    assert!(p.closed);
}

#[test]
fn path_push() {
    let mut p = Path::new("p3", vec![ChartPoint::new(0, 0.0)]);
    p.push(ChartPoint::new(10, 20.0));
    p.push(ChartPoint::new(20, 10.0));
    assert_eq!(p.points.len(), 3);
}

#[test]
fn path_segment_count_open() {
    let p = Path::new(
        "p4",
        vec![
            ChartPoint::new(0, 0.0),
            ChartPoint::new(1, 1.0),
            ChartPoint::new(2, 2.0),
        ],
    );
    assert_eq!(p.segment_count(), 2); // n - 1
}

#[test]
fn path_segment_count_closed() {
    let p = Path::new(
        "p5",
        vec![
            ChartPoint::new(0, 0.0),
            ChartPoint::new(1, 1.0),
            ChartPoint::new(2, 2.0),
        ],
    )
    .with_closed(true);
    assert_eq!(p.segment_count(), 3); // n
}

#[test]
fn path_segment_count_empty() {
    let p = Path::new("p6", vec![]);
    assert_eq!(p.segment_count(), 0);
}

#[test]
fn path_segment_count_single() {
    let p = Path::new("p7", vec![ChartPoint::new(0, 0.0)]);
    assert_eq!(p.segment_count(), 0);
}

#[test]
fn path_total_length_open() {
    // horizontal segment: length = 3
    let p = Path::new(
        "p8",
        vec![
            ChartPoint::new(0, 0.0),
            ChartPoint::new(3, 0.0),
            ChartPoint::new(3, 4.0),
        ],
    );
    // segment 0-3: sqrt(9+0)=3, segment 3-4: sqrt(0+16)=4 → total = 7
    assert!((p.total_length() - 7.0).abs() < f64::EPSILON);
}

#[test]
fn path_total_length_closed() {
    // triangle: (0,0) -> (3,0) -> (3,4) -> close to (0,0)
    let p = Path::new(
        "p9",
        vec![
            ChartPoint::new(0, 0.0),
            ChartPoint::new(3, 0.0),
            ChartPoint::new(3, 4.0),
        ],
    )
    .with_closed(true);
    // 3 + 4 + 5 = 12
    assert!((p.total_length() - 12.0).abs() < f64::EPSILON);
}

#[test]
fn path_total_length_empty() {
    let p = Path::new("p10", vec![]);
    assert!((p.total_length()).abs() < f64::EPSILON);
}

#[test]
fn path_total_length_single_point() {
    let p = Path::new("p11", vec![ChartPoint::new(0, 0.0)]);
    assert!((p.total_length()).abs() < f64::EPSILON);
}

#[test]
fn path_point_access() {
    let points = vec![
        ChartPoint::new(10, 50.0),
        ChartPoint::new(20, 60.0),
        ChartPoint::new(30, 70.0),
    ];
    let p = Path::new("p12", points);

    assert_eq!(p.point(0).unwrap().timestamp, 10);
    assert_eq!(p.point(2).unwrap().price, 70.0);
    assert!(p.point(5).is_none());
}

#[test]
fn path_clone() {
    let p = Path::new("pc", vec![ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0)]);
    let cloned = p.clone();
    assert_eq!(cloned.id, p.id);
    assert_eq!(cloned.points.len(), p.points.len());
}

// ---- DrawingSet: Ellipse ----

#[test]
fn drawing_set_add_ellipse() {
    let mut set = DrawingSet::new();
    set.add_ellipse(Ellipse::new(
        "e1",
        ChartPoint::new(100, 100.0),
        50.0,
        30.0,
    ));
    assert_eq!(set.len(), 1);
    assert!(set.get_ellipse(&DrawingId("e1".to_string())).is_some());
}

#[test]
fn drawing_set_remove_ellipse() {
    let mut set = DrawingSet::new();
    set.add_ellipse(Ellipse::new(
        "e1",
        ChartPoint::new(100, 100.0),
        50.0,
        30.0,
    ));
    assert!(set.remove(&DrawingId("e1".to_string())));
    assert_eq!(set.len(), 0);
}

#[test]
fn drawing_set_all_ellipses() {
    let mut set = DrawingSet::new();
    set.add_ellipse(Ellipse::new(
        "e1",
        ChartPoint::new(100, 100.0),
        50.0,
        30.0,
    ));
    set.add_ellipse(Ellipse::new(
        "e2",
        ChartPoint::new(200, 200.0),
        60.0,
        40.0,
    ));
    assert_eq!(set.all_ellipses().len(), 2);
}

// ---- DrawingSet: Path ----

#[test]
fn drawing_set_add_path() {
    let mut set = DrawingSet::new();
    set.add_path(Path::new(
        "p1",
        vec![ChartPoint::new(0, 0.0), ChartPoint::new(10, 20.0)],
    ));
    assert_eq!(set.len(), 1);
    assert!(set.get_path(&DrawingId("p1".to_string())).is_some());
}

#[test]
fn drawing_set_remove_path() {
    let mut set = DrawingSet::new();
    set.add_path(Path::new(
        "p1",
        vec![ChartPoint::new(0, 0.0), ChartPoint::new(10, 20.0)],
    ));
    assert!(set.remove(&DrawingId("p1".to_string())));
    assert_eq!(set.len(), 0);
}

#[test]
fn drawing_set_all_paths() {
    let mut set = DrawingSet::new();
    set.add_path(Path::new(
        "p1",
        vec![ChartPoint::new(0, 0.0), ChartPoint::new(10, 20.0)],
    ));
    set.add_path(Path::new(
        "p2",
        vec![ChartPoint::new(0, 0.0), ChartPoint::new(5, 10.0)],
    ));
    assert_eq!(set.all_paths().len(), 2);
}

#[test]
fn drawing_set_mixed_with_ellipse_and_path() {
    let mut set = DrawingSet::new();
    set.add_trend_line(TrendLine::new(
        "t1",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(1, 1.0),
    ));
    set.add_ellipse(Ellipse::new(
        "e1",
        ChartPoint::new(100, 100.0),
        50.0,
        30.0,
    ));
    set.add_path(Path::new(
        "p1",
        vec![ChartPoint::new(0, 0.0), ChartPoint::new(10, 20.0)],
    ));
    assert_eq!(set.len(), 3);

    assert!(set.remove(&DrawingId("e1".to_string())));
    assert_eq!(set.len(), 2);
    assert!(set.get_trend_line(&DrawingId("t1".to_string())).is_some());
    assert!(set.get_path(&DrawingId("p1".to_string())).is_some());
}

#[test]
fn drawing_set_add_arrow() {
    let mut set = DrawingSet::new();
    set.add_arrow(Arrow::new(
        "a1",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(100, 50.0),
    ));
    assert_eq!(set.len(), 1);
    assert!(set.get_arrow(&DrawingId("a1".to_string())).is_some());
}

#[test]
fn drawing_set_remove_arrow() {
    let mut set = DrawingSet::new();
    set.add_arrow(Arrow::new(
        "a1",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(100, 50.0),
    ));
    assert!(set.remove(&DrawingId("a1".to_string())));
    assert_eq!(set.len(), 0);
}

#[test]
fn drawing_set_all_arrows() {
    let mut set = DrawingSet::new();
    set.add_arrow(Arrow::new(
        "a1",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(100, 50.0),
    ));
    set.add_arrow(Arrow::new(
        "a2",
        ChartPoint::new(200, 60.0),
        ChartPoint::new(300, 80.0),
    ));
    assert_eq!(set.all_arrows().len(), 2);
}

#[test]
fn arrow_builder_methods() {
    let arrow = Arrow::new("a1", ChartPoint::new(0, 0.0), ChartPoint::new(100, 50.0))
        .with_color([1.0, 0.0, 0.0, 1.0])
        .with_width(2.0)
        .with_style(LineStyle::Dashed)
        .with_arrowhead_size(16.0);
    assert_eq!(arrow.color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(arrow.width, 2.0);
    assert_eq!(arrow.style, LineStyle::Dashed);
    assert_eq!(arrow.arrowhead_size, 16.0);
}

#[test]
fn drawing_set_add_ray() {
    let mut set = DrawingSet::new();
    set.add_ray(Ray::new(
        "r1",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(100, 50.0),
    ));
    assert_eq!(set.len(), 1);
    assert!(set.get_ray(&DrawingId("r1".to_string())).is_some());
}

#[test]
fn drawing_set_remove_ray() {
    let mut set = DrawingSet::new();
    set.add_ray(Ray::new(
        "r1",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(100, 50.0),
    ));
    assert!(set.remove(&DrawingId("r1".to_string())));
    assert_eq!(set.len(), 0);
}

#[test]
fn drawing_set_all_rays() {
    let mut set = DrawingSet::new();
    set.add_ray(Ray::new("r1", ChartPoint::new(0, 0.0), ChartPoint::new(100, 50.0)));
    set.add_ray(Ray::new("r2", ChartPoint::new(200, 60.0), ChartPoint::new(300, 80.0)));
    assert_eq!(set.all_rays().len(), 2);
}

#[test]
fn ray_builder_methods() {
    let ray = Ray::new("r1", ChartPoint::new(0, 0.0), ChartPoint::new(100, 50.0))
        .with_color([0.0, 1.0, 0.0, 1.0])
        .with_width(2.0)
        .with_style(LineStyle::Dotted);
    assert_eq!(ray.color, [0.0, 1.0, 0.0, 1.0]);
    assert_eq!(ray.width, 2.0);
    assert_eq!(ray.style, LineStyle::Dotted);
}

#[test]
fn drawing_set_add_segment() {
    let mut set = DrawingSet::new();
    set.add_segment(Segment::new(
        "s1",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(100, 50.0),
    ));
    assert_eq!(set.len(), 1);
    assert!(set.get_segment(&DrawingId("s1".to_string())).is_some());
}

#[test]
fn drawing_set_remove_segment() {
    let mut set = DrawingSet::new();
    set.add_segment(Segment::new(
        "s1",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(100, 50.0),
    ));
    assert!(set.remove(&DrawingId("s1".to_string())));
    assert_eq!(set.len(), 0);
}

#[test]
fn drawing_set_all_segments() {
    let mut set = DrawingSet::new();
    set.add_segment(Segment::new("s1", ChartPoint::new(0, 0.0), ChartPoint::new(100, 50.0)));
    set.add_segment(Segment::new("s2", ChartPoint::new(200, 60.0), ChartPoint::new(300, 80.0)));
    assert_eq!(set.all_segments().len(), 2);
}

#[test]
fn segment_builder_methods() {
    let seg = Segment::new("s1", ChartPoint::new(0, 0.0), ChartPoint::new(100, 50.0))
        .with_color([0.0, 0.0, 1.0, 1.0])
        .with_width(3.0)
        .with_style(LineStyle::Dotted);
    assert_eq!(seg.color, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(seg.width, 3.0);
    assert_eq!(seg.style, LineStyle::Dotted);
}

// ---- move_drawing correctness tests ----

#[test]
fn move_fibonacci_retracement() {
    let mut set = DrawingSet::new();
    let id = DrawingId("fib".into());
    set.add_fibonacci_retracement(
        FibonacciRetracement::new("fib", ChartPoint::new(100, 100.0), ChartPoint::new(200, 200.0)),
    );
    let delta = ChartPoint::new(10, 5.0);
    assert!(set.move_drawing(&id, delta));
    let fib = set.get_fibonacci_retracement(&id).unwrap();
    assert_eq!(fib.start, ChartPoint::new(110, 105.0));
    assert_eq!(fib.end, ChartPoint::new(210, 205.0));
}

#[test]
fn move_fibonacci_extension() {
    let mut set = DrawingSet::new();
    let id = DrawingId("ext".into());
    set.add_fibonacci_extension(
        FibonacciExtension::new("ext", ChartPoint::new(100, 100.0), ChartPoint::new(200, 200.0), ChartPoint::new(150, 150.0)),
    );
    let delta = ChartPoint::new(5, 10.0);
    assert!(set.move_drawing(&id, delta));
    let ext = set.get_fibonacci_extension(&id).unwrap();
    assert_eq!(ext.point_a, ChartPoint::new(105, 110.0));
    assert_eq!(ext.point_b, ChartPoint::new(205, 210.0));
    assert_eq!(ext.point_c, ChartPoint::new(155, 160.0));
}

#[test]
fn move_pitchfork() {
    let mut set = DrawingSet::new();
    let id = DrawingId("pf".into());
    set.add_pitchfork(
        Pitchfork::new("pf", ChartPoint::new(100, 100.0), ChartPoint::new(200, 200.0), ChartPoint::new(200, 50.0)),
    );
    let delta = ChartPoint::new(20, -5.0);
    assert!(set.move_drawing(&id, delta));
    let pf = set.get_pitchfork(&id).unwrap();
    assert_eq!(pf.point_a, ChartPoint::new(120, 95.0));
    assert_eq!(pf.point_b, ChartPoint::new(220, 195.0));
    assert_eq!(pf.point_c, ChartPoint::new(220, 45.0));
}

#[test]
fn move_ellipse() {
    let mut set = DrawingSet::new();
    let id = DrawingId("el".into());
    set.add_ellipse(
        Ellipse::new("el", ChartPoint::new(500, 150.0), 100.0, 50.0),
    );
    let delta = ChartPoint::new(30, 20.0);
    assert!(set.move_drawing(&id, delta));
    let el = set.get_ellipse(&id).unwrap();
    assert_eq!(el.center, ChartPoint::new(530, 170.0));
    assert_eq!(el.radius_x, 100.0);
    assert_eq!(el.radius_y, 50.0);
}

#[test]
fn move_nonexistent_returns_false() {
    let mut set = DrawingSet::new();
    let id = DrawingId("nope".into());
    assert!(!set.move_drawing(&id, ChartPoint::new(1, 1.0)));
}

// ---- Serde roundtrip tests ----

#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;
    use serde_json;

    #[test]
    fn roundtrip_chart_point() {
        let p = ChartPoint::new(1000, 50.5);
        let json = serde_json::to_string(&p).unwrap();
        let back: ChartPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn roundtrip_drawing_id() {
        let id = DrawingId("test-id".to_string());
        let json = serde_json::to_string(&id).unwrap();
        let back: DrawingId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn roundtrip_line_style() {
        for style in [LineStyle::Solid, LineStyle::Dashed, LineStyle::Dotted] {
            let json = serde_json::to_string(&style).unwrap();
            let back: LineStyle = serde_json::from_str(&json).unwrap();
            assert_eq!(style, back);
        }
    }

    #[test]
    fn roundtrip_trend_line() {
        let tl = TrendLine::new("tl", ChartPoint::new(100, 10.0), ChartPoint::new(200, 20.0))
            .with_color([1.0, 0.0, 0.0, 1.0])
            .with_width(2.0);
        let json = serde_json::to_string(&tl).unwrap();
        let back: TrendLine = serde_json::from_str(&json).unwrap();
        assert_eq!(tl.id, back.id);
        assert_eq!(tl.start, back.start);
        assert_eq!(tl.end, back.end);
        assert_eq!(tl.color, back.color);
        assert_eq!(tl.width, back.width);
    }

    #[test]
    fn roundtrip_horizontal_line() {
        let hl = HorizontalLine::new("hl", 150.0).with_extend_left(false);
        let json = serde_json::to_string(&hl).unwrap();
        let back: HorizontalLine = serde_json::from_str(&json).unwrap();
        assert_eq!(hl.id, back.id);
        assert_eq!(hl.price, back.price);
        assert!(!back.extend_left);
    }

    #[test]
    fn roundtrip_vertical_line() {
        let vl = VerticalLine::new("vl", 500).with_color([0.0, 1.0, 0.0, 1.0]);
        let json = serde_json::to_string(&vl).unwrap();
        let back: VerticalLine = serde_json::from_str(&json).unwrap();
        assert_eq!(vl.timestamp, back.timestamp);
        assert_eq!(vl.color, back.color);
    }

    #[test]
    fn roundtrip_rectangle() {
        let rect = Rectangle::new("r", ChartPoint::new(100, 200.0), ChartPoint::new(300, 100.0))
            .with_fill([1.0, 0.0, 0.0, 0.5]);
        let json = serde_json::to_string(&rect).unwrap();
        let back: Rectangle = serde_json::from_str(&json).unwrap();
        assert_eq!(rect.top_left, back.top_left);
        assert_eq!(rect.fill_color, back.fill_color);
    }

    #[test]
    fn roundtrip_arrow() {
        let a = Arrow::new("a", ChartPoint::new(0, 0.0), ChartPoint::new(100, 50.0))
            .with_arrowhead_size(15.0);
        let json = serde_json::to_string(&a).unwrap();
        let back: Arrow = serde_json::from_str(&json).unwrap();
        assert_eq!(a.arrowhead_size, back.arrowhead_size);
    }

    #[test]
    fn roundtrip_fibonacci_retracement() {
        let fib = FibonacciRetracement::new(
            "fib",
            ChartPoint::new(100, 100.0),
            ChartPoint::new(200, 200.0),
        );
        let json = serde_json::to_string(&fib).unwrap();
        let back: FibonacciRetracement = serde_json::from_str(&json).unwrap();
        assert_eq!(fib.start, back.start);
        assert_eq!(fib.end, back.end);
        assert_eq!(fib.levels, back.levels);
    }

    #[test]
    fn roundtrip_fibonacci_extension() {
        let ext = FibonacciExtension::new(
            "ext",
            ChartPoint::new(100, 100.0),
            ChartPoint::new(200, 200.0),
            ChartPoint::new(150, 150.0),
        );
        let json = serde_json::to_string(&ext).unwrap();
        let back: FibonacciExtension = serde_json::from_str(&json).unwrap();
        assert_eq!(ext.point_a, back.point_a);
        assert_eq!(ext.point_b, back.point_b);
        assert_eq!(ext.point_c, back.point_c);
        assert_eq!(ext.levels, back.levels);
    }

    #[test]
    fn roundtrip_pitchfork() {
        let pf = Pitchfork::new(
            "pf",
            ChartPoint::new(100, 100.0),
            ChartPoint::new(200, 200.0),
            ChartPoint::new(200, 50.0),
        );
        let json = serde_json::to_string(&pf).unwrap();
        let back: Pitchfork = serde_json::from_str(&json).unwrap();
        assert_eq!(pf.point_a, back.point_a);
    }

    #[test]
    fn roundtrip_ellipse() {
        let e = Ellipse::new("e", ChartPoint::new(500, 150.0), 100.0, 50.0)
            .with_fill([0.0, 1.0, 0.0, 0.3]);
        let json = serde_json::to_string(&e).unwrap();
        let back: Ellipse = serde_json::from_str(&json).unwrap();
        assert_eq!(e.center, back.center);
        assert_eq!(e.radius_x, back.radius_x);
        assert_eq!(e.fill_color, back.fill_color);
    }

    #[test]
    fn roundtrip_path() {
        let p = Path::new(
            "path",
            vec![ChartPoint::new(0, 0.0), ChartPoint::new(100, 100.0), ChartPoint::new(200, 50.0)],
        )
        .with_closed(true)
        .with_fill([1.0, 0.0, 0.0, 0.5]);
        let json = serde_json::to_string(&p).unwrap();
        let back: Path = serde_json::from_str(&json).unwrap();
        assert_eq!(p.points, back.points);
        assert!(back.closed);
        assert!(back.fill_color.is_some());
    }

    #[test]
    fn roundtrip_segment() {
        let seg = Segment::new("seg", ChartPoint::new(0, 0.0), ChartPoint::new(100, 100.0));
        let json = serde_json::to_string(&seg).unwrap();
        let back: Segment = serde_json::from_str(&json).unwrap();
        assert_eq!(seg.start, back.start);
        assert_eq!(seg.end, back.end);
    }

    #[test]
    fn roundtrip_ray() {
        let ray = Ray::new("ray", ChartPoint::new(0, 0.0), ChartPoint::new(100, 100.0));
        let json = serde_json::to_string(&ray).unwrap();
        let back: Ray = serde_json::from_str(&json).unwrap();
        assert_eq!(ray.start, back.start);
        assert_eq!(ray.direction, back.direction);
    }

    #[test]
    fn roundtrip_text_drawing() {
        let td = TextDrawing::new("td", ChartPoint::new(100, 50.0), "Hello")
            .with_font_size(18.0)
            .with_align_x(0.5);
        let json = serde_json::to_string(&td).unwrap();
        let back: TextDrawing = serde_json::from_str(&json).unwrap();
        assert_eq!(td.text, back.text);
        assert_eq!(td.font_size, back.font_size);
    }

    #[test]
    fn roundtrip_image_drawing() {
        let img = ImageDrawing::new("img", ChartPoint::new(0, 0.0), "logo.png")
            .with_width(200.0)
            .with_opacity(0.8);
        let json = serde_json::to_string(&img).unwrap();
        let back: ImageDrawing = serde_json::from_str(&json).unwrap();
        assert_eq!(img.src, back.src);
        assert_eq!(img.opacity, back.opacity);
    }

    #[test]
    fn roundtrip_label_drawing() {
        let label = LabelDrawing::new("lbl", ChartPoint::new(100, 100.0), "Buy")
            .with_bg_color([0.0, 0.5, 0.0, 0.9]);
        let json = serde_json::to_string(&label).unwrap();
        let back: LabelDrawing = serde_json::from_str(&json).unwrap();
        assert_eq!(label.text, back.text);
        assert_eq!(label.bg_color, back.bg_color);
    }
}
