use crate::cache::Cache;

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
    inner: Cache<IndicatorKey, IndicatorEntry>,
}

impl IndicatorCache {
    pub fn new(max_entries: usize) -> Self {
        Self { inner: Cache::new(max_entries) }
    }

    pub fn get(&mut self, key: &IndicatorKey) -> Option<&IndicatorEntry> {
        self.inner.get(key)
    }

    pub fn insert(&mut self, key: IndicatorKey, entry: IndicatorEntry) {
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

    fn make_key(id: u32) -> IndicatorKey {
        IndicatorKey { indicator_id: id, data_hash: 0, viewport_hash: 0 }
    }

    fn make_entry() -> IndicatorEntry {
        IndicatorEntry { vertices: vec![[0.0; 8]], indices: vec![0, 1, 2] }
    }

    #[test]
    fn new_cache_is_empty() {
        let cache = IndicatorCache::new(16);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn insert_and_get() {
        let mut cache = IndicatorCache::new(16);
        let key = make_key(1);
        cache.insert(key.clone(), make_entry());
        assert!(cache.get(&key).is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn get_missing_returns_none() {
        let mut cache = IndicatorCache::new(16);
        let key = make_key(99);
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn clear_empties_cache() {
        let mut cache = IndicatorCache::new(16);
        cache.insert(make_key(1), make_entry());
        let _ = cache.get(&make_key(2));
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    #[test]
    fn hit_rate_tracks_correctly() {
        let mut cache = IndicatorCache::new(16);
        let key = make_key(1);
        cache.insert(key.clone(), make_entry());
        let _ = cache.get(&key);
        let _ = cache.get(&make_key(2));
        assert!((cache.hit_rate() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn eviction_when_full() {
        let mut cache = IndicatorCache::new(2);
        cache.insert(make_key(1), make_entry());
        cache.insert(make_key(2), make_entry());
        cache.insert(make_key(3), make_entry());
        assert_eq!(cache.len(), 2);
    }
}
