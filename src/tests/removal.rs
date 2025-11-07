// Removal and slot recycling comprehensive tests
// 删除和槽位回收的全面测试

use crate::DeferredMap;

#[test]
fn test_basic_removal() {
    let mut map = DeferredMap::new();
    
    let h = map.allocate_handle();
    let k = map.insert(h, 42).unwrap();
    
    assert_eq!(map.remove(k), Some(42));
    assert_eq!(map.len(), 0);
    assert_eq!(map.get(k), None);
}

#[test]
fn test_removal_returns_correct_value() {
    let mut map = DeferredMap::new();
    
    let h = map.allocate_handle();
    let k = map.insert(h, "Hello".to_string()).unwrap();
    
    let removed = map.remove(k);
    assert_eq!(removed, Some("Hello".to_string()));
}

#[test]
fn test_removal_of_nonexistent_key() {
    let mut map = DeferredMap::<i32>::new();
    
    // Try to remove with invalid key
    // 尝试使用无效的 key 删除
    let result = map.remove(12345);
    assert_eq!(result, None);
}

#[test]
fn test_removal_with_outdated_key() {
    let mut map = DeferredMap::new();
    
    let h = map.allocate_handle();
    let k1 = map.insert(h, 42).unwrap();
    
    // Remove once
    // 删除一次
    map.remove(k1);
    
    // Try to remove again with outdated key
    // 尝试用过时的 key 再次删除
    let result = map.remove(k1);
    assert_eq!(result, None);
}

#[test]
fn test_removal_decrements_len() {
    let mut map = DeferredMap::new();
    
    let mut keys = Vec::new();
    for i in 0..10 {
        let h = map.allocate_handle();
        let k = map.insert(h, i).unwrap();
        keys.push(k);
    }
    
    assert_eq!(map.len(), 10);
    
    for key in keys {
        map.remove(key);
    }
    
    assert_eq!(map.len(), 0);
}

#[test]
fn test_removal_allows_slot_reuse() {
    let mut map = DeferredMap::new();
    
    let h1 = map.allocate_handle();
    let k1 = map.insert(h1, 42).unwrap();
    let index1 = k1 as u32;
    
    // Remove to free slot
    // 删除以释放 slot
    map.remove(k1);
    
    // Allocate new handle, should reuse same slot
    // 分配新 handle，应该复用相同的 slot
    let h2 = map.allocate_handle();
    let index2 = h2.index();
    
    assert_eq!(index1, index2);
}

#[test]
fn test_removal_increments_generation() {
    let mut map = DeferredMap::new();
    
    let h1 = map.allocate_handle();
    let gen1 = h1.generation();
    let k1 = map.insert(h1, 42).unwrap();
    
    // Remove to increment generation
    // 删除以递增 generation
    map.remove(k1);
    
    let h2 = map.allocate_handle();
    let gen2 = h2.generation();
    
    // Generation (high 30 bits) should increment by 1
    // Generation（高 30 位）应该递增 1
    assert_eq!(gen2, gen1 + 1);
}

#[test]
fn test_removal_of_multiple_elements() {
    let mut map = DeferredMap::new();
    
    let mut keys = Vec::new();
    for i in 0..100 {
        let h = map.allocate_handle();
        let k = map.insert(h, i).unwrap();
        keys.push(k);
    }
    
    // Remove even indexed elements
    // 删除偶数索引的元素
    for i in (0..100).step_by(2) {
        let removed = map.remove(keys[i]);
        assert_eq!(removed, Some(i));
    }
    
    assert_eq!(map.len(), 50);
    
    // Verify odd indexed elements still exist
    // 验证奇数索引的元素仍然存在
    for i in (1..100).step_by(2) {
        assert_eq!(map.get(keys[i]), Some(&i));
    }
}

#[test]
fn test_removal_and_reinsertion_cycle() {
    let mut map = DeferredMap::new();
    
    for cycle in 0..10 {
        let h = map.allocate_handle();
        let k = map.insert(h, cycle).unwrap();
        
        assert_eq!(map.get(k), Some(&cycle));
        assert_eq!(map.remove(k), Some(cycle));
        assert_eq!(map.get(k), None);
    }
}

