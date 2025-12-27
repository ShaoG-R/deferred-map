// Insertion operation comprehensive tests
// 插入操作的全面测试

use crate::DeferredMap;

#[test]
fn test_basic_insertion() {
    let mut map = DeferredMap::new();
    let handle = map.allocate_handle();
    let key = handle.key();
    map.insert(handle, 42);

    assert_eq!(map.get(key), Some(&42));
    assert_eq!(map.len(), 1);
}

#[test]
fn test_multiple_sequential_insertions() {
    let mut map = DeferredMap::new();

    let mut keys = Vec::new();
    for i in 0..100 {
        let handle = map.allocate_handle();
        let key = handle.key();
        map.insert(handle, i);
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
    let k = h.key();
    map_string.insert(h, "Hello".to_string());
    assert_eq!(map_string.get(k), Some(&"Hello".to_string()));

    // Test with Vec
    // 测试 Vec 类型
    let mut map_vec = DeferredMap::new();
    let h = map_vec.allocate_handle();
    let k = h.key();
    map_vec.insert(h, vec![1, 2, 3]);
    assert_eq!(map_vec.get(k), Some(&vec![1, 2, 3]));

    // Test with Option
    // 测试 Option 类型
    let mut map_option = DeferredMap::new();
    let h = map_option.allocate_handle();
    let k = h.key();
    map_option.insert(h, Some(42));
    assert_eq!(map_option.get(k), Some(&Some(42)));
}

#[test]
fn test_insertion_with_zero_sized_type() {
    // Test with unit type ()
    // 测试单元类型 ()
    let mut map = DeferredMap::new();
    let handle = map.allocate_handle();
    let key = handle.key();
    map.insert(handle, ());

    assert_eq!(map.get(key), Some(&()));
    assert_eq!(map.len(), 1);
}

#[test]
fn test_insertion_returns_correct_key() {
    let mut map = DeferredMap::new();
    let handle = map.allocate_handle();
    let handle_key = handle.key();
    let handle_index = handle.index();

    map.insert(handle, 42);

    // The key should have the same index as the handle
    // key 应该与 handle 有相同的 index
    let key_index = handle_key.index();
    assert_eq!(key_index, handle_index);

    // Verify the value is accessible with the key
    // 验证可以使用 key 访问值
    assert_eq!(map.get(handle_key), Some(&42));
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
    map.insert(h1, 1);
    assert!(map.capacity() >= 1);

    // Insert more elements
    // 插入更多元素
    for i in 2..=10 {
        let h = map.allocate_handle();
        map.insert(h, i);
    }

    assert!(map.capacity() >= 10);
    assert_eq!(map.len(), 10);
}

#[test]
fn test_insertion_after_removal_reuses_slot() {
    let mut map = DeferredMap::new();

    let h1 = map.allocate_handle();
    let k1 = h1.key();
    map.insert(h1, 42);
    let index1 = k1.index();

    // Remove to free the slot
    // 移除以释放 slot
    map.remove(k1);

    // Insert again
    // 再次插入
    let h2 = map.allocate_handle();
    let k2 = h2.key();
    map.insert(h2, 100);
    let index2 = k2.index();

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
        map.insert(h, i);
    }

    assert_eq!(map.len(), 50);
}

#[test]
fn test_insertion_preserves_previous_values() {
    let mut map = DeferredMap::new();

    let mut keys = Vec::new();
    for i in 0..10 {
        let h = map.allocate_handle();
        let k = h.key();
        map.insert(h, i * 10);
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
    let k = h.key();
    map.insert(h, large_string.clone());

    assert_eq!(map.get(k), Some(&large_string));
}

#[test]
fn test_insertion_maintains_len_count() {
    let mut map = DeferredMap::new();

    assert_eq!(map.len(), 0);

    let h1 = map.allocate_handle();
    map.insert(h1, 1);
    assert_eq!(map.len(), 1);

    let h2 = map.allocate_handle();
    map.insert(h2, 2);
    assert_eq!(map.len(), 2);

    let h3 = map.allocate_handle();
    map.insert(h3, 3);
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
    let k = h.key();
    map.insert(h, custom);

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
    let k1 = h1.key();
    let h2 = map.allocate_handle();
    let k2 = h2.key();
    let h3 = map.allocate_handle();
    let k3 = h3.key();

    // Then insert in different order
    // 然后以不同的顺序插入
    map.insert(h2, 2);
    map.insert(h1, 1);
    map.insert(h3, 3);

    assert_eq!(map.get(k1), Some(&1));
    assert_eq!(map.get(k2), Some(&2));
    assert_eq!(map.get(k3), Some(&3));
}

#[test]
fn test_insertion_with_box() {
    let mut map = DeferredMap::new();

    let boxed_value = Box::new(42);
    let h = map.allocate_handle();
    let k = h.key();
    map.insert(h, boxed_value);

    assert_eq!(map.get(k), Some(&Box::new(42)));
}
