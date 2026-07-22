use crate::cache::Cache;

/// Cache key for axis tick positions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AxisKey {
    pub axis_type: AxisType,
    pub range_min_bits: u64,
    pub range_max_bits: u64,
    pub available_pixels_bits: u32,
    pub font_size_bits: u32,
}

impl AxisKey {
    pub fn new(
        axis_type: AxisType,
        range_min: f64,
        range_max: f64,
        available_pixels: f32,
        font_size: f32,
    ) -> Self {
        Self {
            axis_type,
            range_min_bits: range_min.to_bits(),
            range_max_bits: range_max.to_bits(),
            available_pixels_bits: available_pixels.to_bits(),
            font_size_bits: font_size.to_bits(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AxisType {
    Price,
    Time,
}

/// Cached axis tick positions.
#[derive(Debug, Clone)]
pub struct AxisEntry {
    pub ticks: Vec<AxisTick>,
}

#[derive(Debug, Clone)]
pub struct AxisTick {
    pub position: f32,
    pub value: f64,
    pub label: String,
}

pub struct AxisCache {
    inner: Cache<AxisKey, AxisEntry>,
}

impl AxisCache {
    pub fn new(max_entries: usize) -> Self {
        Self { inner: Cache::new(max_entries) }
    }

    pub fn get(&mut self, key: &AxisKey) -> Option<&AxisEntry> {
        self.inner.get(key)
    }

    pub fn insert(&mut self, key: AxisKey, entry: AxisEntry) {
        self.inner.insert(key, entry);
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn hit_rate(&self) -> f64 {
        self.inner.hit_rate()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(axis: AxisType) -> AxisKey {
        AxisKey::new(axis, 0.0, 100.0, 500.0, 14.0)
    }

    fn make_entry() -> AxisEntry {
        AxisEntry { ticks: vec![AxisTick { position: 0.0, value: 0.0, label: "0".into() }] }
    }

    // Clasificación: determinística — verifica new_cache_is_empty
    #[test]
    fn new_cache_is_empty() {
        let cache = AxisCache::new(16);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    // Clasificación: determinística — round-trip insert/get — invariante básico
    #[test]
    fn insert_and_get() {
        let mut cache = AxisCache::new(16);
        let key = make_key(AxisType::Price);
        cache.insert(key.clone(), make_entry());
        assert!(cache.get(&key).is_some());
        assert_eq!(cache.len(), 1);
    }

    // Clasificación: determinística — edge case: key inexistente retorna None sin panic
    #[test]
    fn get_missing_returns_none() {
        let mut cache = AxisCache::new(16);
        let key = make_key(AxisType::Time);
        assert!(cache.get(&key).is_none());
    }

    // Clasificación: determinística — verifica reset completo del estado
    #[test]
    fn clear_empties_cache() {
        let mut cache = AxisCache::new(16);
        cache.insert(make_key(AxisType::Price), make_entry());
        let _ = cache.get(&make_key(AxisType::Time));
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    // Clasificación: determinística — verifica cálculo de métrica hit/miss
    #[test]
    fn hit_rate_tracks_correctly() {
        let mut cache = AxisCache::new(16);
        let key = make_key(AxisType::Price);
        cache.insert(key.clone(), make_entry());
        let _ = cache.get(&key);
        let _ = cache.get(&make_key(AxisType::Time));
        assert!((cache.hit_rate() - 0.5).abs() < f64::EPSILON);
    }

    // Clasificación: determinística — verifica política de evicción al exceder capacidad
    #[test]
    fn eviction_when_full() {
        let mut cache = AxisCache::new(2);
        cache.insert(make_key(AxisType::Price), make_entry());
        cache.insert(make_key(AxisType::Time), make_entry());
        // Inserting a third should evict the first
        let k3 = AxisKey::new(AxisType::Price, 10.0, 90.0, 400.0, 12.0);
        cache.insert(k3.clone(), make_entry());
        assert_eq!(cache.len(), 2);
    }
}
