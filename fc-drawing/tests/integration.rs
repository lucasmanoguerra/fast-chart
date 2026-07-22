// ---------------------------------------------------------------------------
// fc-drawing integration tests
//
// Clasificación: todos los tests son determinística — operan sobre datos
// estáticos y no dependen de I/O, red ni estado externo.
// ---------------------------------------------------------------------------

use fc_drawing::{DrawingBounds, Drawing, HitResult, default_aabb_hit_test};
use fc_domain::drawing::{
    ChartPoint,
    TrendLine, Arrow, Ray, Segment, TextDrawing, ImageDrawing, LabelDrawing,
    HorizontalLine, VerticalLine, Rectangle, FibonacciRetracement, FibonacciExtension,
    Pitchfork, Ellipse, Path,
};
use fc_domain::LineStyle;

// ===========================================================================
// DrawingBounds
// ===========================================================================

// Clasificación: determinística — construction and field access
#[test]
fn drawing_bounds_new() {
    let b = DrawingBounds::new(100, 200, 50.0, 100.0);
    assert_eq!(b.time_start, 100);
    assert_eq!(b.time_end, 200);
    assert_eq!(b.price_min, 50.0);
    assert_eq!(b.price_max, 100.0);
}

// Clasificación: determinística — from_point creates zero-size bounds
#[test]
fn drawing_bounds_from_point() {
    let p = ChartPoint::new(500, 75.5);
    let b = DrawingBounds::from_point(p);
    assert_eq!(b.time_start, 500);
    assert_eq!(b.time_end, 500);
    assert_eq!(b.price_min, 75.5);
    assert_eq!(b.price_max, 75.5);
}

// Clasificación: determinística — from_points normalizes min/max
#[test]
fn drawing_bounds_from_points() {
    let a = ChartPoint::new(200, 80.0);
    let b = ChartPoint::new(100, 60.0);
    let bounds = DrawingBounds::from_points(a, b);
    assert_eq!(bounds.time_start, 100);
    assert_eq!(bounds.time_end, 200);
    assert_eq!(bounds.price_min, 60.0);
    assert_eq!(bounds.price_max, 80.0);
}

// Clasificación: determinística — time_width calcula diferencia saturada
#[test]
fn drawing_bounds_time_width() {
    let b = DrawingBounds::new(100, 350, 0.0, 0.0);
    assert_eq!(b.time_width(), 250);
}

// Clasificación: determinística — time_width en caso saturado (end < start)
#[test]
fn drawing_bounds_time_width_saturating() {
    let b = DrawingBounds::new(500, 100, 0.0, 0.0);
    assert_eq!(b.time_width(), 0);
}

// Clasificación: determinística — price_height calcula diferencia
#[test]
fn drawing_bounds_price_height() {
    let b = DrawingBounds::new(0, 0, 30.0, 90.0);
    assert!((b.price_height() - 60.0).abs() < f64::EPSILON);
}

// Clasificación: determinística — contains verifica pertenencia al bounding box
#[test]
fn drawing_bounds_contains() {
    let b = DrawingBounds::new(10, 50, 20.0, 80.0);
    let inside = ChartPoint::new(30, 50.0);
    let outside_time = ChartPoint::new(60, 50.0);
    let outside_price = ChartPoint::new(30, 90.0);
    assert!(b.contains(inside));
    assert!(!b.contains(outside_time));
    assert!(!b.contains(outside_price));
}

// Clasificación: determinística — contains en bordes inclusivos
#[test]
fn drawing_bounds_contains_on_boundary() {
    let b = DrawingBounds::new(10, 50, 20.0, 80.0);
    assert!(b.contains(ChartPoint::new(10, 20.0)));
    assert!(b.contains(ChartPoint::new(50, 80.0)));
}

// Clasificación: determinística — combine unifica dos bounding boxes
#[test]
fn drawing_bounds_combine() {
    let a = DrawingBounds::new(10, 30, 10.0, 40.0);
    let c = DrawingBounds::new(20, 60, 5.0, 80.0);
    let combined = a.combine(&c);
    assert_eq!(combined.time_start, 10);
    assert_eq!(combined.time_end, 60);
    assert_eq!(combined.price_min, 5.0);
    assert_eq!(combined.price_max, 80.0);
}

// Clasificación: determinística — combine con un box contenida en el otro
#[test]
fn drawing_bounds_combine_absorbs() {
    let outer = DrawingBounds::new(0, 100, 0.0, 100.0);
    let inner = DrawingBounds::new(20, 80, 20.0, 80.0);
    let combined = outer.combine(&inner);
    assert_eq!(combined, outer);
}

// ===========================================================================
// HitResult + default_aabb_hit_test
// ===========================================================================

// Clasificación: determinística — AABB hit test con punto dentro
#[test]
fn aabb_hit_inside() {
    let bounds = DrawingBounds::new(100, 200, 50.0, 100.0);
    let p = ChartPoint::new(150, 75.0);
    assert_eq!(default_aabb_hit_test(&bounds, p, 5.0), HitResult::Body);
}

// Clasificación: determinística — AABB hit test con punto fuera
#[test]
fn aabb_hit_outside() {
    let bounds = DrawingBounds::new(100, 200, 50.0, 100.0);
    let p = ChartPoint::new(500, 200.0);
    assert_eq!(default_aabb_hit_test(&bounds, p, 5.0), HitResult::Miss);
}

// Clasificación: determinística — AABB hit test con tolerancia
#[test]
fn aabb_hit_within_tolerance() {
    let bounds = DrawingBounds::new(100, 200, 50.0, 100.0);
    // Just outside but within tolerance
    let p = ChartPoint::new(96, 46.0);
    assert_eq!(default_aabb_hit_test(&bounds, p, 5.0), HitResult::Body);
}

// Clasificación: determinística — AABB hit test fuera de tolerancia
#[test]
fn aabb_hit_beyond_tolerance() {
    let bounds = DrawingBounds::new(100, 200, 50.0, 100.0);
    let p = ChartPoint::new(80, 30.0);
    assert_eq!(default_aabb_hit_test(&bounds, p, 5.0), HitResult::Miss);
}