#[test]
fn test_removal_with_custom_drop() {
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
    
    let drop_count = Arc::new(AtomicUsize::new(0));
    
    struct DropCounter {
        count: Arc<AtomicUsize>,
    }
    
    impl Drop for DropCounter {
        fn drop(&mut self) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }
    
    let mut map = DeferredMap::new();
    
    let h = map.allocate_handle();
    let k = map.insert(h, DropCounter {
        count: drop_count.clone(),
    }).unwrap();
    
    // Drop count should be 0 before removal
    // 删除前 drop 计数应该是 0
    assert_eq!(drop_count.load(Ordering::SeqCst), 0);
    
    map.remove(k);
    
    // Drop count should be 1 after removal
    // 删除后 drop 计数应该是 1
    assert_eq!(drop_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_removal_frees_memory() {
    let mut map = DeferredMap::new();
    
    // Insert large strings
    // 插入大字符串
    let mut keys = Vec::new();
    for _ in 0..100 {
        let large_string = format!("{}", "a".repeat(1000));
        let h = map.allocate_handle();
        let k = map.insert(h, large_string).unwrap();
        keys.push(k);
    }
    
    // Remove all
    // 全部删除
    for key in keys {
        map.remove(key);
    }
    
    assert_eq!(map.len(), 0);
}

#[test]
fn test_removal_maintains_other_elements() {
    let mut map = DeferredMap::new();
    
    let mut keys = Vec::new();
    for i in 0..10 {
        let h = map.allocate_handle();
        let k = map.insert(h, i * 10).unwrap();
        keys.push(k);
    }
    
    // Remove middle element
    // 删除中间元素
    map.remove(keys[5]);
    
    // Verify other elements unchanged
    // 验证其他元素不变
    for (i, &key) in keys.iter().enumerate() {
        if i == 5 {
            assert_eq!(map.get(key), None);
        } else {
            assert_eq!(map.get(key), Some(&(i * 10)));
        }
    }
}

#[test]
fn test_removal_pattern_lifo() {
    let mut map = DeferredMap::new();
    
    let mut keys = Vec::new();
    for i in 0..10 {
        let h = map.allocate_handle();
        let k = map.insert(h, i).unwrap();
        keys.push(k);
    }
    
    // Remove in LIFO order (last in, first out)
    // 以 LIFO 顺序删除（后进先出）
    for i in (0..10).rev() {
        let removed = map.remove(keys[i]);
        assert_eq!(removed, Some(i));
    }
    
    assert!(map.is_empty());
}

#[test]
fn test_removal_pattern_fifo() {
    let mut map = DeferredMap::new();
    
    let mut keys = Vec::new();
    for i in 0..10 {
        let h = map.allocate_handle();
        let k = map.insert(h, i).unwrap();
        keys.push(k);
    }
    
    // Remove in FIFO order (first in, first out)
    // 以 FIFO 顺序删除（先进先出）
    for i in 0..10 {
        let removed = map.remove(keys[i]);
        assert_eq!(removed, Some(i));
    }
    
    assert!(map.is_empty());
}

#[test]
fn test_removal_pattern_random() {
    let mut map = DeferredMap::new();
    
    let mut keys = Vec::new();
    for i in 0..20 {
        let h = map.allocate_handle();
        let k = map.insert(h, i).unwrap();
        keys.push(k);
    }
    
    // Remove in pseudo-random pattern
    // 以伪随机模式删除
    let remove_order = vec![5, 15, 2, 18, 0, 10, 7, 12, 3, 17, 1, 14, 8, 19, 4, 11, 6, 13, 9, 16];
    
    for idx in remove_order {
        map.remove(keys[idx]);
    }
    
    assert!(map.is_empty());
}

#[test]
fn test_removal_after_clear() {
    let mut map = DeferredMap::new();
    
    let h = map.allocate_handle();
    let k = map.insert(h, 42).unwrap();
    
    map.clear();
    
    // Try to remove after clear
    // 在 clear 后尝试删除
    let result = map.remove(k);
    assert_eq!(result, None);
}

#[test]
fn test_removal_slot_added_to_free_list() {
    let mut map = DeferredMap::new();
    
    // Insert and remove to create free list
    // 插入并删除以创建空闲列表
    let h1 = map.allocate_handle();
    let k1 = map.insert(h1, 1).unwrap();
    map.remove(k1);
    
    let h2 = map.allocate_handle();
    let k2 = map.insert(h2, 2).unwrap();
    map.remove(k2);
    
    let h3 = map.allocate_handle();
    let k3 = map.insert(h3, 3).unwrap();
    map.remove(k3);
    
    // Allocating new handles should reuse slots in LIFO order
    // 分配新 handle 应该以 LIFO 顺序复用 slot
    let h4 = map.allocate_handle();
    let index4 = h4.index();
    
    // Should reuse the last removed slot
    // 应该复用最后删除的 slot
    let index3 = k3 as u32;
    assert_eq!(index4, index3);
}

#[test]
fn test_removal_with_box_type() {
    let mut map = DeferredMap::new();
    
    let h = map.allocate_handle();
    let k = map.insert(h, Box::new(42)).unwrap();
    
    let removed = map.remove(k);
    assert_eq!(removed, Some(Box::new(42)));
}

#[test]
fn test_removal_doesnt_affect_capacity() {
    let mut map = DeferredMap::new();
    
    for i in 0..10 {
        let h = map.allocate_handle();
        map.insert(h, i).unwrap();
    }
    
    let capacity_before = map.capacity();
    
    // Remove some elements
    // 删除一些元素
    let h = map.allocate_handle();
    let k = map.insert(h, 0).unwrap();
    map.remove(k);
    
    let capacity_after = map.capacity();
    
    // Capacity shouldn't decrease after removal
    // 删除后容量不应该减少
    assert!(capacity_after >= capacity_before);
}

#[test]
fn test_double_removal_fails() {
    let mut map = DeferredMap::new();
    
    let h = map.allocate_handle();
    let k = map.insert(h, 42).unwrap();
    
    // First removal succeeds
    // 第一次删除成功
    assert_eq!(map.remove(k), Some(42));
    
    // Second removal fails
    // 第二次删除失败
    assert_eq!(map.remove(k), None);
}

