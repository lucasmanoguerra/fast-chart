use crate::cache::Cache;

/// Cache key for formatted text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextKey {
    pub text: String,
    pub font_size_bits: u32,
    pub color: [u8; 4],
}

impl TextKey {
    pub fn new(text: String, font_size: f32, color: [u8; 4]) -> Self {
        Self { text, font_size_bits: font_size.to_bits(), color }
    }

    pub fn font_size(&self) -> f32 {
        f32::from_bits(self.font_size_bits)
    }
}

/// Cached text layout.
#[derive(Debug, Clone)]
pub struct TextEntry {
    pub width: f32,
    pub height: f32,
    pub glyph_ids: Vec<u32>,
}

pub struct TextCache {
    inner: Cache<TextKey, TextEntry>,
}

impl TextCache {
    pub fn new(max_entries: usize) -> Self {
        Self { inner: Cache::new(max_entries) }
    }

    pub fn get(&mut self, key: &TextKey) -> Option<&TextEntry> {
        self.inner.get(key)
    }

    pub fn insert(&mut self, key: TextKey, entry: TextEntry) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(text: &str) -> TextKey {
        TextKey::new(text.to_string(), 14.0, [255, 255, 255, 255])
    }

    fn make_entry() -> TextEntry {
        TextEntry { width: 50.0, height: 16.0, glyph_ids: vec![1, 2, 3] }
    }

    #[test]
    fn new_cache_is_empty() {
        let cache = TextCache::new(16);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn insert_and_get() {
        let mut cache = TextCache::new(16);
        let key = make_key("hello");
        cache.insert(key.clone(), make_entry());
        assert!(cache.get(&key).is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn get_missing_returns_none() {
        let mut cache = TextCache::new(16);
        let key = make_key("missing");
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn clear_empties_cache() {
        let mut cache = TextCache::new(16);
        cache.insert(make_key("a"), make_entry());
        let _ = cache.get(&make_key("z"));
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    #[test]
    fn hit_rate_tracks_correctly() {
        let mut cache = TextCache::new(16);
        let key = make_key("text");
        cache.insert(key.clone(), make_entry());
        let _ = cache.get(&key);
        let _ = cache.get(&make_key("other"));
        assert!((cache.hit_rate() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn eviction_when_full() {
        let mut cache = TextCache::new(2);
        cache.insert(make_key("a"), make_entry());
        cache.insert(make_key("b"), make_entry());
        cache.insert(make_key("c"), make_entry());
        assert_eq!(cache.len(), 2);
    }
}
