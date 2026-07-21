// ---------------------------------------------------------------------------
// fc-drawing — unified Drawing trait and hit-testing
// ---------------------------------------------------------------------------

pub mod trait_def;
pub mod hit;
pub mod bounds;
pub mod impls;

pub use trait_def::Drawing;
pub use hit::{HitResult, default_aabb_hit_test};
pub use bounds::DrawingBounds;
