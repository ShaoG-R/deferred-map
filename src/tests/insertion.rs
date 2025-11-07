// Insertion operation comprehensive tests
// 插入操作的全面测试

use crate::{DeferredMap, DeferredMapError};

#[test]
fn test_basic_insertion() {
    let mut map = DeferredMap::new();
    let handle = map.allocate_handle();
    let key = map.insert(handle, 42).unwrap();
    
    assert_eq!(map.get(key), Some(&42));
    assert_eq!(map.len(), 1);
}

#[test]
fn test_multiple_sequential_insertions() {
    let mut map = DeferredMap::new();
    
    let mut keys = Vec::new();
    for i in 0..100 {
        let handle = map.allocate_handle();
        let key = map.insert(handle, i).unwrap();
        keys.push(key);
    }
    
    assert_eq!(map.len(), 100);
    
    for (i, &key) in keys.iter().enumerate() {
        assert_eq!(map.get(key), Some(&i));
    }
}

#[test]
fn test_insertion_with_different_types() {
    // Test with String
    // 测试 String 类型
    let mut map_string = DeferredMap::new();
    let h = map_string.allocate_handle();
    let k = map_string.insert(h, "Hello".to_string()).unwrap();
    assert_eq!(map_string.get(k), Some(&"Hello".to_string()));
    
    // Test with Vec
    // 测试 Vec 类型
    let mut map_vec = DeferredMap::new();
    let h = map_vec.allocate_handle();
    let k = map_vec.insert(h, vec![1, 2, 3]).unwrap();
    assert_eq!(map_vec.get(k), Some(&vec![1, 2, 3]));
    
    // Test with Option
    // 测试 Option 类型
    let mut map_option = DeferredMap::new();
    let h = map_option.allocate_handle();
    let k = map_option.insert(h, Some(42)).unwrap();
    assert_eq!(map_option.get(k), Some(&Some(42)));
}

#[test]
fn test_insertion_with_zero_sized_type() {
    // Test with unit type ()
    // 测试单元类型 ()
    let mut map = DeferredMap::new();
    let handle = map.allocate_handle();
    let key = map.insert(handle, ()).unwrap();
    
    assert_eq!(map.get(key), Some(&()));
    assert_eq!(map.len(), 1);
}

#[test]
fn test_insertion_returns_correct_key() {
    let mut map = DeferredMap::new();
    let handle = map.allocate_handle();
    let handle_raw = handle.raw_value();
    let handle_index = handle.index();
    
    let key = map.insert(handle, 42).unwrap();
    
    // The key should have the same index as the handle
    // key 应该与 handle 有相同的 index
    let key_index = key as u32;
    assert_eq!(key_index, handle_index);
    
    // The version in the key should be occupied (handle version + 2)
    // key 中的 version 应该是 occupied（handle version + 2）
    let handle_version = (handle_raw >> 32) as u32;
    let key_version = (key >> 32) as u32;
    assert_eq!(key_version, handle_version + 2); // reserved(0bXX01) + 2 = occupied(0bXX11)
}

#[test]
fn test_insertion_with_duplicate_handle_fails() {
    let mut map = DeferredMap::new();
    let handle = map.allocate_handle();
    let key = map.insert(handle, 42).unwrap();
    
    // Try to use the same key as a handle again
    // 尝试将相同的 key 再次作为 handle 使用
    use crate::Handle;
    let duplicate_handle = Handle::new(key);
    let result = map.insert(duplicate_handle, 100);
    
    assert_eq!(result, Err(DeferredMapError::HandleAlreadyUsed));
    assert_eq!(map.get(key), Some(&42)); // Original value unchanged | 原值不变
}

#[test]
fn test_insertion_with_outdated_generation_fails() {
    let mut map = DeferredMap::new();
    
    let handle = map.allocate_handle();
    let old_raw = handle.raw_value();
    let key = map.insert(handle, 42).unwrap();
    
    // Remove the value to increment generation
    // 移除值以递增 generation
    map.remove(key);
    
    // Try to insert with outdated handle
    // 尝试使用过时的 handle 插入
    use crate::Handle;
    let outdated_handle = Handle::new(old_raw);
    let result = map.insert(outdated_handle, 100);
    
    assert_eq!(result, Err(DeferredMapError::GenerationMismatch));
}

#[test]
fn test_insertion_extends_slots_vec() {
    let mut map = DeferredMap::new();
    
    // Initial capacity should be 0
    // 初始容量应该是 0
    assert_eq!(map.capacity(), 0);
    
    // Insert first element
    // 插入第一个元素
    let h1 = map.allocate_handle();
    map.insert(h1, 1).unwrap();
    assert_eq!(map.capacity(), 1);
    
    // Insert more elements
    // 插入更多元素
    for i in 2..=10 {
        let h = map.allocate_handle();
        map.insert(h, i).unwrap();
    }
    
    assert_eq!(map.capacity(), 10);
    assert_eq!(map.len(), 10);
}

