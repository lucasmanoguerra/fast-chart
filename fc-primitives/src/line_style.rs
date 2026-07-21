//! Canonical line rendering style.
//!
//! Used by drawing tools, price lines, theme tokens, and render commands.
//! Single definition — re-exported by fc-domain, fc-theme, and fc-render.

/// Line rendering style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LineStyle {
    /// Solid line: `─────`
    Solid,
    /// Dashed line: `─ ─ ─`
    Dashed,
    /// Dotted line: `· · ·`
    Dotted,
}

impl Default for LineStyle {
    fn default() -> Self {
        Self::Solid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_solid() {
        assert_eq!(LineStyle::default(), LineStyle::Solid);
    }

    #[test]
    fn all_variants_distinct() {
        assert_ne!(LineStyle::Solid, LineStyle::Dashed);
        assert_ne!(LineStyle::Dashed, LineStyle::Dotted);
        assert_ne!(LineStyle::Solid, LineStyle::Dotted);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrip() {
        let json = serde_json::to_string(&LineStyle::Dashed).unwrap();
        let back: LineStyle = serde_json::from_str(&json).unwrap();
        assert_eq!(back, LineStyle::Dashed);
    }
}
