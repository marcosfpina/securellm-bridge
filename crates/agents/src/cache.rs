//! LRU cache for tool results (performance optimization)

use crate::{ToolResult};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;

/// Thread-safe LRU cache for tool results
pub struct ResultCache {
    cache: Mutex<LruCache<String, ToolResult>>,
}

impl ResultCache {
    /// Create a new cache with capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(capacity).unwrap())),
        }
    }
    
    /// Get cached result (O(1) average)
    pub fn get(&self, key: &str) -> Option<ToolResult> {
        self.cache.lock().unwrap().get(key).cloned()
    }
    
    /// Put result in cache (O(1) average)
    pub fn put(&self, key: String, value: ToolResult) {
        self.cache.lock().unwrap().put(key, value);
    }
    
    /// Check if key exists (O(1))
    pub fn contains(&self, key: &str) -> bool {
        self.cache.lock().unwrap().contains(key)
    }
    
    /// Clear all cached results
    pub fn clear(&self) {
        self.cache.lock().unwrap().clear();
    }
    
    /// Get cache hit rate statistics
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.lock().unwrap();
        CacheStats {
            size: cache.len(),
            capacity: cache.cap().get(),
        }
    }
}

impl Default for ResultCache {
    fn default() -> Self {
        Self::new(256) // Default 256 entries
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.size as f64 / self.capacity as f64
        }
    }
}
