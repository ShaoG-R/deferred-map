use crate::DeferredMap;

#[test]
fn test_reserve() {
    let mut map: DeferredMap<i32> = DeferredMap::new();
    assert_eq!(map.capacity(), 0);

    map.reserve(100);
    assert!(map.capacity() >= 100);
}

#[test]
fn test_shrink_to_fit() {
    let mut map: DeferredMap<i32> = DeferredMap::with_capacity(100);
    assert!(map.capacity() >= 100);

    let handle = map.allocate_handle();
    map.insert(handle, 42);

    map.shrink_to_fit();
    // Capacity should ideally be close to 1 (plus overhead)
    // Note: Exact capacity depends on allocator and Vec implementation details
    assert!(map.capacity() < 100);
    assert!(map.capacity() >= 1);
}

#[test]
fn test_retain() {
    let mut map: DeferredMap<i32> = DeferredMap::new();
    let mut verify_map = std::collections::HashMap::new();

    for i in 0..10 {
        let handle = map.allocate_handle();
        let key = handle.key();
        map.insert(handle, i);
        verify_map.insert(key, i);
    }

    assert_eq!(map.len(), 10);

    // Keep only even numbers
    map.retain(|_, v| *v % 2 == 0);

    assert_eq!(map.len(), 5);

    // Verify content
    for (key, val) in verify_map {
        if val % 2 == 0 {
            assert_eq!(map.get(key), Some(&val));
        } else {
            assert_eq!(map.get(key), None);
        }
    }
}

#[test]
fn test_retain_reuse_slots() {
    let mut map: DeferredMap<i32> = DeferredMap::new();
    let mut initial_keys = Vec::new();

    for i in 0..5 {
        let handle = map.allocate_handle();
        initial_keys.push(handle.key());
        map.insert(handle, i);
    }

    // Remove everything
    map.retain(|_, _| false);
    assert_eq!(map.len(), 0);

    // Re-insert
    for i in 0..5 {
        let handle = map.allocate_handle();
        map.insert(handle, i + 100);
    }

    assert_eq!(map.len(), 5);
}
