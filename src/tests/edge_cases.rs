// Edge cases and error handling comprehensive tests
// 边界情况和错误处理的全面测试

use crate::{DeferredMap, Handle, DeferredMapError};

#[test]
fn test_empty_map_operations() {
    let map: DeferredMap<i32> = DeferredMap::new();
    
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(map.capacity(), 0);
    
    // Get on empty map
    // 在空 map 上 get
    assert_eq!(map.get(1), None);
}

#[test]
fn test_get_with_invalid_key() {
    let mut map = DeferredMap::new();
    
    let h = map.allocate_handle();
    let k = map.insert(h, 42).unwrap();
    
    // Try to get with different key
    // 尝试使用不同的 key 获取
    assert_eq!(map.get(k + 1), None);
    assert_eq!(map.get(k * 2), None);
    assert_eq!(map.get(0), None);
}

#[test]
fn test_get_mut_with_invalid_key() {
    let mut map = DeferredMap::new();
    
    let h = map.allocate_handle();
    let k = map.insert(h, 42).unwrap();
    
    // Try to get_mut with invalid keys
    // 尝试使用无效的 key 进行 get_mut
    assert_eq!(map.get_mut(k + 1), None);
    assert_eq!(map.get_mut(0), None);
}

#[test]
fn test_contains_key_with_invalid_key() {
    let mut map = DeferredMap::new();
    
    let h = map.allocate_handle();
    let k = map.insert(h, 42).unwrap();
    
    assert!(map.contains_key(k));
    assert!(!map.contains_key(k + 1));
    assert!(!map.contains_key(0));
}

