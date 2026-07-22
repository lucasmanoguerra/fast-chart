use std::time::Instant;

use crate::cache::Cache;

/// Cache key for geometry (vertex data).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GeometryKey {
    /// Series identifier.
    pub series_id: u32,
    /// Viewport hash (time range + price range).
    pub viewport_hash: u64,
    /// Data hash (content hash of the data slice).
    pub data_hash: u64,
}

/// Cached geometry (vertex/index data).
#[derive(Debug, Clone)]
pub struct GeometryEntry {
    /// Vertex data (position, color, tex_coord).
    pub vertices: Vec<[f32; 8]>,
    /// Index data.
    pub indices: Vec<u32>,
    /// When this entry was created.
    pub created_at: Instant,
}

/// Caches vertex geometry for series rendering.
pub struct GeometryCache {
    inner: Cache<GeometryKey, GeometryEntry>,
}

impl GeometryCache {
    pub fn new(max_entries: usize) -> Self {
        Self { inner: Cache::new(max_entries) }
    }

    pub fn get(&mut self, key: &GeometryKey) -> Option<&GeometryEntry> {
        self.inner.get(key)
    }

    pub fn insert(&mut self, key: GeometryKey, entry: GeometryEntry) {
        self.inner.insert(key, entry);
    }

    pub fn invalidate(&mut self, key: &GeometryKey) -> bool {
        self.inner.invalidate(key)
    }

    /// Remove all entries for a given series.
    pub fn invalidate_series(&mut self, series_id: u32) {
        self.inner.invalidate_where(|k| k.series_id == series_id);
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

    fn make_key(series_id: u32, vp: u64, data: u64) -> GeometryKey {
        GeometryKey { series_id, viewport_hash: vp, data_hash: data }
    }

    fn make_entry() -> GeometryEntry {
        GeometryEntry {
            vertices: vec![[0.0; 8]],
            indices: vec![0, 1, 2],
            created_at: Instant::now(),
        }
    }

    // Clasificación: determinística — verifica new_cache_is_empty
    #[test]
    fn new_cache_is_empty() {
        let cache = GeometryCache::new(16);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    // Clasificación: determinística — round-trip insert/get — invariante básico
    #[test]
    fn insert_and_get() {
        let mut cache = GeometryCache::new(16);
        let key = make_key(1, 100, 200);
        cache.insert(key.clone(), make_entry());
        assert!(cache.get(&key).is_some());
        assert_eq!(cache.len(), 1);
    }

    // Clasificación: determinística — edge case: key inexistente retorna None sin panic
    #[test]
    fn get_missing_returns_none() {
        let mut cache = GeometryCache::new(16);
        let key = make_key(1, 100, 200);
        assert!(cache.get(&key).is_none());
    }

    // Clasificación: determinística — verifica eliminación de entry y actualización de orden
    #[test]
    fn invalidate_removes_entry() {
        let mut cache = GeometryCache::new(16);
        let key = make_key(1, 100, 200);
        cache.insert(key.clone(), make_entry());
        assert!(cache.invalidate(&key));
        assert!(cache.get(&key).is_none());
        assert_eq!(cache.len(), 0);
    }

    // Clasificación: determinística — edge case: invalidar key inexistente
    #[test]
    fn invalidate_returns_false_for_missing() {
        let mut cache = GeometryCache::new(16);
        let key = make_key(1, 100, 200);
        assert!(!cache.invalidate(&key));
    }

    // Clasificación: determinística — verifica eliminación de entry y actualización de orden
    #[test]
    fn invalidate_series_removes_all_for_series() {
        let mut cache = GeometryCache::new(16);
        cache.insert(make_key(1, 1, 1), make_entry());
        cache.insert(make_key(1, 2, 2), make_entry());
        cache.insert(make_key(2, 1, 1), make_entry());
        cache.invalidate_series(1);
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&make_key(2, 1, 1)).is_some());
    }

    // Clasificación: determinística — verifica reset completo del estado
    #[test]
    fn clear_empties_cache_and_resets_stats() {
        let mut cache = GeometryCache::new(16);
        cache.insert(make_key(1, 1, 1), make_entry());
        let _ = cache.get(&make_key(99, 99, 99));
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    // Clasificación: determinística — verifica cálculo de métrica hit/miss
    #[test]
    fn hit_rate_tracks_correctly() {
        let mut cache = GeometryCache::new(16);
        let key = make_key(1, 100, 200);
        cache.insert(key.clone(), make_entry());
        let _ = cache.get(&key);
        let _ = cache.get(&make_key(2, 0, 0));
        let _ = cache.get(&key);
        assert!((cache.hit_rate() - 2.0 / 3.0).abs() < f64::EPSILON);
    }

    // Clasificación: determinística — verifica política de evicción al exceder capacidad
    #[test]
    fn eviction_removes_oldest_when_full() {
        let mut cache = GeometryCache::new(2);
        let k1 = make_key(1, 1, 1);
        let k2 = make_key(2, 2, 2);
        let k3 = make_key(3, 3, 3);
        cache.insert(k1.clone(), make_entry());
        cache.insert(k2.clone(), make_entry());
        cache.insert(k3.clone(), make_entry());
        assert_eq!(cache.len(), 2);
        assert!(cache.get(&k1).is_none());
        assert!(cache.get(&k2).is_some());
        assert!(cache.get(&k3).is_some());
    }
}
