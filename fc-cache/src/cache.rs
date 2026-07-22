//! Generic bounded cache with hit-rate tracking.
//!
//! All domain-specific caches (Geometry, Text, Axis, Grid, Indicator)
//! delegate to this single implementation.

use std::collections::HashMap;
use std::hash::Hash;

/// Bounded cache with FIFO eviction and hit-rate tracking.
///
/// When the cache reaches `max_entries`, the oldest entry (by insertion
/// order) is evicted on the next `insert`.
pub struct Cache<K, V> {
    entries: HashMap<K, V>,
    /// Insertion order for FIFO eviction.
    order: Vec<K>,
    max_entries: usize,
    hits: u64,
    misses: u64,
}

impl<K: Clone + Eq + Hash, V> Cache<K, V> {
    /// Create a new cache with the given capacity.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(max_entries),
            order: Vec::with_capacity(max_entries),
            max_entries,
            hits: 0,
            misses: 0,
        }
    }

    /// Look up an entry. Tracks hit/miss statistics.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(entry) = self.entries.get(key) {
            self.hits += 1;
            Some(entry)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Insert an entry. Evicts the oldest entry if at capacity.
    pub fn insert(&mut self, key: K, value: V) {
        if self.entries.len() >= self.max_entries {
            // Evict oldest by insertion order
            if let Some(oldest) = self.order.first().cloned() {
                self.entries.remove(&oldest);
                self.order.remove(0);
            }
        }
        self.entries.insert(key.clone(), value);
        self.order.push(key);
    }

    /// Remove a specific entry. Returns true if it existed.
    pub fn invalidate(&mut self, key: &K) -> bool {
        if self.entries.remove(key).is_some() {
            self.order.retain(|k| k != key);
            true
        } else {
            false
        }
    }

    /// Remove all entries matching a predicate.
    pub fn invalidate_where(&mut self, mut pred: impl FnMut(&K) -> bool) {
        self.order.retain(|k| {
            if pred(k) {
                self.entries.remove(k);
                false
            } else {
                true
            }
        });
    }

    /// Clear all entries and reset statistics.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// Cache hit rate (0.0–1.0). Returns 0.0 if no lookups have been made.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 }
    }

    /// Number of entries currently cached.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Clasificación: determinística — verifica new_cache_is_empty
    #[test]
    fn new_cache_is_empty() {
        let cache: Cache<u32, String> = Cache::new(4);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    // Clasificación: determinística — round-trip insert/get — invariante básico
    #[test]
    fn insert_and_get() {
        let mut cache = Cache::new(4);
        cache.insert(1, "a".to_string());
        assert_eq!(cache.get(&1), Some(&"a".to_string()));
        assert_eq!(cache.len(), 1);
    }

    // Clasificación: determinística — edge case: key inexistente retorna None sin panic
    #[test]
    fn get_missing_returns_none() {
        let mut cache: Cache<u32, String> = Cache::new(4);
        assert_eq!(cache.get(&99), None);
    }

    // Clasificación: determinística — verifica cálculo de métrica hit/miss
    #[test]
    fn hit_rate_tracks_correctly() {
        let mut cache = Cache::new(4);
        cache.insert(1, "a".to_string());
        let _ = cache.get(&1); // hit
        let _ = cache.get(&2); // miss
        let _ = cache.get(&1); // hit
        assert!((cache.hit_rate() - 2.0 / 3.0).abs() < f64::EPSILON);
    }

    // Clasificación: determinística — verifica política de evicción al exceder capacidad
    #[test]
    fn eviction_removes_oldest() {
        let mut cache = Cache::new(2);
        cache.insert(1, "a".to_string());
        cache.insert(2, "b".to_string());
        cache.insert(3, "c".to_string());
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&1), None); // evicted
        assert_eq!(cache.get(&2), Some(&"b".to_string()));
        assert_eq!(cache.get(&3), Some(&"c".to_string()));
    }

    // Clasificación: determinística — verifica eliminación de entry y actualización de orden
    #[test]
    fn invalidate_removes_entry() {
        let mut cache = Cache::new(4);
        cache.insert(1, "a".to_string());
        assert!(cache.invalidate(&1));
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.len(), 0);
    }

    // Clasificación: determinística — edge case: invalidar key inexistente
    #[test]
    fn invalidate_returns_false_for_missing() {
        let mut cache: Cache<u32, String> = Cache::new(4);
        assert!(!cache.invalidate(&99));
    }

    // Clasificación: determinística — verifica invalidación condicional por predicado
    #[test]
    fn invalidate_where_filters_correctly() {
        let mut cache = Cache::new(8);
        cache.insert(1, "a".to_string());
        cache.insert(2, "b".to_string());
        cache.insert(3, "c".to_string());
        cache.invalidate_where(|k| *k == 2);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&1), Some(&"a".to_string()));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some(&"c".to_string()));
    }

    // Clasificación: determinística — verifica reset completo del estado
    #[test]
    fn clear_empties_cache_and_resets_stats() {
        let mut cache = Cache::new(4);
        cache.insert(1, "a".to_string());
        let _ = cache.get(&99);
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
    }
}
