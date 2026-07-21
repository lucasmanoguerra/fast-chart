use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use fc_render::commands::DrawCommand;
use fc_render::pixel_perfect::{pixel_perfect_rect, snap_point, PixelPerfect};
use fc_sessions::Session;
use fc_theme::{ChartTheme, Rgba, ThemeToken};
use fc_renderer_wgpu::scissor::ScissorRect;
use fc_renderer_wgpu::types::Vertex;
use fc_renderer_wgpu::vertex_gen::{generate_sorted_vertices, generate_vertices};

const SURFACE_W: f32 = 1920.0;
const SURFACE_H: f32 = 1080.0;
const BULLISH: [f32; 4] = [0.26, 0.70, 0.44, 1.0];
const BEARISH: [f32; 4] = [0.85, 0.26, 0.26, 1.0];
const GRID: [f32; 4] = [0.20, 0.20, 0.22, 1.0];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Generate synthetic candlestick-style DrawCommands (rect body + line wick).
fn make_candle_commands(n: usize) -> Vec<DrawCommand> {
    let mut cmds = Vec::with_capacity(n * 2);
    for i in 0..n {
        let x = i as f32 * 8.0;
        let open = 100.0 + (i as f32 * 0.3).sin() * 50.0;
        let close = open + ((i as f32 * 0.7).cos() * 10.0);
        let high = open.max(close) + 3.0;
        let low = open.min(close) - 3.0;
        let color = if close >= open { BULLISH } else { BEARISH };

        // Candle body (filled rect)
        cmds.push(DrawCommand::filled_rect(
            x,
            close.min(open),
            6.0,
            (open - close).abs().max(1.0),
            color,
            0,
        ));
        // Wick (line from low to high)
        cmds.push(DrawCommand::line(
            x + 3.0,
            high,
            x + 3.0,
            low,
            color,
            1.0,
            0,
        ));
    }
    cmds
}

/// Generate mixed DrawCommands with varied z-indices.
fn make_mixed_commands(n: usize) -> Vec<DrawCommand> {
    let mut cmds = Vec::with_capacity(n);
    for i in 0..n {
        let z = (i % 5) as i32;
        let x = (i % 100) as f32 * 10.0;
        let y = (i / 100) as f32 * 10.0;
        match i % 4 {
            0 => cmds.push(DrawCommand::filled_rect(x, y, 8.0, 6.0, GRID, z)),
            1 => cmds.push(DrawCommand::line(x, y, x + 20.0, y + 10.0, GRID, 1.0, z)),
            2 => cmds.push(DrawCommand::filled_circle(x + 5.0, y + 5.0, 3.0, GRID, z)),
            _ => cmds.push(DrawCommand::filled_triangle(
                x, y, x + 8.0, y, x + 4.0, y + 6.0, GRID, z,
            )),
        }
    }
    cmds
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

fn bench_vertex_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("vertex_generation");
    for size in [100, 1_000, 10_000, 100_000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let cmds = make_candle_commands(size);
            b.iter(|| {
                let mut verts: Vec<Vertex> = Vec::new();
                let mut inds: Vec<u32> = Vec::new();
                for cmd in &cmds {
                    generate_vertices(cmd, SURFACE_W, SURFACE_H, &mut verts, &mut inds);
                }
                (verts.len(), inds.len())
            });
        });
    }
    group.finish();
}

fn bench_draw_command_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_command_sorting");
    for size in [100, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let cmds = make_mixed_commands(size);
            b.iter(|| {
                let mut verts: Vec<Vertex> = Vec::new();
                let mut inds: Vec<u32> = Vec::new();
                generate_sorted_vertices(&cmds, SURFACE_W, SURFACE_H, &mut verts, &mut inds);
                (verts.len(), inds.len())
            });
        });
    }
    group.finish();
}