// ===========================================================================
// TrendLine
// ===========================================================================

// Clasificación: determinística — construcción con valores por defecto
#[test]
fn trendline_construction() {
    let start = ChartPoint::new(1000, 50.0);
    let end = ChartPoint::new(2000, 80.0);
    let tl = TrendLine::new("tl-1", start, end);
    assert_eq!(tl.id().0, "tl-1");
    assert_eq!(tl.start, start);
    assert_eq!(tl.end, end);
    assert_eq!(tl.color, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(tl.width, 1.0);
    assert_eq!(tl.style, LineStyle::Solid);
    assert!(!tl.selected);
}

// Clasificación: determinística — builder methods aplican correctamente
#[test]
fn trendline_builder() {
    let tl = TrendLine::new(
        "tl-2",
        ChartPoint::new(100, 10.0),
        ChartPoint::new(200, 20.0),
    )
    .with_color([1.0, 0.0, 0.0, 1.0])
    .with_width(3.0)
    .with_style(LineStyle::Dashed);
    assert_eq!(tl.color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(tl.width, 3.0);
    assert_eq!(tl.style, LineStyle::Dashed);
}

// Clasificación: determinística — move_by desplaza ambos extremos
#[test]
fn trendline_move_by() {
    let mut tl = TrendLine::new(
        "tl-3",
        ChartPoint::new(1000, 50.0),
        ChartPoint::new(2000, 80.0),
    );
    let delta = ChartPoint::new(500, 10.0);
    tl.move_by(delta);
    assert_eq!(tl.start, ChartPoint::new(1500, 60.0));
    assert_eq!(tl.end, ChartPoint::new(2500, 90.0));
}

// Clasificación: determinística — bounds calcula bounding box desde ambos puntos
#[test]
fn trendline_bounds() {
    let tl = TrendLine::new(
        "tl-4",
        ChartPoint::new(2000, 80.0),
        ChartPoint::new(1000, 30.0),
    );
    let b = tl.bounds();
    assert_eq!(b.time_start, 1000);
    assert_eq!(b.time_end, 2000);
    assert_eq!(b.price_min, 30.0);
    assert_eq!(b.price_max, 80.0);
}

// Clasificación: determinística — hit_test con punto sobre la línea
#[test]
fn trendline_hit_test_on_line() {
    let tl = TrendLine::new(
        "tl-5",
        ChartPoint::new(1000, 50.0),
        ChartPoint::new(2000, 100.0),
    );
    // Punto dentro del AABB
    let p = ChartPoint::new(1500, 75.0);
    assert_eq!(tl.hit_test(p, 10.0), HitResult::Body);
}

// Clasificación: determinística — hit_test con punto lejano
#[test]
fn trendline_hit_test_miss() {
    let tl = TrendLine::new(
        "tl-6",
        ChartPoint::new(1000, 50.0),
        ChartPoint::new(1050, 55.0),
    );
    let p = ChartPoint::new(5000, 200.0);
    assert_eq!(tl.hit_test(p, 1.0), HitResult::Miss);
}

// Clasificación: determinística — selección se puede alternar
#[test]
fn trendline_selection() {
    let mut tl = TrendLine::new(
        "tl-7",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(100, 100.0),
    );
    assert!(!tl.is_selected());
    tl.set_selected(true);
    assert!(tl.is_selected());
    tl.set_selected(false);
    assert!(!tl.is_selected());
}

// Clasificación: determinística — as_any permite downcast
#[test]
fn trendline_as_any_downcast() {
    let tl = TrendLine::new(
        "tl-8",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(10, 10.0),
    );
    let drawing: &dyn Drawing = &tl;
    assert!(drawing.as_any().downcast_ref::<TrendLine>().is_some());
    assert!(drawing.as_any().downcast_ref::<Arrow>().is_none());
}

// ===========================================================================
// Arrow
// ===========================================================================

// Clasificación: determinística — construcción con arrowhead_size por defecto
#[test]
fn arrow_construction() {
    let a = Arrow::new("arr-1", ChartPoint::new(100, 10.0), ChartPoint::new(200, 20.0));
    assert_eq!(a.id().0, "arr-1");
    assert_eq!(a.arrowhead_size, 12.0);
    assert!(!a.selected);
}

// Clasificación: determinística — builder con arrowhead_size custom
#[test]
fn arrow_builder() {
    let a = Arrow::new("arr-2", ChartPoint::new(0, 0.0), ChartPoint::new(100, 100.0))
        .with_arrowhead_size(20.0)
        .with_width(2.5)
        .with_color([0.0, 1.0, 0.0, 1.0]);
    assert_eq!(a.arrowhead_size, 20.0);
    assert_eq!(a.width, 2.5);
    assert_eq!(a.color, [0.0, 1.0, 0.0, 1.0]);
}

// Clasificación: determinística — move_by desplaza ambos extremos
#[test]
fn arrow_move_by() {
    let mut a = Arrow::new("arr-3", ChartPoint::new(100, 10.0), ChartPoint::new(200, 30.0));
    a.move_by(ChartPoint::new(50, 5.0));
    assert_eq!(a.start, ChartPoint::new(150, 15.0));
    assert_eq!(a.end, ChartPoint::new(250, 35.0));
}

// Clasificación: determinística — bounds calcula AABB de ambos extremos
#[test]
fn arrow_bounds() {
    let a = Arrow::new("arr-4", ChartPoint::new(300, 50.0), ChartPoint::new(100, 10.0));
    let b = a.bounds();
    assert_eq!(b.time_start, 100);
    assert_eq!(b.time_end, 300);
    assert_eq!(b.price_min, 10.0);
    assert_eq!(b.price_max, 50.0);
}

// ===========================================================================
// Ray
// ===========================================================================

// Clasificación: determinística — construcción de ray
#[test]
fn ray_construction() {
    let r = Ray::new("ray-1", ChartPoint::new(100, 50.0), ChartPoint::new(200, 70.0));
    assert_eq!(r.id().0, "ray-1");
    assert_eq!(r.start, ChartPoint::new(100, 50.0));
    assert_eq!(r.direction, ChartPoint::new(200, 70.0));
}

// Clasificación: determinística — move_by solo desplaza el origen
#[test]
fn ray_move_by() {
    let mut r = Ray::new("ray-2", ChartPoint::new(100, 50.0), ChartPoint::new(200, 70.0));
    r.move_by(ChartPoint::new(10, 5.0));
    assert_eq!(r.start, ChartPoint::new(110, 55.0));
    // direction no se mueve (solo start se desplaza en Ray)
    assert_eq!(r.direction, ChartPoint::new(200, 70.0));
}

// Clasificación: determinística — bounds usa start y direction
#[test]
fn ray_bounds() {
    let r = Ray::new("ray-3", ChartPoint::new(200, 80.0), ChartPoint::new(100, 30.0));
    let b = r.bounds();
    assert_eq!(b.time_start, 100);
    assert_eq!(b.time_end, 200);
    assert_eq!(b.price_min, 30.0);
    assert_eq!(b.price_max, 80.0);
}

// Clasificación: determinística — hit_test proyecta punto sobre el rayo
#[test]
fn ray_hit_test_on_ray() {
    let r = Ray::new("ray-4", ChartPoint::new(100, 50.0), ChartPoint::new(200, 70.0));
    // Punto sobre la dirección del rayo, más allá de direction
    let p = ChartPoint::new(300, 90.0);
    assert_eq!(r.hit_test(p, 10.0), HitResult::Body);
}

// Clasificación: determinística — hit_test miss fuera del rayo
#[test]
fn ray_hit_test_miss() {
    let r = Ray::new("ray-5", ChartPoint::new(100, 50.0), ChartPoint::new(200, 50.0));
    let p = ChartPoint::new(150, 200.0);
    assert_eq!(r.hit_test(p, 5.0), HitResult::Miss);
}

// Clasificación: determinística — ray degenerado (start == direction)
#[test]
fn ray_hit_test_degenerate() {
    let r = Ray::new("ray-6", ChartPoint::new(100, 50.0), ChartPoint::new(100, 50.0));
    let p = ChartPoint::new(100, 50.0);
    assert_eq!(r.hit_test(p, 5.0), HitResult::Body);
    let far = ChartPoint::new(999, 999.0);
    assert_eq!(r.hit_test(far, 1.0), HitResult::Miss);
}

// ===========================================================================
// Segment
// ===========================================================================

// Clasificación: determinística — construcción con valores por defecto
#[test]
fn segment_construction() {
    let s = Segment::new("seg-1", ChartPoint::new(100, 20.0), ChartPoint::new(300, 80.0));
    assert_eq!(s.id().0, "seg-1");
    assert_eq!(s.color, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(s.width, 1.0);
    assert_eq!(s.style, LineStyle::Solid);
}

// Clasificación: determinística — move_by desplaza ambos extremos
#[test]
fn segment_move_by() {
    let mut s = Segment::new("seg-2", ChartPoint::new(100, 20.0), ChartPoint::new(300, 80.0));
    s.move_by(ChartPoint::new(50, 10.0));
    assert_eq!(s.start, ChartPoint::new(150, 30.0));
    assert_eq!(s.end, ChartPoint::new(350, 90.0));
}

// Clasificación: determinística — bounds normaliza puntos invertidos
#[test]
fn segment_bounds() {
    let s = Segment::new("seg-3", ChartPoint::new(500, 100.0), ChartPoint::new(100, 20.0));
    let b = s.bounds();
    assert_eq!(b.time_start, 100);
    assert_eq!(b.time_end, 500);
    assert_eq!(b.price_min, 20.0);
    assert_eq!(b.price_max, 100.0);
}

// ===========================================================================
// TextDrawing
// ===========================================================================

// Clasificación: determinística — construcción con valores por defecto
#[test]
fn text_drawing_construction() {
    let t = TextDrawing::new("txt-1", ChartPoint::new(100, 50.0), "Hello");
    assert_eq!(t.id().0, "txt-1");
    assert_eq!(t.text, "Hello");
    assert_eq!(t.font_size, 14.0);
    assert_eq!(t.align_x, 0.0);
    assert_eq!(t.align_y, 0.5);
    assert!(!t.selected);
}

// Clasificación: determinística — builder methods aplican correctamente
#[test]
fn text_drawing_builder() {
    let t = TextDrawing::new("txt-2", ChartPoint::new(0, 0.0), "Test")
        .with_color([1.0, 0.0, 0.0, 1.0])
        .with_font_size(20.0)
        .with_align_x(0.5)
        .with_align_y(1.0);
    assert_eq!(t.color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(t.font_size, 20.0);
    assert_eq!(t.align_x, 0.5);
    assert_eq!(t.align_y, 1.0);
}

// Clasificación: determinística — move_by desplaza posición
#[test]
fn text_drawing_move_by() {
    let mut t = TextDrawing::new("txt-3", ChartPoint::new(100, 50.0), "Move");
    t.move_by(ChartPoint::new(200, 30.0));
    assert_eq!(t.position, ChartPoint::new(300, 80.0));
}

// Clasificación: determinística — bounds es punto-size (zero-size)
#[test]
fn text_drawing_bounds_is_point() {
    let t = TextDrawing::new("txt-4", ChartPoint::new(500, 100.0), "Point");
    let b = t.bounds();
    assert_eq!(b.time_width(), 0);
    assert!((b.price_height()).abs() < f64::EPSILON);
    assert!(b.contains(ChartPoint::new(500, 100.0)));
}

// ===========================================================================
// ImageDrawing
// ===========================================================================

// Clasificación: determinística — construcción con dimensiones por defecto
#[test]
fn image_drawing_construction() {
    let img = ImageDrawing::new("img-1", ChartPoint::new(100, 50.0), "path/to/img.png");
    assert_eq!(img.id().0, "img-1");
    assert_eq!(img.src, "path/to/img.png");
    assert_eq!(img.width, 100.0);
    assert_eq!(img.height, 100.0);
    assert_eq!(img.opacity, 1.0);
}

// Clasificación: determinística — builder methods aplican correctamente
#[test]
fn image_drawing_builder() {
    let img = ImageDrawing::new("img-2", ChartPoint::new(0, 0.0), "a.png")
        .with_width(200.0)
        .with_height(150.0)
        .with_opacity(0.5);
    assert_eq!(img.width, 200.0);
    assert_eq!(img.height, 150.0);
    assert!((img.opacity - 0.5).abs() < f32::EPSILON);
}

// Clasificación: determinística — move_by desplaza posición
#[test]
fn image_drawing_move_by() {
    let mut img = ImageDrawing::new("img-3", ChartPoint::new(100, 50.0), "b.png");
    img.move_by(ChartPoint::new(25, 10.0));
    assert_eq!(img.position, ChartPoint::new(125, 60.0));
}

// ===========================================================================
// LabelDrawing
// ===========================================================================

// Clasificación: determinística — construcción con colores por defecto
#[test]
fn label_drawing_construction() {
    let l = LabelDrawing::new("lbl-1", ChartPoint::new(100, 50.0), "Price");
    assert_eq!(l.id().0, "lbl-1");
    assert_eq!(l.text, "Price");
    assert_eq!(l.font_size, 12.0);
    assert_eq!(l.padding, 4.0);
    assert!(!l.selected);
}

// Clasificación: determinística — builder methods aplican correctamente
#[test]
fn label_drawing_builder() {
    let l = LabelDrawing::new("lbl-2", ChartPoint::new(0, 0.0), "X")
        .with_text_color([0.0, 1.0, 0.0, 1.0])
        .with_bg_color([0.0, 0.0, 0.0, 0.8])
        .with_border_color([1.0, 1.0, 1.0, 1.0])
        .with_font_size(16.0);
    assert_eq!(l.text_color, [0.0, 1.0, 0.0, 1.0]);
    assert_eq!(l.bg_color, [0.0, 0.0, 0.0, 0.8]);
    assert_eq!(l.border_color, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(l.font_size, 16.0);
}

// Clasificación: determinística — move_by desplaza posición
#[test]
fn label_drawing_move_by() {
    let mut l = LabelDrawing::new("lbl-3", ChartPoint::new(100, 50.0), "Move");
    l.move_by(ChartPoint::new(50, 25.0));
    assert_eq!(l.position, ChartPoint::new(150, 75.0));
}

// ===========================================================================
// HorizontalLine
// ===========================================================================

// Clasificación: determinística — construcción con valores por defecto
#[test]
fn horizontal_line_construction() {
    let hl = HorizontalLine::new("hl-1", 75.5);
    assert_eq!(hl.id().0, "hl-1");
    assert_eq!(hl.price, 75.5);
    assert!(hl.extend_left);
    assert!(hl.extend_right);
    assert_eq!(hl.style, LineStyle::Solid);
}

// Clasificación: determinística — move_by solo modifica price
#[test]
fn horizontal_line_move_by() {
    let mut hl = HorizontalLine::new("hl-2", 100.0);
    hl.move_by(ChartPoint::new(999, 25.0));
    assert_eq!(hl.price, 125.0);
    // Timestamp no aplica para horizontal
}

// Clasificación: determinística — bounds cubre todo el rango temporal
#[test]
fn horizontal_line_bounds() {
    let hl = HorizontalLine::new("hl-3", 50.0);
    let b = hl.bounds();
    assert_eq!(b.time_start, 0);
    assert_eq!(b.time_end, u64::MAX);
    assert_eq!(b.price_min, 50.0);
    assert_eq!(b.price_max, 50.0);
}

// Clasificación: determinística — builder con extensiones personalizadas
#[test]
fn horizontal_line_builder() {
    let hl = HorizontalLine::new("hl-4", 100.0)
        .with_color([1.0, 0.0, 0.0, 1.0])
        .with_width(2.0)
        .with_style(LineStyle::Dotted)
        .with_extend_left(false)
        .with_extend_right(false);
    assert_eq!(hl.color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(hl.width, 2.0);
    assert_eq!(hl.style, LineStyle::Dotted);
    assert!(!hl.extend_left);
    assert!(!hl.extend_right);
}

// ===========================================================================
// VerticalLine
// ===========================================================================

// Clasificación: determinística — construcción con valores por defecto
#[test]
fn vertical_line_construction() {
    let vl = VerticalLine::new("vl-1", 5000);
    assert_eq!(vl.id().0, "vl-1");
    assert_eq!(vl.timestamp, 5000);
    assert_eq!(vl.style, LineStyle::Solid);
    assert!(!vl.selected);
}

// Clasificación: determinística — move_by solo modifica timestamp
#[test]
fn vertical_line_move_by() {
    let mut vl = VerticalLine::new("vl-2", 1000);
    vl.move_by(ChartPoint::new(500, 999.0));
    assert_eq!(vl.timestamp, 1500);
}

// Clasificación: determinística — bounds cubre todo el rango de precios
#[test]
fn vertical_line_bounds() {
    let vl = VerticalLine::new("vl-3", 3000);
    let b = vl.bounds();
    assert_eq!(b.time_start, 3000);
    assert_eq!(b.time_end, 3000);
    assert_eq!(b.price_min, f64::MIN);
    assert_eq!(b.price_max, f64::MAX);
}

// ===========================================================================
// Rectangle
// ===========================================================================

// Clasificación: determinística — construcción con valores por defecto
#[test]
fn rectangle_construction() {
    let r = Rectangle::new("rect-1", ChartPoint::new(100, 80.0), ChartPoint::new(300, 40.0));
    assert_eq!(r.id().0, "rect-1");
    assert!(r.fill_color.is_none());
    assert_eq!(r.style, LineStyle::Solid);
}

// Clasificación: determinística — builder con fill color
#[test]
fn rectangle_builder() {
    let r = Rectangle::new("rect-2", ChartPoint::new(0, 0.0), ChartPoint::new(100, 100.0))
        .with_color([0.0, 0.0, 1.0, 1.0])
        .with_width(2.0)
        .with_style(LineStyle::Dashed)
        .with_fill([0.0, 0.0, 0.5, 0.3]);
    assert_eq!(r.color, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(r.width, 2.0);
    assert_eq!(r.style, LineStyle::Dashed);
    assert!(r.fill_color.is_some());
}

// Clasificación: determinística — move_by desplaza ambas esquinas
#[test]
fn rectangle_move_by() {
    let mut r = Rectangle::new("rect-3", ChartPoint::new(100, 80.0), ChartPoint::new(300, 40.0));
    r.move_by(ChartPoint::new(50, 10.0));
    assert_eq!(r.top_left, ChartPoint::new(150, 90.0));
    assert_eq!(r.bottom_right, ChartPoint::new(350, 50.0));
}

// Clasificación: determinística — bounds desde las dos esquinas
#[test]
fn rectangle_bounds() {
    let r = Rectangle::new("rect-4", ChartPoint::new(300, 90.0), ChartPoint::new(100, 20.0));
    let b = r.bounds();
    assert_eq!(b.time_start, 100);
    assert_eq!(b.time_end, 300);
    assert_eq!(b.price_min, 20.0);
    assert_eq!(b.price_max, 90.0);
}

// Clasificación: determinística — width_ts y height_price calculan dimensiones
#[test]
fn rectangle_dimensions() {
    let r = Rectangle::new("rect-5", ChartPoint::new(100, 50.0), ChartPoint::new(400, 150.0));
    assert_eq!(r.width_ts(), 300);
    assert!((r.height_price() - 100.0).abs() < f64::EPSILON);
}

// Clasificación: determinística — width_ts con esquinas invertidas
#[test]
fn rectangle_dimensions_inverted() {
    let r = Rectangle::new("rect-6", ChartPoint::new(400, 150.0), ChartPoint::new(100, 50.0));
    assert_eq!(r.width_ts(), 300);
    assert!((r.height_price() - 100.0).abs() < f64::EPSILON);
}

// ===========================================================================
// FibonacciRetracement
// ===========================================================================

// Clasificación: determinística — construcción con niveles por defecto
#[test]
fn fib_retracement_construction() {
    let fib = FibonacciRetracement::new(
        "fib-1",
        ChartPoint::new(1000, 100.0),
        ChartPoint::new(2000, 200.0),
    );
    assert_eq!(fib.id().0, "fib-1");
    assert_eq!(fib.levels, vec![0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0]);
    assert_eq!(fib.style, LineStyle::Dashed);
}

// Clasificación: determinística — price_at_level interpola linealmente
#[test]
fn fib_retracement_price_at_level() {
    let fib = FibonacciRetracement::new(
        "fib-2",
        ChartPoint::new(0, 100.0),
        ChartPoint::new(1000, 200.0),
    );
    // Level 0.0 → start price
    assert!((fib.price_at_level(0.0) - 100.0).abs() < f64::EPSILON);
    // Level 1.0 → end price
    assert!((fib.price_at_level(1.0) - 200.0).abs() < f64::EPSILON);
    // Level 0.5 → midpoint
    assert!((fib.price_at_level(0.5) - 150.0).abs() < f64::EPSILON);
    // Level 0.382 → 138.2
    assert!((fib.price_at_level(0.382) - 138.2).abs() < 1e-10);
}

// Clasificación: determinística — level_prices retorna pares correctos
#[test]
fn fib_retracement_level_prices() {
    let fib = FibonacciRetracement::new(
        "fib-3",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(1000, 100.0),
    );
    let prices = fib.level_prices();
    assert_eq!(prices.len(), 7);
    assert!((prices[0].1).abs() < f64::EPSILON); // 0% → 0.0
    assert!((prices[6].1 - 100.0).abs() < f64::EPSILON); // 100% → 100.0
}

// Clasificación: determinística — builder con niveles custom
#[test]
fn fib_retracement_builder() {
    let fib = FibonacciRetracement::new(
        "fib-4",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(100, 100.0),
    )
    .with_color([1.0, 0.0, 0.0, 1.0])
    .with_width(2.0)
    .with_style(LineStyle::Solid)
    .with_levels(vec![0.0, 0.5, 1.0]);
    assert_eq!(fib.levels, vec![0.0, 0.5, 1.0]);
    assert_eq!(fib.style, LineStyle::Solid);
}

// Clasificación: determinística — move_by desplaza ambos anchor points
#[test]
fn fib_retracement_move_by() {
    let mut fib = FibonacciRetracement::new(
        "fib-5",
        ChartPoint::new(100, 50.0),
        ChartPoint::new(200, 100.0),
    );
    fib.move_by(ChartPoint::new(10, 5.0));
    assert_eq!(fib.start, ChartPoint::new(110, 55.0));
    assert_eq!(fib.end, ChartPoint::new(210, 105.0));
}

// ===========================================================================
// FibonacciExtension
// ===========================================================================

// Clasificación: determinística — construcción con niveles por defecto
#[test]
fn fib_extension_construction() {
    let ext = FibonacciExtension::new(
        "ext-1",
        ChartPoint::new(100, 50.0),
        ChartPoint::new(200, 100.0),
        ChartPoint::new(150, 70.0),
    );
    assert_eq!(ext.id().0, "ext-1");
    assert_eq!(ext.levels, vec![0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0, 1.272, 1.618]);
    assert_eq!(ext.style, LineStyle::Dashed);
}

// Clasificación: determinística — price_at_level usa fórmula C + (B-A)*level
#[test]
fn fib_extension_price_at_level() {
    let ext = FibonacciExtension::new(
        "ext-2",
        ChartPoint::new(0, 100.0),   // A
        ChartPoint::new(100, 200.0),  // B
        ChartPoint::new(200, 150.0),  // C
    );
    // Level 0.0 → C.price = 150.0
    assert!((ext.price_at_level(0.0) - 150.0).abs() < f64::EPSILON);
    // Level 1.0 → C.price + (B.price - A.price) = 150 + 100 = 250
    assert!((ext.price_at_level(1.0) - 250.0).abs() < f64::EPSILON);
    // Level 0.5 → 150 + 50 = 200
    assert!((ext.price_at_level(0.5) - 200.0).abs() < f64::EPSILON);
    // Level 1.618 → 150 + 161.8 = 311.8
    assert!((ext.price_at_level(1.618) - 311.8).abs() < 1e-10);
}

// Clasificación: determinística — move_by desplaza los tres puntos
#[test]
fn fib_extension_move_by() {
    let mut ext = FibonacciExtension::new(
        "ext-3",
        ChartPoint::new(100, 50.0),
        ChartPoint::new(200, 100.0),
        ChartPoint::new(150, 75.0),
    );
    ext.move_by(ChartPoint::new(10, 5.0));
    assert_eq!(ext.point_a, ChartPoint::new(110, 55.0));
    assert_eq!(ext.point_b, ChartPoint::new(210, 105.0));
    assert_eq!(ext.point_c, ChartPoint::new(160, 80.0));
}

// ===========================================================================
// Pitchfork
// ===========================================================================

// Clasificación: determinística — construcción con tres puntos
#[test]
fn pitchfork_construction() {
    let pf = Pitchfork::new(
        "pf-1",
        ChartPoint::new(100, 50.0),
        ChartPoint::new(200, 100.0),
        ChartPoint::new(300, 30.0),
    );
    assert_eq!(pf.id().0, "pf-1");
    assert_eq!(pf.style, LineStyle::Solid);
    assert!(!pf.selected);
}

// Clasificación: determinística — move_by desplaza los tres puntos
#[test]
fn pitchfork_move_by() {
    let mut pf = Pitchfork::new(
        "pf-2",
        ChartPoint::new(100, 50.0),
        ChartPoint::new(200, 100.0),
        ChartPoint::new(300, 30.0),
    );
    pf.move_by(ChartPoint::new(10, 5.0));
    assert_eq!(pf.point_a, ChartPoint::new(110, 55.0));
    assert_eq!(pf.point_b, ChartPoint::new(210, 105.0));
    assert_eq!(pf.point_c, ChartPoint::new(310, 35.0));
}

// Clasificación: determinística — bounds calcula AABB de los tres puntos
#[test]
fn pitchfork_bounds() {
    let pf = Pitchfork::new(
        "pf-3",
        ChartPoint::new(200, 80.0),
        ChartPoint::new(100, 30.0),
        ChartPoint::new(300, 100.0),
    );
    let b = pf.bounds();
    assert_eq!(b.time_start, 100);
    assert_eq!(b.time_end, 300);
    assert_eq!(b.price_min, 30.0);
    assert_eq!(b.price_max, 100.0);
}

// Clasificación: determinística — hit_test en la línea media (A → midpoint(B,C))
#[test]
fn pitchfork_hit_test_on_median() {
    let a = ChartPoint::new(0, 0.0);
    let b = ChartPoint::new(200, 100.0);
    let c = ChartPoint::new(200, -100.0);
    let pf = Pitchfork::new("pf-4", a, b, c);
    // Midpoint of B,C = (200, 0.0) → median goes from (0,0) to (200,0)
    let on_median = ChartPoint::new(100, 0.0);
    assert_eq!(pf.hit_test(on_median, 10.0), HitResult::Body);
}

// Clasificación: determinística — hit_test fuera de la línea media
#[test]
fn pitchfork_hit_test_miss() {
    let pf = Pitchfork::new(
        "pf-5",
        ChartPoint::new(0, 0.0),
        ChartPoint::new(200, 100.0),
        ChartPoint::new(200, -100.0),
    );
    let far = ChartPoint::new(100, 500.0);
    assert_eq!(pf.hit_test(far, 5.0), HitResult::Miss);
}

// ===========================================================================
// Ellipse
// ===========================================================================

// Clasificación: determinística — construcción con radio y centro
#[test]
fn ellipse_construction() {
    let e = Ellipse::new("ell-1", ChartPoint::new(500, 50.0), 100.0, 30.0);
    assert_eq!(e.id().0, "ell-1");
    assert_eq!(e.center, ChartPoint::new(500, 50.0));
    assert_eq!(e.radius_x, 100.0);
    assert_eq!(e.radius_y, 30.0);
    assert!(e.fill_color.is_none());
}

// Clasificación: determinística — bounds calcula AABB desde centro y radios
#[test]
fn ellipse_bounds() {
    let e = Ellipse::new("ell-2", ChartPoint::new(500, 50.0), 100.0, 30.0);
    let b = e.bounds();
    assert_eq!(b.time_start, 400);
    assert_eq!(b.time_end, 600);
    assert_eq!(b.price_min, 20.0);
    assert_eq!(b.price_max, 80.0);
}

// Clasificación: determinística — hit_test con punto en el centro
#[test]
fn ellipse_hit_test_center() {
    let e = Ellipse::new("ell-3", ChartPoint::new(500, 50.0), 100.0, 30.0);
    assert_eq!(e.hit_test(ChartPoint::new(500, 50.0), 1.0), HitResult::Body);
}

// Clasificación: determinística — hit_test con punto fuera
#[test]
fn ellipse_hit_test_outside() {
    let e = Ellipse::new("ell-4", ChartPoint::new(500, 50.0), 10.0, 10.0);
    let far = ChartPoint::new(999, 999.0);
    assert_eq!(e.hit_test(far, 1.0), HitResult::Miss);
}

// Clasificación: determinística — move_by desplaza el centro
#[test]
fn ellipse_move_by() {
    let mut e = Ellipse::new("ell-5", ChartPoint::new(100, 50.0), 20.0, 10.0);
    e.move_by(ChartPoint::new(50, 25.0));
    assert_eq!(e.center, ChartPoint::new(150, 75.0));
}

// Clasificación: determinística — builder methods
#[test]
fn ellipse_builder() {
    let e = Ellipse::new("ell-6", ChartPoint::new(0, 0.0), 50.0, 50.0)
        .with_color([1.0, 0.0, 0.0, 1.0])
        .with_width(3.0)
        .with_style(LineStyle::Dotted)
        .with_fill([0.5, 0.5, 0.5, 0.5]);
    assert_eq!(e.color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(e.width, 3.0);
    assert_eq!(e.style, LineStyle::Dotted);
    assert!(e.fill_color.is_some());
}

// ===========================================================================
// Path
// ===========================================================================

// Clasificación: determinística — construcción con puntos
#[test]
fn path_construction() {
    let pts = vec![
        ChartPoint::new(100, 10.0),
        ChartPoint::new(200, 50.0),
        ChartPoint::new(300, 30.0),
    ];
    let p = Path::new("path-1", pts);
    assert_eq!(p.id().0, "path-1");
    assert_eq!(p.points.len(), 3);
    assert!(!p.closed);
    assert!(p.fill_color.is_none());
}

// Clasificación: determinística — push agrega un punto
#[test]
fn path_push() {
    let mut p = Path::new("path-2", vec![]);
    p.push(ChartPoint::new(100, 10.0));
    p.push(ChartPoint::new(200, 20.0));
    assert_eq!(p.points.len(), 2);
}

// Clasificación: determinística — segment_count para camino abierto
#[test]
fn path_segment_count_open() {
    let p = Path::new(
        "path-3",
        vec![
            ChartPoint::new(0, 0.0),
            ChartPoint::new(100, 50.0),
            ChartPoint::new(200, 30.0),
            ChartPoint::new(300, 80.0),
        ],
    );
    // 4 points open → 3 segments
    assert_eq!(p.segment_count(), 3);
}

// Clasificación: determinística — segment_count para camino cerrado
#[test]
fn path_segment_count_closed() {
    let p = Path::new(
        "path-4",
        vec![
            ChartPoint::new(0, 0.0),
            ChartPoint::new(100, 50.0),
            ChartPoint::new(200, 30.0),
        ],
    )
    .with_closed(true);
    // 3 points closed → 3 segments
    assert_eq!(p.segment_count(), 3);
}

// Clasificación: determinística — segment_count con menos de 2 puntos
#[test]
fn path_segment_count_empty_or_single() {
    let empty = Path::new("path-5a", vec![]);
    assert_eq!(empty.segment_count(), 0);

    let single = Path::new("path-5b", vec![ChartPoint::new(0, 0.0)]);
    assert_eq!(single.segment_count(), 0);
}

// Clasificación: determinística — total_length calcula suma euclidiana
#[test]
fn path_total_length() {
    // Two points at distance 100 in timestamp, 0 in price
    let p = Path::new(
        "path-6",
        vec![ChartPoint::new(0, 50.0), ChartPoint::new(100, 50.0)],
    );
    assert!((p.total_length() - 100.0).abs() < f64::EPSILON);
}

// Clasificación: determinística — total_length incluye segmento de cierre
#[test]
fn path_total_length_closed() {
    let p = Path::new(
        "path-7",
        vec![
            ChartPoint::new(0, 0.0),
            ChartPoint::new(100, 0.0),
            ChartPoint::new(100, 100.0),
        ],
    )
    .with_closed(true);
    // Open path: 100 + 100 = 200
    // Closed adds: distance from (100,100) to (0,0) = sqrt(20000) ≈ 141.42
    let expected = 100.0 + 100.0 + (100.0_f64.powi(2) + 100.0_f64.powi(2)).sqrt();
    assert!((p.total_length() - expected).abs() < 1e-10);
}

// Clasificación: determinística — total_length con 0 o 1 puntos
#[test]
fn path_total_length_trivial() {
    let empty = Path::new("path-8a", vec![]);
    assert!((empty.total_length()).abs() < f64::EPSILON);

    let single = Path::new("path-8b", vec![ChartPoint::new(0, 0.0)]);
    assert!((single.total_length()).abs() < f64::EPSILON);
}

// Clasificación: determinística — bounds calcula AABB de todos los puntos
#[test]
fn path_bounds() {
    let p = Path::new(
        "path-9",
        vec![
            ChartPoint::new(300, 80.0),
            ChartPoint::new(100, 20.0),
            ChartPoint::new(200, 60.0),
        ],
    );
    let b = p.bounds();
    assert_eq!(b.time_start, 100);
    assert_eq!(b.time_end, 300);
    assert_eq!(b.price_min, 20.0);
    assert_eq!(b.price_max, 80.0);
}

// Clasificación: determinística — bounds con camino vacío retorna origen
#[test]
fn path_bounds_empty() {
    let p = Path::new("path-10", vec![]);
    let b = p.bounds();
    assert_eq!(b.time_start, 0);
    assert_eq!(b.time_end, 0);
    assert_eq!(b.price_min, 0.0);
    assert_eq!(b.price_max, 0.0);
}

// Clasificación: determinística — point retorna referencia por índice
#[test]
fn path_point() {
    let p = Path::new(
        "path-11",
        vec![ChartPoint::new(0, 0.0), ChartPoint::new(100, 50.0)],
    );
    assert_eq!(p.point(0), Some(&ChartPoint::new(0, 0.0)));
    assert_eq!(p.point(1), Some(&ChartPoint::new(100, 50.0)));
    assert_eq!(p.point(2), None);
}

// Clasificación: determinística — move_by desplaza todos los puntos
#[test]
fn path_move_by() {
    let mut p = Path::new(
        "path-12",
        vec![ChartPoint::new(100, 10.0), ChartPoint::new(200, 30.0)],
    );
    p.move_by(ChartPoint::new(50, 20.0));
    assert_eq!(p.points[0], ChartPoint::new(150, 30.0));
    assert_eq!(p.points[1], ChartPoint::new(250, 50.0));
}

// Clasificación: determinística — hit_test en segmento del camino
#[test]
fn path_hit_test_on_segment() {
    let p = Path::new(
        "path-13",
        vec![ChartPoint::new(0, 0.0), ChartPoint::new(200, 0.0)],
    );
    // Punto sobre el segmento horizontal
    assert_eq!(p.hit_test(ChartPoint::new(100, 0.0), 5.0), HitResult::Body);
}

// Clasificación: determinística — hit_test fuera del camino
#[test]
fn path_hit_test_miss() {
    let p = Path::new(
        "path-14",
        vec![ChartPoint::new(0, 0.0), ChartPoint::new(200, 0.0)],
    );
    assert_eq!(
        p.hit_test(ChartPoint::new(100, 100.0), 5.0),
        HitResult::Miss
    );
}

// Clasificación: determinística — hit_test con camino de múltiples segmentos
#[test]
fn path_hit_test_multi_segment() {
    let p = Path::new(
        "path-15",
        vec![
            ChartPoint::new(0, 0.0),
            ChartPoint::new(100, 100.0),
            ChartPoint::new(200, 0.0),
        ],
    );
    // Punto sobre el primer segmento
    assert_eq!(p.hit_test(ChartPoint::new(50, 50.0), 5.0), HitResult::Body);
    // Punto sobre el segundo segmento
    assert_eq!(p.hit_test(ChartPoint::new(150, 50.0), 5.0), HitResult::Body);
    // Punto lejos de ambos
    assert_eq!(
        p.hit_test(ChartPoint::new(100, 0.0), 5.0),
        HitResult::Miss
    );
}

// Clasificación: determinística — builder methods aplican correctamente
#[test]
fn path_builder() {
    let p = Path::new("path-16", vec![])
        .with_color([1.0, 0.0, 0.0, 1.0])
        .with_width(3.0)
        .with_style(LineStyle::Dashed)
        .with_closed(true)
        .with_fill([0.5, 0.5, 0.5, 0.5]);
    assert_eq!(p.color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(p.width, 3.0);
    assert_eq!(p.style, LineStyle::Dashed);
    assert!(p.closed);
    assert!(p.fill_color.is_some());
}

// ===========================================================================
// Trait dispatch — todos los tipos implementan Drawing
// ===========================================================================

// Clasificación: determinística — verificación de trait objects para todos los tipos
#[test]
fn all_types_implement_drawing_trait() {
    let tl: Box<dyn Drawing> = Box::new(TrendLine::new("t", ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0)));
    let ar: Box<dyn Drawing> = Box::new(Arrow::new("a", ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0)));
    let ray: Box<dyn Drawing> = Box::new(Ray::new("r", ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0)));
    let seg: Box<dyn Drawing> = Box::new(Segment::new("s", ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0)));
    let txt: Box<dyn Drawing> = Box::new(TextDrawing::new("x", ChartPoint::new(0, 0.0), "t"));
    let img: Box<dyn Drawing> = Box::new(ImageDrawing::new("i", ChartPoint::new(0, 0.0), "f.png"));
    let lbl: Box<dyn Drawing> = Box::new(LabelDrawing::new("l", ChartPoint::new(0, 0.0), "L"));
    let hl: Box<dyn Drawing> = Box::new(HorizontalLine::new("h", 50.0));
    let vl: Box<dyn Drawing> = Box::new(VerticalLine::new("v", 100));
    let rect: Box<dyn Drawing> = Box::new(Rectangle::new("rc", ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0)));
    let fib: Box<dyn Drawing> = Box::new(FibonacciRetracement::new("f", ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0)));
    let ext: Box<dyn Drawing> = Box::new(FibonacciExtension::new("e", ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0), ChartPoint::new(0, 0.5)));
    let pf: Box<dyn Drawing> = Box::new(Pitchfork::new("p", ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0), ChartPoint::new(1, -1.0)));
    let ell: Box<dyn Drawing> = Box::new(Ellipse::new("el", ChartPoint::new(50, 50.0), 10.0, 10.0));
    let path: Box<dyn Drawing> = Box::new(Path::new("pa", vec![ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0)]));

    // All should produce valid bounds and not panic
    let drawings: Vec<&dyn Drawing> = vec![
        &*tl, &*ar, &*ray, &*seg, &*txt, &*img, &*lbl,
        &*hl, &*vl, &*rect, &*fib, &*ext, &*pf, &*ell, &*path,
    ];
    for d in &drawings {
        let _ = d.bounds();
        let _ = d.hit_test(ChartPoint::new(50, 50.0), 10.0);
        assert!(!d.is_selected());
    }
}

