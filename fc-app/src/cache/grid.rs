use std::collections::HashMap;

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
    entries: HashMap<GridKey, GridEntry>,
    max_entries: usize,
    hits: u64,
    misses: u64,
}

impl GridCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(max_entries),
            max_entries,
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, key: &GridKey) -> Option<&GridEntry> {
        if let Some(entry) = self.entries.get(key) {
            self.hits += 1;
            Some(entry)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, key: GridKey, entry: GridEntry) {
        if self.entries.len() >= self.max_entries {
            let keys_to_remove: Vec<GridKey> = self
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

    fn make_key(w: u32, h: u32) -> GridKey {
        GridKey::new(w, h, (0.0, 100.0), (0.0, 1000.0))
    }

    fn make_entry() -> GridEntry {
        GridEntry {
            horizontal_lines: vec![10.0, 50.0, 90.0],
            vertical_lines: vec![200.0, 400.0, 600.0],
        }
    }

    // Clasificación: determinística — verifica que el constructor inicializa correctamente el estado
    #[test]
    fn new_cache_is_empty() {
        let cache = GridCache::new(16);
        assert_eq!(cache.len(), 0);
    }

    // Clasificación: determinística — verifica round-trip insert/get — invariante básico del cache
    #[test]
    fn insert_and_get() {
        let mut cache = GridCache::new(16);
        let key = make_key(800, 600);
        cache.insert(key.clone(), make_entry());
        assert!(cache.get(&key).is_some());
        assert_eq!(cache.len(), 1);
    }

    // Clasificación: determinística — edge case: buscar key inexistente no debe panic ni corrompir estado
    #[test]
    fn get_missing_returns_none() {
        let mut cache = GridCache::new(16);
        let key = make_key(800, 600);
        assert!(cache.get(&key).is_none());
    }

    // Clasificación: determinística — verifica que clear() resetea completamente el estado y las estadísticas
    #[test]
    fn clear_empties_cache() {
        let mut cache = GridCache::new(16);
        cache.insert(make_key(800, 600), make_entry());
        let _ = cache.get(&make_key(1024, 768));
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    // Clasificación: determinística — verifica el cálculo de hit rate — métrica crítica para decisiones de cache
    #[test]
    fn hit_rate_tracks_correctly() {
        let mut cache = GridCache::new(16);
        let key = make_key(800, 600);
        cache.insert(key.clone(), make_entry());
        let _ = cache.get(&key); // hit
        let _ = cache.get(&make_key(1024, 768)); // miss
        assert!((cache.hit_rate() - 0.5).abs() < f64::EPSILON);
    }

    // Clasificación: determinística — verifica FIFO eviction — al exceder capacidad se elimina la entrada más antigua
    #[test]
    fn eviction_when_full() {
        let mut cache = GridCache::new(2);
        cache.insert(make_key(100, 100), make_entry());
        cache.insert(make_key(200, 200), make_entry());
        cache.insert(make_key(300, 300), make_entry());
        assert_eq!(cache.len(), 2);
    }
}
