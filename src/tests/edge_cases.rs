// Edge cases and error handling comprehensive tests
// 边界情况和错误处理的全面测试

use crate::DeferredMap;

#[test]
fn test_get_with_invalid_key() {
    let mut map = DeferredMap::new();

    let h = map.allocate_handle();
    let _ = h.key();
    map.insert(h, 42);

    // Try to get with valid key from un-inserted handle
    // 尝试使用未插入 handle 的有效 key 获取
    let h2 = map.allocate_handle();
    let k2 = h2.key();
    assert_eq!(map.get(k2), None);
}

#[test]
fn test_get_mut_with_invalid_key() {
    let mut map = DeferredMap::new();

    let h = map.allocate_handle();
    let _ = h.key();
    map.insert(h, 42);

    // Try to get_mut with valid key from un-inserted handle
    // 尝试使用未插入 handle 的有效 key 进行 get_mut
    let h2 = map.allocate_handle();
    let k2 = h2.key();
    assert_eq!(map.get_mut(k2), None);
}

#[test]
fn test_contains_key_with_invalid_key() {
    let mut map = DeferredMap::new();

    let h = map.allocate_handle();
    let k = h.key();
    map.insert(h, 42);

    assert!(map.contains_key(k));

    let h2 = map.allocate_handle();
    let k2 = h2.key();
    assert!(!map.contains_key(k2));
}

#[test]
fn test_operations_after_clear() {
    let mut map = DeferredMap::new();

    // Insert some elements
    // 插入一些元素
    let mut keys = Vec::new();
    for i in 0..10 {
        let h = map.allocate_handle();
        let k = h.key();
        map.insert(h, i);
        keys.push(k);
    }

    map.clear();

    // All previous keys should be invalid
    // 所有先前的 key 应该无效
    for key in keys {
        assert_eq!(map.get(key), None);
        assert_eq!(map.remove(key), None);
        assert!(!map.contains_key(key));
    }

    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
}

#[test]
fn test_iter_on_empty_map() {
    let map: DeferredMap<i32> = DeferredMap::new();

    let count = map.iter().count();
    assert_eq!(count, 0);
}

#[test]
fn test_iter_mut_on_empty_map() {
    let mut map: DeferredMap<i32> = DeferredMap::new();

    let count = map.iter_mut().count();
    assert_eq!(count, 0);
}

#[test]
fn test_iter_skips_removed_elements() {
    let mut map = DeferredMap::new();

    let mut keys = Vec::new();
    for i in 0..10 {
        let h = map.allocate_handle();
        let k = h.key();
        map.insert(h, i);
        keys.push(k);
    }

    // Remove some elements
    // 删除一些元素
    map.remove(keys[2]);
    map.remove(keys[5]);
    map.remove(keys[8]);

    let count = map.iter().count();
    assert_eq!(count, 7); // 10 - 3 = 7
}

#[test]
fn test_iter_mut_skips_removed_elements() {
    let mut map = DeferredMap::new();

    let mut keys = Vec::new();
    for i in 0..10 {
        let h = map.allocate_handle();
        let k = h.key();
        map.insert(h, i);
        keys.push(k);
    }

    map.remove(keys[0]);
    map.remove(keys[9]);

    let count = map.iter_mut().count();
    assert_eq!(count, 8);
}

#[test]
fn test_clone_empty_map() {
    let map: DeferredMap<i32> = DeferredMap::new();
    let cloned = map.clone();

    assert_eq!(cloned.len(), 0);
    assert_eq!(cloned.capacity(), 0);
    assert!(cloned.is_empty());
}

#[test]
fn test_clone_with_values() {
    let mut map = DeferredMap::new();

    let h1 = map.allocate_handle();
    let _k1 = h1.key();
    map.insert(h1, 42);

    let h2 = map.allocate_handle();
    let _k2 = h2.key();
    map.insert(h2, 100);

    let cloned = map.clone();
    assert_eq!(cloned.len(), 2);

    // Verify values exist by iterating
    let values: Vec<_> = cloned.iter().map(|(_, v)| *v).collect();
    assert!(values.contains(&42));
    assert!(values.contains(&100));
}

#[test]
fn test_clone_independence() {
    let mut map = DeferredMap::new();

    let h = map.allocate_handle();
    let k = h.key();
    map.insert(h, 42);

    let mut cloned = map.clone();

    // Find the key corresponding to 42 in the cloned map
    let (k_cloned, _) = cloned
        .iter()
        .find(|(_, v)| **v == 42)
        .expect("Value 42 not found in cloned map");

    // Modify cloned map using its own key
    if let Some(value) = cloned.get_mut(k_cloned) {
        *value = 100;
    }

    // Original should be unchanged
    // 原始的应该不变
    assert_eq!(map.get(k), Some(&42));
    // Verify modification in cloned map
    assert_eq!(cloned.get(k_cloned), Some(&100));
}

#[test]
fn test_clone_from() {
    let mut map1 = DeferredMap::new();
    let h1 = map1.allocate_handle();
    let _k1 = h1.key();
    map1.insert(h1, 1);

    let mut map2 = DeferredMap::new();
    let h2 = map2.allocate_handle();
    map2.insert(h2, 2);

    map2.clone_from(&map1);

    assert_eq!(map2.len(), 1);
    // Verify content via iteration
    let (k2, v2) = map2.iter().next().expect("Map2 should not be empty");
    assert_eq!(v2, &1);
    // Verification that k2 is usable on map2
    assert_eq!(map2.get(k2), Some(&1));
}