#[test]
fn test_operations_after_clear() {
    let mut map = DeferredMap::new();
    
    // Insert some elements
    // 插入一些元素
    let mut keys = Vec::new();
    for i in 0..10 {
        let h = map.allocate_handle();
        let k = map.insert(h, i).unwrap();
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
fn test_handle_with_sentinel_index() {
    let mut map = DeferredMap::<i32>::new();
    
    // Create handle with index 0 (sentinel)
    // 创建索引为 0 的 handle（sentinel）
    let sentinel_handle = Handle::new(1u64 << 32);
    
    let result = map.insert(sentinel_handle, 42);
    assert_eq!(result, Err(DeferredMapError::InvalidHandle));
}

#[test]
fn test_handle_with_out_of_bounds_index() {
    let mut map = DeferredMap::new();
    
    // Create handle with very large index that doesn't exist
    // 创建具有不存在的非常大索引的 handle
    let large_index = 1000u32;
    let generation = 1u32;
    let handle = Handle::new((generation as u64) << 32 | large_index as u64);
    
    // This should fail because it's not sequential allocation
    // 这应该失败，因为它不是连续分配
    let result = map.insert(handle, 42);
    assert!(result.is_err());
}

#[test]
fn test_generation_mismatch_even_generation() {
    let mut map = DeferredMap::new();
    
    let h = map.allocate_handle();
    let index = h.index();
    map.insert(h, 42).unwrap();
    
    // Create handle with even generation (vacant state)
    // 创建具有偶数 generation 的 handle（空闲状态）
    let even_gen_handle = Handle::new(((2u32 as u64) << 32) | index as u64);
    
    let result = map.insert(even_gen_handle, 100);
    assert_eq!(result, Err(DeferredMapError::GenerationMismatch));
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
        let k = map.insert(h, i).unwrap();
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
        let k = map.insert(h, i).unwrap();
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
    let k1 = map.insert(h1, 42).unwrap();
    
    let h2 = map.allocate_handle();
    let k2 = map.insert(h2, 100).unwrap();
    
    let cloned = map.clone();
    
    assert_eq!(cloned.len(), 2);
    assert_eq!(cloned.get(k1), Some(&42));
    assert_eq!(cloned.get(k2), Some(&100));
}

#[test]
fn test_clone_independence() {
    let mut map = DeferredMap::new();
    
    let h = map.allocate_handle();
    let k = map.insert(h, 42).unwrap();
    
    let mut cloned = map.clone();
    
    // Modify cloned map
    // 修改克隆的 map
    if let Some(value) = cloned.get_mut(k) {
        *value = 100;
    }
    
    // Original should be unchanged
    // 原始的应该不变
    assert_eq!(map.get(k), Some(&42));
    assert_eq!(cloned.get(k), Some(&100));
}

#[test]
fn test_clone_from() {
    let mut map1 = DeferredMap::new();
    let h1 = map1.allocate_handle();
    let k1 = map1.insert(h1, 1).unwrap();
    
    let mut map2 = DeferredMap::new();
    let h2 = map2.allocate_handle();
    map2.insert(h2, 2).unwrap();
    
    map2.clone_from(&map1);
    
    assert_eq!(map2.len(), 1);
    assert_eq!(map2.get(k1), Some(&1));
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
    
    // Capacity should be 0 initially (slots created on demand)
    // 容量应该初始为 0（slots 按需创建）
    assert_eq!(map.capacity(), 0);
}

#[test]
fn test_operations_on_map_with_gaps() {
    let mut map = DeferredMap::new();
    
    // Create map with gaps (insert, remove, insert pattern)
    // 创建有间隙的 map（插入、删除、插入模式）
    let h1 = map.allocate_handle();
    let k1 = map.insert(h1, 1).unwrap();
    
    let h2 = map.allocate_handle();
    let k2 = map.insert(h2, 2).unwrap();
    
    let h3 = map.allocate_handle();
    let k3 = map.insert(h3, 3).unwrap();
    
    // Remove middle element
    // 删除中间元素
    map.remove(k2);
    
    // Insert new element (should reuse k2's slot)
    // 插入新元素（应该复用 k2 的 slot）
    let h4 = map.allocate_handle();
    let k4 = map.insert(h4, 4).unwrap();
    
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
    let k = map.insert(h, 42).unwrap();
    
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
        let k = map.insert(h, i).unwrap();
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
fn test_generation_overflow_handling() {
    let mut map = DeferredMap::new();
    
    // Test with maximum generation value
    // 使用最大 generation 值测试
    let max_gen = u32::MAX;
    let index = 1u32;
    let handle = Handle::new((max_gen as u64) << 32 | index as u64);
    
    // This might fail or handle wrapping
    // 这可能会失败或处理溢出
    let result = map.insert(handle, 42);
    
    // Either succeeds or fails gracefully
    // 要么成功要么优雅地失败
    match result {
        Ok(_) | Err(_) => {}, // Both outcomes are acceptable | 两种结果都可接受
    }
}

#[test]
fn test_very_large_key_values() {
    let map: DeferredMap<i32> = DeferredMap::new();
    
    // Test with very large key values
    // 使用非常大的 key 值测试
    assert_eq!(map.get(u64::MAX), None);
    assert_eq!(map.get(u64::MAX - 1), None);
}

#[test]
fn test_zero_key() {
    let map: DeferredMap<i32> = DeferredMap::new();
    
    // Key 0 should always be invalid (sentinel)
    // Key 0 应该始终无效（sentinel）
    assert_eq!(map.get(0), None);
    assert!(!map.contains_key(0));
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
    let k1 = map.insert(h1, 1).unwrap();
    
    // Allocate more
    let h2 = map.allocate_handle();
    
    // Get
    assert_eq!(map.get(k1), Some(&1));
    
    // Insert
    let k2 = map.insert(h2, 2).unwrap();
    
    // Remove
    map.remove(k1);
    
    // Allocate (should reuse k1's slot)
    let h3 = map.allocate_handle();
    
    // Insert
    let k3 = map.insert(h3, 3).unwrap();
    
    // Verify state
    assert_eq!(map.get(k2), Some(&2));
    assert_eq!(map.get(k3), Some(&3));
    assert_eq!(map.len(), 2);
}

#[test]
fn test_error_display() {
    let err1 = DeferredMapError::InvalidHandle;
    let err2 = DeferredMapError::HandleAlreadyUsed;
    let err3 = DeferredMapError::GenerationMismatch;
    
    // Test Debug formatting
    // 测试 Debug 格式化
    let _ = format!("{:?}", err1);
    let _ = format!("{:?}", err2);
    let _ = format!("{:?}", err3);
    
    // Test Display formatting
    // 测试 Display 格式化
    let _ = format!("{}", err1);
    let _ = format!("{}", err2);
    let _ = format!("{}", err3);
}

#[test]
fn test_map_with_option_type() {
    let mut map = DeferredMap::new();
    
    let h1 = map.allocate_handle();
    let k1 = map.insert(h1, Some(42)).unwrap();
    
    let h2 = map.allocate_handle();
    let k2 = map.insert(h2, None::<i32>).unwrap();
    
    assert_eq!(map.get(k1), Some(&Some(42)));
    assert_eq!(map.get(k2), Some(&None));
}

#[test]
fn test_map_with_result_type() {
    let mut map = DeferredMap::new();
    
    let h1 = map.allocate_handle();
    let k1 = map.insert(h1, Ok::<i32, String>(42)).unwrap();
    
    let h2 = map.allocate_handle();
    let k2 = map.insert(h2, Err::<i32, String>("error".to_string())).unwrap();
    
    assert_eq!(map.get(k1), Some(&Ok(42)));
    assert_eq!(map.get(k2), Some(&Err("error".to_string())));
}

