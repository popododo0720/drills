//! A least-recently-used (LRU) cache.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::num::NonZeroUsize;

// An entry in the doubly linked list stored in `LruCache::nodes`.
#[derive(Debug)]
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}

/// A fixed-size cache that evicts the least recently used entry when it is full.
#[derive(Debug)]
pub struct LruCache<K, V> {
    capacity: NonZeroUsize,
    map: HashMap<K, usize>,
    nodes: Vec<Node<K, V>>,
    head: Option<usize>,
    tail: Option<usize>,
}

impl<K, V> LruCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Creates an empty cache that holds at most `capacity` entries.
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            capacity,
            map: HashMap::new(),
            nodes: Vec::new(),
            head: None,
            tail: None,
        }
    }

    /// Returns the value for `key`, or `None` if it is not in the cache.
    ///
    /// A found entry becomes the most recently used one.
    pub fn get<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let index = *self.map.get(key)?;
        self.detach(index);
        self.push_front(index);
        Some(self.nodes[index].value.clone())
    }

    /// Inserts `key` with `value`, or updates the value if `key` is already present.
    ///
    /// The entry becomes the most recently used one. If the cache is full and `key` is new,
    /// the least recently used entry is evicted first.
    pub fn put(&mut self, key: K, value: V) {
        // The key is already present: update the value and mark it as most recently used.
        if let Some(&index) = self.map.get(&key) {
            self.nodes[index].value = value;
            self.detach(index);
            self.push_front(index);
            return;
        }

        let index = if self.nodes.len() < self.capacity.get() {
            // There is room: append a new node.
            self.nodes.push(Node {
                key: key.clone(),
                value,
                prev: None,
                next: None,
            });
            self.nodes.len() - 1
        } else {
            // The cache is full: reuse the slot of the least recently used node.
            let index = self.tail.expect("a full cache always has a tail");
            self.detach(index);
            self.map.remove(&self.nodes[index].key);
            self.nodes[index].key = key.clone();
            self.nodes[index].value = value;
            index
        };

        self.map.insert(key, index);
        self.push_front(index);
    }

    // Unlinks the node at `index` from the list. The node stays in `nodes`.
    fn detach(&mut self, index: usize) {
        let prev = self.nodes[index].prev;
        let next = self.nodes[index].next;

        match prev {
            Some(p) => self.nodes[p].next = next,
            None => self.head = next,
        }

        match next {
            Some(n) => self.nodes[n].prev = prev,
            None => self.tail = prev,
        }

        self.nodes[index].prev = None;
        self.nodes[index].next = None;
    }

    // Links the node at `index` in as the new head (most recently used).
    //
    // The node must already be detached.
    fn push_front(&mut self, index: usize) {
        let old_head = self.head;

        self.nodes[index].prev = None;
        self.nodes[index].next = old_head;

        match old_head {
            Some(h) => self.nodes[h].prev = Some(index),
            None => self.tail = Some(index),
        }

        self.head = Some(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Keys are string literals and values are unused, so `i32` is enough.
    type TestCache = LruCache<&'static str, i32>;

    fn new_cache() -> TestCache {
        LruCache::new(NonZeroUsize::new(3).expect("3 is not zero"))
    }

    // Appends a node for `key` and links it in as the head. Returns its index.
    fn push(cache: &mut TestCache, key: &'static str) -> usize {
        cache.nodes.push(Node {
            key,
            value: 0,
            prev: None,
            next: None,
        });
        let index = cache.nodes.len() - 1;
        cache.push_front(index);
        index
    }

    // Collects keys by following `next` links from the head.
    fn keys_from_head(cache: &TestCache) -> Vec<&'static str> {
        let mut keys = Vec::new();
        let mut current = cache.head;
        while let Some(index) = current {
            keys.push(cache.nodes[index].key);
            current = cache.nodes[index].next;
        }
        keys
    }

    // Collects keys by following `prev` links from the tail.
    fn keys_from_tail(cache: &TestCache) -> Vec<&'static str> {
        let mut keys = Vec::new();
        let mut current = cache.tail;
        while let Some(index) = current {
            keys.push(cache.nodes[index].key);
            current = cache.nodes[index].prev;
        }
        keys
    }

    #[test]
    fn push_front_on_empty_sets_head_and_tail() {
        let mut cache = new_cache();

        let a = push(&mut cache, "a");

        assert_eq!(cache.head, Some(a));
        assert_eq!(cache.tail, Some(a));
    }

    #[test]
    fn push_front_adds_to_front() {
        let mut cache = new_cache();

        push(&mut cache, "c");
        push(&mut cache, "b");
        push(&mut cache, "a");

        assert_eq!(keys_from_head(&cache), ["a", "b", "c"]);
        assert_eq!(keys_from_tail(&cache), ["c", "b", "a"]);
    }

    #[test]
    fn detach_middle_links_neighbors() {
        let mut cache = new_cache();
        push(&mut cache, "c");
        let b = push(&mut cache, "b");
        push(&mut cache, "a");
        // List is now: a <-> b <-> c

        cache.detach(b);

        assert_eq!(keys_from_head(&cache), ["a", "c"]);
        assert_eq!(keys_from_tail(&cache), ["c", "a"]);
        assert_eq!(cache.nodes[b].prev, None);
        assert_eq!(cache.nodes[b].next, None);
    }

    #[test]
    fn detach_head_moves_head_to_next() {
        let mut cache = new_cache();
        push(&mut cache, "c");
        push(&mut cache, "b");
        let a = push(&mut cache, "a");
        // List is now: a <-> b <-> c

        cache.detach(a);

        assert_eq!(keys_from_head(&cache), ["b", "c"]);
        assert_eq!(keys_from_tail(&cache), ["c", "b"]);
    }

    #[test]
    fn detach_tail_moves_tail_to_prev() {
        let mut cache = new_cache();
        let c = push(&mut cache, "c");
        push(&mut cache, "b");
        push(&mut cache, "a");
        // List is now: a <-> b <-> c

        cache.detach(c);

        assert_eq!(keys_from_head(&cache), ["a", "b"]);
        assert_eq!(keys_from_tail(&cache), ["b", "a"]);
    }

    #[test]
    fn detach_only_node_empties_list() {
        let mut cache = new_cache();
        let a = push(&mut cache, "a");

        cache.detach(a);

        assert_eq!(cache.head, None);
        assert_eq!(cache.tail, None);
        assert_eq!(keys_from_head(&cache), Vec::<&str>::new());
    }
}