#[test]
fn test_default_creates_empty_map() {
    let map: DeferredMap<i32> = DeferredMap::default();

    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
}

#[test]
fn test_with_capacity_zero() {
    let map: DeferredMap<i32> = DeferredMap::with_capacity(0);

    assert_eq!(map.capacity(), 0);
    assert!(map.is_empty());
}

#[test]
fn test_with_capacity_large() {
    let map: DeferredMap<i32> = DeferredMap::with_capacity(1000);

    // Capacity should be at least what was requested
    // 容量应至少为请求的大小
    assert!(map.capacity() >= 1000);
}

#[test]
fn test_operations_on_map_with_gaps() {
    let mut map = DeferredMap::new();

    // Create map with gaps (insert, remove, insert pattern)
    // 创建有间隙的 map（插入、删除、插入模式）
    let h1 = map.allocate_handle();
    let k1 = h1.key();
    map.insert(h1, 1);

    let h2 = map.allocate_handle();
    let k2 = h2.key();
    map.insert(h2, 2);

    let h3 = map.allocate_handle();
    let k3 = h3.key();
    map.insert(h3, 3);

    // Remove middle element
    // 删除中间元素
    map.remove(k2);

    // Insert new element (should reuse k2's slot)
    // 插入新元素（应该复用 k2 的 slot）
    let h4 = map.allocate_handle();
    let k4 = h4.key();
    map.insert(h4, 4);

    // Verify all operations work correctly
    // 验证所有操作正常工作
    assert_eq!(map.get(k1), Some(&1));
    assert_eq!(map.get(k2), None); // Outdated key | 过时的 key
    assert_eq!(map.get(k3), Some(&3));
    assert_eq!(map.get(k4), Some(&4));

    assert_eq!(map.len(), 3);
}

#[test]
fn test_get_after_modification() {
    let mut map = DeferredMap::new();

    let h = map.allocate_handle();
    let k = h.key();
    map.insert(h, 42);

    // Modify through get_mut
    // 通过 get_mut 修改
    if let Some(value) = map.get_mut(k) {
        *value = 100;
    }

    // Verify change through get
    // 通过 get 验证更改
    assert_eq!(map.get(k), Some(&100));
}

#[test]
fn test_remove_during_iteration() {
    let mut map = DeferredMap::new();

    let mut keys = Vec::new();
    for i in 0..10 {
        let h = map.allocate_handle();
        let k = h.key();
        map.insert(h, i);
        keys.push(k);
    }

    // Collect keys from iteration
    // 从迭代中收集 key
    let iter_keys: Vec<_> = map.iter().map(|(k, _)| k).collect();

    // Remove some elements
    // 删除一些元素
    for &k in &iter_keys[0..5] {
        map.remove(k);
    }

    // New iteration should show reduced count
    // 新迭代应该显示减少的计数
    assert_eq!(map.iter().count(), 5);
}

#[test]
fn test_very_large_key_values() {
    let mut map: DeferredMap<i32> = DeferredMap::new();

    // Test with valid key format but not inserted
    let h = map.allocate_handle();
    let k = h.key();
    assert_eq!(map.get(k), None);
}

#[test]
fn test_consecutive_handle_allocations() {
    let mut map = DeferredMap::<i32>::new();

    let handles: Vec<_> = (0..100).map(|_| map.allocate_handle()).collect();

    // Verify all handles have unique indices
    // 验证所有 handle 都有唯一的索引
    let mut indices = std::collections::HashSet::new();
    for handle in &handles {
        let index = handle.index();
        assert!(indices.insert(index), "Duplicate index found");
    }
}

#[test]
fn test_interleaved_operations() {
    let mut map = DeferredMap::new();

    // Allocate
    let h1 = map.allocate_handle();

    // Insert
    let k1 = h1.key();
    map.insert(h1, 1);

    // Allocate more
    let h2 = map.allocate_handle();

    // Get
    assert_eq!(map.get(k1), Some(&1));

    // Insert
    let k2 = h2.key();
    map.insert(h2, 2);

    // Remove
    map.remove(k1);

    // Allocate (should reuse k1's slot)
    let h3 = map.allocate_handle();

    // Insert
    let k3 = h3.key();
    map.insert(h3, 3);

    // Verify state
    assert_eq!(map.get(k2), Some(&2));
    assert_eq!(map.get(k3), Some(&3));
    assert_eq!(map.len(), 2);
}

#[test]
fn test_map_with_option_type() {
    let mut map = DeferredMap::new();

    let h1 = map.allocate_handle();
    let k1 = h1.key();
    map.insert(h1, Some(42));

    let h2 = map.allocate_handle();
    let k2 = h2.key();
    map.insert(h2, None::<i32>);

    assert_eq!(map.get(k1), Some(&Some(42)));
    assert_eq!(map.get(k2), Some(&None));
}

#[test]
fn test_map_with_result_type() {
    let mut map = DeferredMap::new();

    let h1 = map.allocate_handle();
    let k1 = h1.key();
    map.insert(h1, Ok::<i32, String>(42));

    let h2 = map.allocate_handle();
    let k2 = h2.key();
    map.insert(h2, Err::<i32, String>("error".to_string()));

    assert_eq!(map.get(k1), Some(&Ok(42)));
    assert_eq!(map.get(k2), Some(&Err("error".to_string())));
}