fn bench_scissor_intersection(c: &mut Criterion) {
    let mut group = c.benchmark_group("scissor_intersection");

    group.bench_function("overlapping", |bencher| {
        let rect_a = ScissorRect::new(0, 0, 800, 600);
        let rect_b = ScissorRect::new(200, 150, 600, 400);
        bencher.iter(|| rect_a.intersect(&rect_b));
    });

    group.bench_function("non_overlapping", |bencher| {
        let rect_a = ScissorRect::new(0, 0, 100, 100);
        let rect_b = ScissorRect::new(500, 500, 100, 100);
        bencher.iter(|| rect_a.intersect(&rect_b));
    });

    group.bench_function("contained", |bencher| {
        let outer = ScissorRect::new(0, 0, 1920, 1080);
        let inner = ScissorRect::new(100, 100, 500, 300);
        bencher.iter(|| outer.intersect(&inner));
    });

    group.finish();
}

fn bench_pixel_perfect(c: &mut Criterion) {
    let mut group = c.benchmark_group("pixel_perfect");

    group.bench_function("snap_f32", |b| {
        b.iter(|| {
            let v = 3.7_f32;
            v.snap()
        });
    });

    group.bench_function("snap_point", |b| {
        b.iter(|| {
            snap_point(fc_render::coordinates::ScreenPoint { x: 3.7, y: 12.3 })
        });
    });

    group.bench_function("pixel_perfect_rect", |b| {
        b.iter(|| pixel_perfect_rect(3.2, 5.7, 10.3, 20.9));
    });

    group.finish();
}

fn bench_theme_hot_swap(c: &mut Criterion) {
    let mut group = c.benchmark_group("theme_hot_swap");

    group.bench_function("set_color", |b| {
        b.iter_batched(
            || ChartTheme::dark(),
            |mut theme| {
                theme.set_color(ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0));
                theme
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("get_color", |b| {
        let theme = ChartTheme::dark();
        b.iter(|| theme.get_color(ThemeToken::Bullish));
    });

    group.bench_function("set_all_tokens", |b| {
        b.iter_batched(
            || ChartTheme::dark(),
            |mut theme| {
                let tokens = [
                    ThemeToken::Background,
                    ThemeToken::GridLine,
                    ThemeToken::Bullish,
                    ThemeToken::Bearish,
                    ThemeToken::CrosshairLine,
                    ThemeToken::TextPrimary,
                    ThemeToken::Watermark,
                ];
                for token in tokens {
                    theme.set_color(token, Rgba::rgb(0.5, 0.5, 0.5));
                }
                theme
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache");

    group.bench_function("construct_1k", |b| {
        b.iter(|| fc_renderer_wgpu::cache::GpuCache::new(1024));
    });

    group.bench_function("construct_1m", |b| {
        b.iter(|| fc_renderer_wgpu::cache::GpuCache::new(1_000_000));
    });

    group.bench_function("capacity_check", |b| {
        let cache = fc_renderer_wgpu::cache::GpuCache::new(1024);
        b.iter(|| cache.capacity());
    });

    group.finish();
}

fn bench_session_containment(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_containment");

    group.bench_function("us_regular_inside", |b| {
        let session = Session::new("US Regular", 14, 30, 21, 0);
        b.iter(|| session.contains_utc(16, 0));
    });

    group.bench_function("us_regular_outside", |b| {
        let session = Session::new("US Regular", 14, 30, 21, 0);
        b.iter(|| session.contains_utc(8, 0));
    });

    group.bench_function("wrapped_session_inside", |b| {
        // After-hours wraps midnight: 21:00 – 01:00 UTC
        let session = Session::new("After-Hours", 21, 0, 1, 0);
        b.iter(|| session.contains_utc(23, 30));
    });

    group.bench_function("wrapped_session_outside", |b| {
        let session = Session::new("After-Hours", 21, 0, 1, 0);
        b.iter(|| session.contains_utc(10, 0));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_vertex_generation,
    bench_draw_command_sorting,
    bench_scissor_intersection,
    bench_pixel_perfect,
    bench_theme_hot_swap,
    bench_cache_operations,
    bench_session_containment,
);
criterion_main!(benches);
