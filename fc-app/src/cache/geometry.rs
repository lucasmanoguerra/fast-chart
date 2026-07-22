use std::collections::HashMap;
use std::time::Instant;

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
    entries: HashMap<GeometryKey, GeometryEntry>,
    max_entries: usize,
    hits: u64,
    misses: u64,
}

impl GeometryCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(max_entries),
            max_entries,
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, key: &GeometryKey) -> Option<&GeometryEntry> {
        if let Some(entry) = self.entries.get(key) {
            self.hits += 1;
            Some(entry)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, key: GeometryKey, entry: GeometryEntry) {
        if self.entries.len() >= self.max_entries {
            if let Some(oldest_key) = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.created_at)
                .map(|(k, _)| k.clone())
            {
                self.entries.remove(&oldest_key);
            }
        }
        self.entries.insert(key, entry);
    }

    pub fn invalidate(&mut self, key: &GeometryKey) -> bool {
        self.entries.remove(key).is_some()
    }

    pub fn invalidate_series(&mut self, series_id: u32) {
        self.entries.retain(|k, _| k.series_id != series_id);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.hits = 0;
        self.misses = 0;
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(series_id: u32, vp: u64, data: u64) -> GeometryKey {
        GeometryKey {
            series_id,
            viewport_hash: vp,
            data_hash: data,
        }
    }

    fn make_entry() -> GeometryEntry {
        GeometryEntry {
            vertices: vec![[0.0; 8]],
            indices: vec![0, 1, 2],
            created_at: Instant::now(),
        }
    }

    // Clasificación: determinística — verifica que el constructor inicializa correctamente el estado
    #[test]
    fn new_cache_is_empty() {
        let cache = GeometryCache::new(16);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    // Clasificación: determinística — verifica round-trip insert/get — invariante básico del cache
    #[test]
    fn insert_and_get() {
        let mut cache = GeometryCache::new(16);
        let key = make_key(1, 100, 200);
        let entry = make_entry();
        cache.insert(key.clone(), entry);
        assert!(cache.get(&key).is_some());
        assert_eq!(cache.len(), 1);
    }

    // Clasificación: determinística — edge case: buscar key inexistente no debe panic ni corrompir estado
    #[test]
    fn get_missing_returns_none() {
        let mut cache = GeometryCache::new(16);
        let key = make_key(1, 100, 200);
        assert!(cache.get(&key).is_none());
    }

    // Clasificación: determinística — verifica que invalidate() elimina la entry y actualiza el vector de orden
    #[test]
    fn invalidate_removes_entry() {
        let mut cache = GeometryCache::new(16);
        let key = make_key(1, 100, 200);
        cache.insert(key.clone(), make_entry());
        assert!(cache.invalidate(&key));
        assert!(cache.get(&key).is_none());
        assert_eq!(cache.len(), 0);
    }

    // Clasificación: determinística — edge case: invalidar key inexistente retorna false sin corrompir estado
    #[test]
    fn invalidate_returns_false_for_missing() {
        let mut cache = GeometryCache::new(16);
        let key = make_key(1, 100, 200);
        assert!(!cache.invalidate(&key));
    }

    // Clasificación: determinística — verifica invalidación selectiva por serie — preserva entries de otras series
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

    // Clasificación: determinística — verifica que clear() resetea completamente el estado y las estadísticas
    #[test]
    fn clear_empties_cache_and_resets_stats() {
        let mut cache = GeometryCache::new(16);
        cache.insert(make_key(1, 1, 1), make_entry());
        let _ = cache.get(&make_key(99, 99, 99));
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    // Clasificación: determinística — verifica el cálculo de hit rate — métrica crítica para decisiones de cache
    #[test]
    fn hit_rate_tracks_correctly() {
        let mut cache = GeometryCache::new(16);
        let key = make_key(1, 100, 200);
        cache.insert(key.clone(), make_entry());
        let _ = cache.get(&key); // hit
        let _ = cache.get(&make_key(2, 0, 0)); // miss
        let _ = cache.get(&key); // hit
        assert!((cache.hit_rate() - 2.0 / 3.0).abs() < f64::EPSILON);
    }

    // Clasificación: determinística — verifica FIFO eviction — al exceder capacidad se elimina la entrada más antigua
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
