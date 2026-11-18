//! Tests for the public API of `LruCache`.

use std::num::NonZeroUsize;

use lru::LruCache;

#[test]
fn get_returns_value_after_put() {
    let capacity = NonZeroUsize::new(2).expect("2 is not zero");
    let mut cache = LruCache::new(capacity);

    cache.put("web1".to_string(), 200);

    assert_eq!(cache.get("web1"), Some(200));
    assert_eq!(cache.get("db1"), None);
}

#[test]
fn put_evicts_least_recently_used_when_full() {
    let capacity = NonZeroUsize::new(2).expect("2 is not zero");
    let mut cache = LruCache::new(capacity);

    cache.put("web1".to_string(), 200);
    cache.put("db1".to_string(), 400);
    cache.put("cache1".to_string(), 600);

    assert_eq!(cache.get("web1"), None);
    assert_eq!(cache.get("db1"), Some(400));
    assert_eq!(cache.get("cache1"), Some(600));
}

#[test]
fn get_marks_entry_as_recently_used() {
    let capacity = NonZeroUsize::new(2).expect("2 is not zero");
    let mut cache = LruCache::new(capacity);

    cache.put("web1".to_string(), 200);
    cache.put("db1".to_string(), 400);

    assert_eq!(cache.get("web1"), Some(200));

    cache.put("cache1".to_string(), 600);

    assert_eq!(cache.get("web1"), Some(200));
    assert_eq!(cache.get("db1"), None);
    assert_eq!(cache.get("cache1"), Some(600));
}

#[test]
fn put_existing_key_does_not_evict_other_entries() {
    let capacity = NonZeroUsize::new(2).expect("2 is not zero");
    let mut cache = LruCache::new(capacity);

    cache.put("web1".to_string(), 200);
    cache.put("db1".to_string(), 400);
    cache.put("web1".to_string(), 600);

    assert_eq!(cache.get("web1"), Some(600));
    assert_eq!(cache.get("db1"), Some(400));
}
