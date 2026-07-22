use crate::cache::Cache;

/// Cache key for grid lines.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GridKey {
    pub width: u32,
    pub height: u32,
    pub price_min_bits: u64,
    pub price_max_bits: u64,
    pub time_min_bits: u64,
    pub time_max_bits: u64,
}

impl GridKey {
    pub fn new(width: u32, height: u32, price_range: (f64, f64), time_range: (f64, f64)) -> Self {
        Self {
            width,
            height,
            price_min_bits: price_range.0.to_bits(),
            price_max_bits: price_range.1.to_bits(),
            time_min_bits: time_range.0.to_bits(),
            time_max_bits: time_range.1.to_bits(),
        }
    }
}

/// Cached grid lines.
#[derive(Debug, Clone)]
pub struct GridEntry {
    pub horizontal_lines: Vec<f32>,
    pub vertical_lines: Vec<f32>,
}

pub struct GridCache {
    inner: Cache<GridKey, GridEntry>,
}

impl GridCache {
    pub fn new(max_entries: usize) -> Self {
        Self { inner: Cache::new(max_entries) }
    }

    pub fn get(&mut self, key: &GridKey) -> Option<&GridEntry> {
        self.inner.get(key)
    }

    pub fn insert(&mut self, key: GridKey, entry: GridEntry) {
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

    fn make_key() -> GridKey {
        GridKey::new(800, 600, (100.0, 200.0), (0.0, 1000.0))
    }

    fn make_entry() -> GridEntry {
        GridEntry { horizontal_lines: vec![10.0, 20.0], vertical_lines: vec![100.0] }
    }

    // Clasificación: determinística — verifica new_cache_is_empty
    #[test]
    fn new_cache_is_empty() {
        let cache = GridCache::new(16);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    // Clasificación: determinística — round-trip insert/get — invariante básico
    #[test]
    fn insert_and_get() {
        let mut cache = GridCache::new(16);
        let key = make_key();
        cache.insert(key.clone(), make_entry());
        assert!(cache.get(&key).is_some());
        assert_eq!(cache.len(), 1);
    }

    // Clasificación: determinística — edge case: key inexistente retorna None sin panic
    #[test]
    fn get_missing_returns_none() {
        let mut cache = GridCache::new(16);
        let key = make_key();
        assert!(cache.get(&key).is_none());
    }

    // Clasificación: determinística — verifica reset completo del estado
    #[test]
    fn clear_empties_cache() {
        let mut cache = GridCache::new(16);
        cache.insert(make_key(), make_entry());
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    // Clasificación: determinística — verifica política de evicción al exceder capacidad
    #[test]
    fn eviction_when_full() {
        let mut cache = GridCache::new(2);
        cache.insert(make_key(), make_entry());
        let k2 = GridKey::new(1024, 768, (50.0, 150.0), (0.0, 500.0));
        cache.insert(k2, make_entry());
        let k3 = GridKey::new(1920, 1080, (0.0, 300.0), (0.0, 2000.0));
        cache.insert(k3, make_entry());
        assert_eq!(cache.len(), 2);
    }
}
