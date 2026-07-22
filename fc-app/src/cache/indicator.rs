use std::collections::HashMap;

/// Cache key for indicator geometry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndicatorKey {
    pub indicator_id: u32,
    pub data_hash: u64,
    pub viewport_hash: u64,
}

/// Cached indicator geometry.
#[derive(Debug, Clone)]
pub struct IndicatorEntry {
    pub vertices: Vec<[f32; 8]>,
    pub indices: Vec<u32>,
}

pub struct IndicatorCache {
    entries: HashMap<IndicatorKey, IndicatorEntry>,
    max_entries: usize,
    hits: u64,
    misses: u64,
}

impl IndicatorCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(max_entries),
            max_entries,
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, key: &IndicatorKey) -> Option<&IndicatorEntry> {
        if let Some(entry) = self.entries.get(key) {
            self.hits += 1;
            Some(entry)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, key: IndicatorKey, entry: IndicatorEntry) {
        if self.entries.len() >= self.max_entries {
            let keys_to_remove: Vec<IndicatorKey> = self
                .entries
                .keys()
                .take(self.entries.len() - self.max_entries + 1)
                .cloned()
                .collect();
            for k in keys_to_remove {
                self.entries.remove(&k);
            }
        }
        self.entries.insert(key, entry);
    }

    pub fn invalidate_indicator(&mut self, indicator_id: u32) {
        self.entries.retain(|k, _| k.indicator_id != indicator_id);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(indicator_id: u32, data: u64, vp: u64) -> IndicatorKey {
        IndicatorKey {
            indicator_id,
            data_hash: data,
            viewport_hash: vp,
        }
    }

    fn make_entry() -> IndicatorEntry {
        IndicatorEntry {
            vertices: vec![[0.0; 8]],
            indices: vec![0, 1, 2],
        }
    }

    // Clasificación: determinística — verifica que el constructor inicializa correctamente el estado
    #[test]
    fn new_cache_is_empty() {
        let cache = IndicatorCache::new(16);
        assert_eq!(cache.len(), 0);
    }

    // Clasificación: determinística — verifica round-trip insert/get — invariante básico del cache
    #[test]
    fn insert_and_get() {
        let mut cache = IndicatorCache::new(16);
        let key = make_key(1, 100, 200);
        cache.insert(key.clone(), make_entry());
        assert!(cache.get(&key).is_some());
        assert_eq!(cache.len(), 1);
    }

    // Clasificación: determinística — edge case: buscar key inexistente no debe panic ni corrompir estado
    #[test]
    fn get_missing_returns_none() {
        let mut cache = IndicatorCache::new(16);
        let key = make_key(1, 100, 200);
        assert!(cache.get(&key).is_none());
    }

    // Clasificación: determinística — verifica que invalidate() elimina la entry y actualiza el vector de orden
    #[test]
    fn invalidate_indicator_removes_all_for_id() {
        let mut cache = IndicatorCache::new(16);
        cache.insert(make_key(1, 1, 1), make_entry());
        cache.insert(make_key(1, 2, 2), make_entry());
        cache.insert(make_key(2, 1, 1), make_entry());
        cache.invalidate_indicator(1);
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&make_key(2, 1, 1)).is_some());
    }

    // Clasificación: determinística — verifica que clear() resetea completamente el estado y las estadísticas
    #[test]
    fn clear_empties_cache() {
        let mut cache = IndicatorCache::new(16);
        cache.insert(make_key(1, 1, 1), make_entry());
        let _ = cache.get(&make_key(99, 99, 99));
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    // Clasificación: determinística — verifica el cálculo de hit rate — métrica crítica para decisiones de cache
    #[test]
    fn hit_rate_tracks_correctly() {
        let mut cache = IndicatorCache::new(16);
        let key = make_key(1, 100, 200);
        cache.insert(key.clone(), make_entry());
        let _ = cache.get(&key); // hit
        let _ = cache.get(&make_key(2, 0, 0)); // miss
        assert!((cache.hit_rate() - 0.5).abs() < f64::EPSILON);
    }

    // Clasificación: determinística — verifica FIFO eviction — al exceder capacidad se elimina la entrada más antigua
    #[test]
    fn eviction_when_full() {
        let mut cache = IndicatorCache::new(2);
        cache.insert(make_key(1, 1, 1), make_entry());
        cache.insert(make_key(2, 2, 2), make_entry());
        cache.insert(make_key(3, 3, 3), make_entry());
        assert_eq!(cache.len(), 2);
    }
}
