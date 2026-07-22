//! Color primitives for chart rendering.

/// A named color as RGBA with channels in `[0.0, 1.0]`.
///
/// # Examples
///
/// ```
/// use fc_primitives::color::Rgba;
///
/// let opaque = Rgba::rgb(1.0, 0.0, 0.0);
/// assert_eq!(opaque, Rgba(1.0, 0.0, 0.0, 1.0));
///
/// let transparent = Rgba::new(0.0, 1.0, 0.0, 0.5);
/// assert_eq!(transparent.3, 0.5);
///
/// let from_hex = Rgba::from_hex(0xFF0000FF);
/// assert!((from_hex.0 - 1.0).abs() < f64::EPSILON);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba(pub f64, pub f64, pub f64, pub f64);

impl Rgba {
    /// Create a new RGBA color.
    #[inline]
    pub const fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self(r, g, b, a)
    }

    /// Fully opaque convenience constructor.
    #[inline]
    pub const fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self(r, g, b, 1.0)
    }

    /// From `0xRRGGBBAA` hex integer.
    #[inline]
    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 24) & 0xFF) as f64 / 255.0;
        let g = ((hex >> 16) & 0xFF) as f64 / 255.0;
        let b = ((hex >> 8) & 0xFF) as f64 / 255.0;
        let a = (hex & 0xFF) as f64 / 255.0;
        Self(r, g, b, a)
    }

    /// Convert to `[f32; 4]` array for GPU rendering.
    #[inline]
    pub fn to_f32_array(self) -> [f32; 4] {
        [self.0 as f32, self.1 as f32, self.2 as f32, self.3 as f32]
    }

    /// Blend with another color using the given alpha factor.
    #[inline]
    pub fn blend(self, other: Self, factor: f64) -> Self {
        Self(
            self.0 + (other.0 - self.0) * factor,
            self.1 + (other.1 - self.1) * factor,
            self.2 + (other.2 - self.2) * factor,
            self.3 + (other.3 - self.3) * factor,
        )
    }

    /// With alpha overridden.
    #[inline]
    pub fn with_alpha(self, alpha: f64) -> Self {
        Self(self.0, self.1, self.2, alpha)
    }
}

impl Default for Rgba {
    fn default() -> Self {
        Self(0.0, 0.0, 0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Clasificación: determinística — verifica rgba_new
    #[test]
    fn rgba_new() {
        let c = Rgba::new(0.1, 0.2, 0.3, 0.4);
        assert_eq!(c.0, 0.1);
        assert_eq!(c.1, 0.2);
        assert_eq!(c.2, 0.3);
        assert_eq!(c.3, 0.4);
    }

    // Clasificación: determinística — verifica rgba_rgb
    #[test]
    fn rgba_rgb() {
        let c = Rgba::rgb(1.0, 0.5, 0.0);
        assert_eq!(c, Rgba(1.0, 0.5, 0.0, 1.0));
    }

    // Clasificación: determinística — verifica rgba_from_hex
    #[test]
    fn rgba_from_hex() {
        let c = Rgba::from_hex(0xFF0000FF);
        assert!((c.0 - 1.0).abs() < f64::EPSILON);
        assert!((c.1).abs() < f64::EPSILON);
        assert!((c.2).abs() < f64::EPSILON);
        assert!((c.3 - 1.0).abs() < f64::EPSILON);
    }

    // Clasificación: determinística — verifica rgba_default
    #[test]
    fn rgba_default() {
        let c = Rgba::default();
        assert_eq!(c, Rgba(0.0, 0.0, 0.0, 1.0));
    }

    // Clasificación: determinística — verifica rgba_to_f32_array
    #[test]
    fn rgba_to_f32_array() {
        let c = Rgba(0.5, 0.25, 0.125, 0.75);
        let arr = c.to_f32_array();
        assert_eq!(arr, [0.5f32, 0.25, 0.125, 0.75]);
    }

    // Clasificación: determinística — verifica conteo de shortcuts registrado
    #[test]
    fn rgba_blend() {
        let a = Rgba(0.0, 0.0, 0.0, 1.0);
        let b = Rgba(1.0, 1.0, 1.0, 1.0);
        let blended = a.blend(b, 0.5);
        assert!((blended.0 - 0.5).abs() < f64::EPSILON);
    }

    // Clasificación: determinística — verifica rgba_with_alpha
    #[test]
    fn rgba_with_alpha() {
        let c = Rgba(1.0, 0.0, 0.0, 1.0).with_alpha(0.5);
        assert_eq!(c, Rgba(1.0, 0.0, 0.0, 0.5));
    }
}
