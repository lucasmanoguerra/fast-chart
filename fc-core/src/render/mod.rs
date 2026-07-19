//! Rendering engine re-exported from `fc-render`.
//!
//! All rendering types, traits, and commands are defined in the `fc-render`
//! crate and re-exported here for backward compatibility.

pub use fc_render::*;

#[cfg(feature = "sessions")]
pub mod session;
