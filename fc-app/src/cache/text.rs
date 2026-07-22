use std::collections::HashMap;

/// Cache key for formatted text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextKey {
    pub text: String,
    pub font_size_bits: u32,
    pub color: [u8; 4],
}

impl TextKey {
    pub fn new(text: String, font_size: f32, color: [u8; 4]) -> Self {
        Self {
            text,
            font_size_bits: font_size.to_bits(),
            color,
        }
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
    entries: HashMap<TextKey, TextEntry>,
    max_entries: usize,
    hits: u64,
    misses: u64,
}

impl TextCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(max_entries),
            max_entries,
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, key: &TextKey) -> Option<&TextEntry> {
        if let Some(entry) = self.entries.get(key) {
            self.hits += 1;
            Some(entry)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, key: TextKey, entry: TextEntry) {
        if self.entries.len() >= self.max_entries {
            let keys_to_remove: Vec<TextKey> = self
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

    fn make_key(text: &str) -> TextKey {
        TextKey::new(text.to_string(), 14.0, [255, 255, 255, 255])
    }

    fn make_entry() -> TextEntry {
        TextEntry {
            width: 50.0,
            height: 16.0,
            glyph_ids: vec![1, 2, 3],
        }
    }

    // Clasificación: determinística — verifica que el constructor inicializa correctamente el estado
    #[test]
    fn new_cache_is_empty() {
        let cache = TextCache::new(16);
        assert_eq!(cache.len(), 0);
    }

    // Clasificación: determinística — verifica round-trip insert/get — invariante básico del cache
    #[test]
    fn insert_and_get() {
        let mut cache = TextCache::new(16);
        let key = make_key("hello");
        cache.insert(key.clone(), make_entry());
        assert!(cache.get(&key).is_some());
        assert_eq!(cache.len(), 1);
    }

    // Clasificación: determinística — edge case: buscar key inexistente no debe panic ni corrompir estado
    #[test]
    fn get_missing_returns_none() {
        let mut cache = TextCache::new(16);
        let key = make_key("missing");
        assert!(cache.get(&key).is_none());
    }

    // Clasificación: determinística — verifica que clear() resetea completamente el estado y las estadísticas
    #[test]
    fn clear_empties_cache() {
        let mut cache = TextCache::new(16);
        cache.insert(make_key("a"), make_entry());
        let _ = cache.get(&make_key("z"));
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    // Clasificación: determinística — verifica el cálculo de hit rate — métrica crítica para decisiones de cache
    #[test]
    fn hit_rate_tracks_correctly() {
        let mut cache = TextCache::new(16);
        let key = make_key("text");
        cache.insert(key.clone(), make_entry());
        let _ = cache.get(&key); // hit
        let _ = cache.get(&make_key("other")); // miss
        assert!((cache.hit_rate() - 0.5).abs() < f64::EPSILON);
    }

    // Clasificación: determinística — verifica FIFO eviction — al exceder capacidad se elimina la entrada más antigua
    #[test]
    fn eviction_when_full() {
        let mut cache = TextCache::new(2);
        cache.insert(make_key("a"), make_entry());
        cache.insert(make_key("b"), make_entry());
        cache.insert(make_key("c"), make_entry());
        assert_eq!(cache.len(), 2);
    }
}