// Clasificación: determinística — type_name retorna nombre válido
#[test]
fn type_name_returns_correct_type() {
    let tl = TrendLine::new("t", ChartPoint::new(0, 0.0), ChartPoint::new(1, 1.0));
    let d: &dyn Drawing = &tl;
    assert!(d.type_name().contains("TrendLine"));
}

// ===========================================================================
// HitResult enum
// ===========================================================================

// Clasificación: determinística — HitResult variants son distinguibles
#[test]
fn hit_result_variants() {
    assert_ne!(HitResult::Miss, HitResult::Body);
    assert_ne!(HitResult::Miss, HitResult::ControlPoint(0));
    assert_ne!(HitResult::Body, HitResult::ControlPoint(0));
    assert_eq!(HitResult::ControlPoint(1), HitResult::ControlPoint(1));
    assert_ne!(HitResult::ControlPoint(0), HitResult::ControlPoint(1));
}

// ===========================================================================
// Edge cases — saturating arithmetic
// ===========================================================================

// Clasificación: determinística — move_by con saturación en u64 overflow
#[test]
fn trendline_move_by_saturating_overflow() {
    let mut tl = TrendLine::new(
        "edge-1",
        ChartPoint::new(u64::MAX - 50, 0.0),
        ChartPoint::new(u64::MAX - 10, 50.0),
    );
    // Adding 100 should saturate to u64::MAX
    tl.move_by(ChartPoint::new(100, 0.0));
    assert_eq!(tl.start.timestamp, u64::MAX);
    assert_eq!(tl.end.timestamp, u64::MAX);
}

// Clasificación: determinística — Path vacío hit_test retorna Miss
#[test]
fn path_hit_test_empty() {
    let p = Path::new("edge-2", vec![]);
    assert_eq!(p.hit_test(ChartPoint::new(0, 0.0), 10.0), HitResult::Miss);
}

// Clasificación: determinística — Path con un solo punto hit_test retorna Miss
#[test]
fn path_hit_test_single_point() {
    let p = Path::new("edge-3", vec![ChartPoint::new(100, 50.0)]);
    assert_eq!(p.hit_test(ChartPoint::new(100, 50.0), 10.0), HitResult::Miss);
}
