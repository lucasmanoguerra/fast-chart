//! Animation: `AnimatedValue` and `AnimationEngine` for smooth transitions.
//!
//! Demonstrates: creating animations, easing functions, retargeting,
//! the engine for managing multiple named tracks, and garbage collection.

use fc_app::animation::{AnimatedValue, AnimationEngine, Easing};

pub fn run() {
    // Single animated value: linear interpolation from 50k → 52k over 500ms
    let mut price_anim = AnimatedValue::new(50_000.0, 52_000.0, 500.0, Easing::Linear);
    price_anim.update(250.0);
    println!("Linear price at 250ms: {:.2}", price_anim.current());

    // Easing comparison at t = 0.5
    println!("\nEasing comparison at 50% progress:");
    let easings: [(&str, Easing); 5] = [
        ("Linear", Easing::Linear),
        ("EaseIn", Easing::EaseIn),
        ("EaseOut", Easing::EaseOut),
        ("EaseInOut", Easing::EaseInOut),
        (
            "Spring",
            Easing::Spring {
                stiffness: 200.0,
                damping: 5.0,
            },
        ),
    ];
    for (name, easing) in &easings {
        let mut a = AnimatedValue::new(0.0, 100.0, 1000.0, *easing);
        a.update(500.0);
        println!("  {name}: {:.2}", a.current());
    }

    // AnimationEngine: manage multiple tracks
    let mut engine = AnimationEngine::new();
    engine.animate(
        "scroll",
        AnimatedValue::new(0.0, 500.0, 600.0, Easing::EaseInOut),
    );
    engine.animate(
        "fade",
        AnimatedValue::new(1.0, 0.0, 300.0, Easing::EaseOut),
    );

    // Tick the clock
    engine.update(200.0);
    println!("\nEngine at 200ms:");
    println!("  scroll = {:.2}", engine.value("scroll").unwrap());
    println!("  fade   = {:.2}", engine.value("fade").unwrap());
    println!("  active = {}", engine.active_count());

    // Fade completes at 300ms, scroll still running
    engine.update(200.0);
    println!("\nEngine at 400ms:");
    println!("  fade complete = {}", engine.is_complete("fade"));
    println!("  scroll complete = {}", engine.is_complete("scroll"));

    // Retarget: change destination mid-animation
    let mut zoom = AnimatedValue::new(1.0, 5.0, 400.0, Easing::EaseInOut);
    zoom.update(200.0);
    println!("\nZoom retarget:");
    println!("  before retarget: {:.2}", zoom.current());
    zoom.retarget(3.0);
    zoom.update(100.0);
    println!("  after retarget + 100ms: {:.2}", zoom.current());

    // GC: clean up completed animations
    engine.update(400.0);
    println!("\nEngine after all complete:");
    println!("  active before gc = {}", engine.active_count());
    engine.gc();
    println!("  active after gc  = {}", engine.active_count());
}