#[test]
fn test_insertion_after_removal_reuses_slot() {
    let mut map = DeferredMap::new();
    
    let h1 = map.allocate_handle();
    let k1 = map.insert(h1, 42).unwrap();
    let index1 = k1 as u32;
    
    // Remove to free the slot
    // 移除以释放 slot
    map.remove(k1);
    
    // Insert again
    // 再次插入
    let h2 = map.allocate_handle();
    let k2 = map.insert(h2, 100).unwrap();
    let index2 = k2 as u32;
    
    // Should reuse the same index
    // 应该复用相同的索引
    assert_eq!(index1, index2);
    
    // But keys should be different (different generation)
    // 但 key 应该不同（不同的 generation）
    assert_ne!(k1, k2);
}

#[test]
fn test_insertion_with_preallocated_capacity() {
    let mut map = DeferredMap::with_capacity(100);
    
    // Insert elements without extending Vec
    // 插入元素而不扩展 Vec
    for i in 0..50 {
        let h = map.allocate_handle();
        map.insert(h, i).unwrap();
    }
    
    assert_eq!(map.len(), 50);
}

#[test]
fn test_insertion_preserves_previous_values() {
    let mut map = DeferredMap::new();
    
    let mut keys = Vec::new();
    for i in 0..10 {
        let h = map.allocate_handle();
        let k = map.insert(h, i * 10).unwrap();
        keys.push(k);
    }
    
    // Verify all previous values are still accessible
    // 验证所有先前的值仍然可访问
    for (i, &key) in keys.iter().enumerate() {
        assert_eq!(map.get(key), Some(&(i * 10)));
    }
}

#[test]
fn test_insertion_with_large_values() {
    let mut map = DeferredMap::new();
    
    // Insert large string
    // 插入大字符串
    let large_string = "a".repeat(10000);
    let h = map.allocate_handle();
    let k = map.insert(h, large_string.clone()).unwrap();
    
    assert_eq!(map.get(k), Some(&large_string));
}

#[test]
fn test_insertion_maintains_len_count() {
    let mut map = DeferredMap::new();
    
    assert_eq!(map.len(), 0);
    
    let h1 = map.allocate_handle();
    map.insert(h1, 1).unwrap();
    assert_eq!(map.len(), 1);
    
    let h2 = map.allocate_handle();
    map.insert(h2, 2).unwrap();
    assert_eq!(map.len(), 2);
    
    let h3 = map.allocate_handle();
    map.insert(h3, 3).unwrap();
    assert_eq!(map.len(), 3);
}

#[test]
fn test_insertion_with_custom_struct() {
    #[derive(Debug, PartialEq)]
    struct CustomStruct {
        id: u32,
        name: String,
        values: Vec<i32>,
    }
    
    let mut map = DeferredMap::new();
    
    let custom = CustomStruct {
        id: 1,
        name: "Test".to_string(),
        values: vec![1, 2, 3],
    };
    
    let h = map.allocate_handle();
    let k = map.insert(h, custom).unwrap();
    
    if let Some(value) = map.get(k) {
        assert_eq!(value.id, 1);
        assert_eq!(value.name, "Test");
        assert_eq!(value.values, vec![1, 2, 3]);
    } else {
        panic!("Value not found");
    }
}

#[test]
fn test_insertion_interleaved_with_allocations() {
    let mut map = DeferredMap::new();
    
    // Allocate multiple handles first
    // 先分配多个 handle
    let h1 = map.allocate_handle();
    let h2 = map.allocate_handle();
    let h3 = map.allocate_handle();
    
    // Then insert in different order
    // 然后以不同的顺序插入
    let k2 = map.insert(h2, 2).unwrap();
    let k1 = map.insert(h1, 1).unwrap();
    let k3 = map.insert(h3, 3).unwrap();
    
    assert_eq!(map.get(k1), Some(&1));
    assert_eq!(map.get(k2), Some(&2));
    assert_eq!(map.get(k3), Some(&3));
}

#[test]
fn test_insertion_with_box() {
    let mut map = DeferredMap::new();
    
    let boxed_value = Box::new(42);
    let h = map.allocate_handle();
    let k = map.insert(h, boxed_value).unwrap();
    
    assert_eq!(map.get(k), Some(&Box::new(42)));
}

#[test]
fn test_insertion_error_messages() {
    let mut map = DeferredMap::new();
    
    // Test InvalidHandle error
    // 测试 InvalidHandle 错误
    use crate::Handle;
    let invalid_handle = Handle::new(0 | (1u64 << 32));
    let result = map.insert(invalid_handle, 42);
    match result {
        Err(DeferredMapError::InvalidHandle) => {},
        _ => panic!("Expected InvalidHandle error"),
    }
    
    // Test HandleAlreadyUsed error
    // 测试 HandleAlreadyUsed 错误
    let h = map.allocate_handle();
    let k = map.insert(h, 42).unwrap();
    let duplicate = Handle::new(k);
    let result = map.insert(duplicate, 100);
    match result {
        Err(DeferredMapError::HandleAlreadyUsed) => {},
        _ => panic!("Expected HandleAlreadyUsed error"),
    }
}

