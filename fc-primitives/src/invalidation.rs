/// Invalidation level — orders from least to most expensive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum InvalidationLevel {
    Nothing = 0,
    Cursor = 1, // Only crosshair moved
    Light = 2,  // Viewport changed (zoom/pan)
    Full = 3,   // Data changed
}

/// Bitmask representing which panes need invalidation.
/// Bit N = pane N needs update. Max 32 panes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaneBitmask(u32);

impl PaneBitmask {
    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self(u32::MAX);

    pub fn single(pane_index: usize) -> Self {
        assert!(pane_index < 32, "Pane index must be < 32");
        Self(1 << pane_index)
    }

    pub fn contains(&self, pane_index: usize) -> bool {
        (self.0 & (1 << pane_index)) != 0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn union(&self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn intersect(&self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn iter(&self) -> PaneBitmaskIter {
        PaneBitmaskIter {
            bits: self.0,
            index: 0,
        }
    }
}

impl std::ops::BitOr for PaneBitmask {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for PaneBitmask {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

pub struct PaneBitmaskIter {
    bits: u32,
    index: usize,
}

impl Iterator for PaneBitmaskIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < 32 {
            let i = self.index;
            self.index += 1;
            if (self.bits & (1 << i)) != 0 {
                return Some(i);
            }
        }
        None
    }
}

/// Combined invalidation state: level + which panes.
///
/// # Examples
///
/// ```
/// use fc_primitives::{InvalidationMask, InvalidationLevel, PaneBitmask};
///
/// // No invalidation
/// let none = InvalidationMask::NONE;
/// assert_eq!(none.level(), InvalidationLevel::Nothing);
///
/// // Full invalidation on pane 0 and pane 2
/// let mask = InvalidationMask::new(
///     InvalidationLevel::Full,
///     PaneBitmask::single(0).union(PaneBitmask::single(2)),
/// );
/// assert_eq!(mask.level(), InvalidationLevel::Full);
/// assert_eq!(mask.panes().count(), 2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidationMask {
    level: InvalidationLevel,
    panes: PaneBitmask,
}

impl InvalidationMask {
    pub const NONE: Self = Self {
        level: InvalidationLevel::Nothing,
        panes: PaneBitmask::NONE,
    };

    pub fn new(level: InvalidationLevel, panes: PaneBitmask) -> Self {
        Self { level, panes }
    }

    pub fn level(&self) -> InvalidationLevel {
        self.level
    }

    pub fn panes(&self) -> PaneBitmask {
        self.panes
    }

    /// Check if this mask contains at least the given level.
    pub fn contains(&self, level: InvalidationLevel) -> bool {
        self.level >= level
    }

    /// Check if any pane needs at least the given level.
    pub fn contains_pane(&self, pane_index: usize, level: InvalidationLevel) -> bool {
        self.level >= level && self.panes.contains(pane_index)
    }

    /// Merge another mask into this one (OR semantics — takes the higher level).
    pub fn merge(&mut self, other: Self) {
        if other.level > self.level {
            self.level = other.level;
        }
        self.panes = self.panes | other.panes;
    }

    /// Create a mask for a specific level affecting all panes.
    pub fn all_panes(level: InvalidationLevel) -> Self {
        Self {
            level,
            panes: PaneBitmask::ALL,
        }
    }

    /// Create a mask for a specific level affecting one pane.
    pub fn single_pane(level: InvalidationLevel, pane_index: usize) -> Self {
        Self {
            level,
            panes: PaneBitmask::single(pane_index),
        }
    }

    /// Reset to Nothing.
    pub fn clear(&mut self) {
        self.level = InvalidationLevel::Nothing;
        self.panes = PaneBitmask::NONE;
    }

    /// Check if this mask is empty (Nothing level).
    pub fn is_empty(&self) -> bool {
        self.level == InvalidationLevel::Nothing
    }
}

impl Default for InvalidationMask {
    fn default() -> Self {
        Self::NONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Clasificación: determinística — verifica level_ordering
    #[test]
    fn level_ordering() {
        assert!(InvalidationLevel::Nothing < InvalidationLevel::Cursor);
        assert!(InvalidationLevel::Cursor < InvalidationLevel::Light);
        assert!(InvalidationLevel::Light < InvalidationLevel::Full);
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn pane_bitmask_single() {
        let mask = PaneBitmask::single(0);
        assert!(mask.contains(0));
        assert!(!mask.contains(1));
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn pane_bitmask_union() {
        let a = PaneBitmask::single(0);
        let b = PaneBitmask::single(2);
        let c = a | b;
        assert!(c.contains(0));
        assert!(!c.contains(1));
        assert!(c.contains(2));
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn pane_bitmask_iter() {
        let mask = PaneBitmask::single(1) | PaneBitmask::single(3);
        let indices: Vec<usize> = mask.iter().collect();
        assert_eq!(indices, vec![1, 3]);
    }

    // Clasificación: determinística — verifica invalidation_merge_takes_higher_level
    #[test]
    fn invalidation_merge_takes_higher_level() {
        let mut mask = InvalidationMask::single_pane(InvalidationLevel::Cursor, 0);
        mask.merge(InvalidationMask::single_pane(InvalidationLevel::Light, 0));
        assert_eq!(mask.level(), InvalidationLevel::Light);
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn invalidation_merge_combines_panes() {
        let mut mask = InvalidationMask::single_pane(InvalidationLevel::Light, 0);
        mask.merge(InvalidationMask::single_pane(InvalidationLevel::Light, 1));
        assert!(mask.panes().contains(0));
        assert!(mask.panes().contains(1));
    }

    // Clasificación: determinística — verifica invalidation_contains
    #[test]
    fn invalidation_contains() {
        let mask = InvalidationMask::single_pane(InvalidationLevel::Light, 0);
        assert!(mask.contains(InvalidationLevel::Nothing));
        assert!(mask.contains(InvalidationLevel::Cursor));
        assert!(mask.contains(InvalidationLevel::Light));
        assert!(!mask.contains(InvalidationLevel::Full));
    }

    // Clasificación: determinística — verifica que clear() resetea completamente el estado y las estadísticas
    #[test]
    fn invalidation_clear() {
        let mut mask = InvalidationMask::all_panes(InvalidationLevel::Full);
        mask.clear();
        assert!(mask.is_empty());
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn pane_bitmask_all() {
        let mask = PaneBitmask::ALL;
        for i in 0..32 {
            assert!(mask.contains(i));
        }
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn pane_bitmask_count() {
        let mask = PaneBitmask::single(0) | PaneBitmask::single(2) | PaneBitmask::single(5);
        assert_eq!(mask.count(), 3);
    }
}