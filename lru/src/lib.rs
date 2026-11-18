//! A least-recently-used cache.

use std::collections::{HashMap, VecDeque};
use std::num::NonZeroUsize;

/// A fixed-size cache that evicts the least recently used entry when it is full.
pub struct LruCache {
    capacity: NonZeroUsize,
    map: HashMap<String, i32>,
    order: VecDeque<String>,
}

impl LruCache {
    /// Creates an empty cache that holds at most `capacity` entries.
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            capacity,
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    /// Returns the value for `key`, or `None` if it is not in the cache.
    ///
    /// A found entry becomes the most recently used one.
    pub fn get(&mut self, key: &str) -> Option<i32> {
        let value = self.map.get(key).copied()?;
        self.touch(key);

        Some(value)
    }

    /// Inserts `key` with `value`, or updates the value if `key` is already present.
    ///
    /// The entry becomes the most recently used one. If the cache is full and `key` is new,
    /// the least recently used entry is evicted first.
    pub fn put(&mut self, key: String, value: i32) {
        if self.map.contains_key(&key) {
            self.touch(&key);
        } else {
            self.evict_if_full();
            self.order.push_back(key.clone());
        }
        self.map.insert(key, value);
    }

    // Moves key to the most recently used position.
    fn touch(&mut self, key: &str) {
        if let Some(index) = self.order.iter().position(|k| k == key)
            && let Some(k) = self.order.remove(index)
        {
            self.order.push_back(k);
        }
    }

    // Evicts the least recently used entry if the cache is full.
    fn evict_if_full(&mut self) {
        if self.map.len() >= self.capacity.get()
            && let Some(oldest) = self.order.pop_front()
        {
            self.map.remove(&oldest);
        }
    }
}
