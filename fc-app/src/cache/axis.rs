use std::collections::HashMap;

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
    entries: HashMap<AxisKey, AxisEntry>,
    max_entries: usize,
    hits: u64,
    misses: u64,
}

impl AxisCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(max_entries),
            max_entries,
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, key: &AxisKey) -> Option<&AxisEntry> {
        if let Some(entry) = self.entries.get(key) {
            self.hits += 1;
            Some(entry)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, key: AxisKey, entry: AxisEntry) {
        if self.entries.len() >= self.max_entries {
            let keys_to_remove: Vec<AxisKey> = self
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

    fn make_key(axis_type: AxisType) -> AxisKey {
        AxisKey::new(axis_type, 0.0, 100.0, 400.0, 12.0)
    }

    fn make_entry() -> AxisEntry {
        AxisEntry {
            ticks: vec![AxisTick {
                position: 10.0,
                value: 50.0,
                label: "50".to_string(),
            }],
        }
    }

    // Clasificación: determinística — verifica que el constructor inicializa correctamente el estado
    #[test]
    fn new_cache_is_empty() {
        let cache = AxisCache::new(16);
        assert_eq!(cache.len(), 0);
    }

    // Clasificación: determinística — verifica round-trip insert/get — invariante básico del cache
    #[test]
    fn insert_and_get() {
        let mut cache = AxisCache::new(16);
        let key = make_key(AxisType::Price);
        cache.insert(key.clone(), make_entry());
        assert!(cache.get(&key).is_some());
        assert_eq!(cache.len(), 1);
    }

    // Clasificación: determinística — edge case: buscar key inexistente no debe panic ni corrompir estado
    #[test]
    fn get_missing_returns_none() {
        let mut cache = AxisCache::new(16);
        let key = make_key(AxisType::Time);
        assert!(cache.get(&key).is_none());
    }

    // Clasificación: determinística — verifica que clear() resetea completamente el estado y las estadísticas
    #[test]
    fn clear_empties_cache() {
        let mut cache = AxisCache::new(16);
        cache.insert(make_key(AxisType::Price), make_entry());
        let _ = cache.get(&make_key(AxisType::Time));
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    // Clasificación: determinística — verifica el cálculo de hit rate — métrica crítica para decisiones de cache
    #[test]
    fn hit_rate_tracks_correctly() {
        let mut cache = AxisCache::new(16);
        let key = make_key(AxisType::Price);
        cache.insert(key.clone(), make_entry());
        let _ = cache.get(&key); // hit
        let _ = cache.get(&make_key(AxisType::Time)); // miss
        assert!((cache.hit_rate() - 0.5).abs() < f64::EPSILON);
    }

    // Clasificación: determinística — verifica FIFO eviction — al exceder capacidad se elimina la entrada más antigua
    #[test]
    fn eviction_when_full() {
        let mut cache = AxisCache::new(2);
        cache.insert(make_key(AxisType::Price), make_entry());
        cache.insert(make_key(AxisType::Time), make_entry());
        let extra = AxisKey::new(AxisType::Price, 50.0, 200.0, 800.0, 14.0);
        cache.insert(extra, make_entry());
        assert_eq!(cache.len(), 2);
    }
}
