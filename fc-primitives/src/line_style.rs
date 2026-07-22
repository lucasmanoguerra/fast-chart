//! Canonical line rendering style.
//!
//! Used by drawing tools, price lines, theme tokens, and render commands.
//! Single definition — re-exported by fc-domain, fc-theme, and fc-render.

/// Line rendering style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default)]
pub enum LineStyle {
    /// Solid line: `─────`
    #[default]
    Solid,
    /// Dashed line: `─ ─ ─`
    Dashed,
    /// Dotted line: `· · ·`
    Dotted,
}


#[cfg(test)]
mod tests {
    use super::*;

    // Clasificación: determinística — verifica default_is_solid
    #[test]
    fn default_is_solid() {
        assert_eq!(LineStyle::default(), LineStyle::Solid);
    }

    // Clasificación: determinística — verifica all_variants_distinct
    #[test]
    fn all_variants_distinct() {
        assert_ne!(LineStyle::Solid, LineStyle::Dashed);
        assert_ne!(LineStyle::Dashed, LineStyle::Dotted);
        assert_ne!(LineStyle::Solid, LineStyle::Dotted);
    }

    #[cfg(feature = "serde")]
    // Clasificación: determinística — verifica serde_roundtrip
    #[test]
    fn serde_roundtrip() {
        let json = serde_json::to_string(&LineStyle::Dashed).unwrap();
        let back: LineStyle = serde_json::from_str(&json).unwrap();
        assert_eq!(back, LineStyle::Dashed);
    }
}
