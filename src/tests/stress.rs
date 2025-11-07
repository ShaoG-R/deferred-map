// Stress tests and performance validation
// 压力测试和性能验证

use crate::DeferredMap;

#[test]
fn test_large_scale_insertions() {
    let mut map = DeferredMap::new();
    
    let count = 10_000;
    let mut keys = Vec::with_capacity(count);
    
    for i in 0..count {
        let h = map.allocate_handle();
        let k = h.key();
        map.insert(h, i).unwrap();
        keys.push(k);
    }
    
    assert_eq!(map.len(), count);
    
    // Verify all values
    // 验证所有值
    for (i, &key) in keys.iter().enumerate() {
        assert_eq!(map.get(key), Some(&i));
    }
}

#[test]
fn test_large_scale_removals() {
    let mut map = DeferredMap::new();
    
    let count = 10_000;
    let mut keys = Vec::with_capacity(count);
    
    // Insert
    // 插入
    for i in 0..count {
        let h = map.allocate_handle();
        let k = h.key();
        map.insert(h, i).unwrap();
        keys.push(k);
    }
    
    // Remove all
    // 全部删除
    for key in keys {
        map.remove(key);
    }
    
    assert_eq!(map.len(), 0);
    assert!(map.is_empty());
}

#[test]
fn test_interleaved_operations_stress() {
    let mut map = DeferredMap::new();
    
    let cycles = 1_000;
    let batch_size = 10;
    
    for _ in 0..cycles {
        let mut keys = Vec::new();
        
        // Insert batch
        // 插入批次
        for i in 0..batch_size {
            let h = map.allocate_handle();
            let k = h.key();
            map.insert(h, i).unwrap();
            keys.push(k);
        }
        
        // Remove half
        // 删除一半
        for i in 0..batch_size/2 {
            map.remove(keys[i]);
        }
    }
    
    // Map should still be consistent
    // Map 应该仍然保持一致
    assert!(map.len() > 0);
}

#[test]
fn test_slot_reuse_stress() {
    let mut map = DeferredMap::new();
    
    let iterations = 1_000;
    
    for i in 0..iterations {
        let h = map.allocate_handle();
        let k = h.key();
        map.insert(h, i).unwrap();
        
        // Immediately remove to stress free list
        // 立即删除以测试空闲列表
        map.remove(k);
    }
    
    // After many allocate-insert-remove cycles, capacity should be minimal
    // 经过多次分配-插入-删除循环后，容量应该保持最小
    assert!(map.capacity() <= 10); // Should reuse slots efficiently | 应该高效地复用 slot
    assert_eq!(map.len(), 0);
}

#[test]
fn test_fragmentation_handling() {
    let mut map = DeferredMap::new();
    
    let count = 1_000;
    let mut keys = Vec::with_capacity(count);
    
    // Fill map
    // 填充 map
    for i in 0..count {
        let h = map.allocate_handle();
        let k = h.key();
        map.insert(h, i).unwrap();
        keys.push(k);
    }
    
    // Remove every other element to create fragmentation
    // 删除每隔一个元素以创建碎片
    for i in (0..count).step_by(2) {
        map.remove(keys[i]);
    }
    
    assert_eq!(map.len(), count / 2);
    
    // Fill gaps with new values
    // 用新值填充间隙
    for i in 0..count/2 {
        let h = map.allocate_handle();
        let k = h.key();
        map.insert(h, i + count).unwrap();
        // Update to new key
        keys[i * 2] = k;
    }
    
    assert_eq!(map.len(), count);
}

#[test]
fn test_alternating_operations() {
    let mut map = DeferredMap::new();
    
    let operations = 5_000;
    let mut keys = Vec::new();
    
    for i in 0..operations {
        if i % 2 == 0 {
            // Insert
            // 插入
            let h = map.allocate_handle();
            let k = h.key();
            map.insert(h, i).unwrap();
            keys.push(k);
        } else if !keys.is_empty() {
            // Remove
            // 删除
            let k = keys.remove(0);
            map.remove(k);
        }
    }
    
    // Verify remaining elements
    // 验证剩余元素
    assert_eq!(map.len(), keys.len());
}

#[test]
fn test_generation_increment_stress() {
    let mut map = DeferredMap::new();
    
    let cycles = 100;
    let h1 = map.allocate_handle();
    let index = h1.index();
    let mut k = h1.key();
    map.insert(h1, 0).unwrap();
    
    // Repeatedly remove and reinsert at same slot
    // 重复地在同一个 slot 删除和重新插入
    for i in 1..=cycles {
        map.remove(k);
        
        let h = map.allocate_handle();
        assert_eq!(h.index(), index); // Should reuse same slot | 应该复用相同的 slot
        
        let new_k = h.key();
        map.insert(h, i).unwrap();
        
        // Old key should be invalid
        // 旧 key 应该无效
        assert_eq!(map.get(k), None);
        assert_eq!(map.get(new_k), Some(&i));
        
        k = new_k;
    }
}

#[test]
fn test_many_handles_allocated_at_once() {
    let mut map = DeferredMap::<i32>::new();
    
    let count = 10_000;
    let mut handles = Vec::with_capacity(count);
    
    // Allocate many handles without inserting
    // 分配许多 handle 但不插入
    for _ in 0..count {
        handles.push(map.allocate_handle());
    }
    
    // Then insert in reverse order
    // 然后以相反的顺序插入
    for (i, handle) in handles.into_iter().rev().enumerate() {
        map.insert(handle, i).unwrap();
    }
    
    assert_eq!(map.len(), count);
}

#[test]
fn test_iterator_stress() {
    let mut map = DeferredMap::new();
    
    let count = 1_000;
    
    // Insert many elements
    // 插入许多元素
    for i in 0..count {
        let h = map.allocate_handle();
        map.insert(h, i).unwrap();
    }
    
    // Multiple iterations
    // 多次迭代
    for _ in 0..10 {
        let sum: usize = map.iter().map(|(_, &v)| v).sum();
        let expected_sum = (0..count).sum();
        assert_eq!(sum, expected_sum);
    }
}

#[test]
fn test_iterator_mut_stress() {
    let mut map = DeferredMap::new();
    
    let count = 1_000;
    
    for i in 0..count {
        let h = map.allocate_handle();
        map.insert(h, i).unwrap();
    }
    
    // Modify all values multiple times
    // 多次修改所有值
    for _ in 0..10 {
        for (_, value) in map.iter_mut() {
            *value += 1;
        }
    }
    
    // Check all values incremented correctly
    // 检查所有值是否正确递增
    for (_, &value) in map.iter() {
        assert!(value >= 10);
    }
}

#[test]
fn test_clear_and_refill_stress() {
    let mut map = DeferredMap::new();
    
    let cycles = 100;
    let elements_per_cycle = 100;
    
    for cycle in 0..cycles {
        // Fill map
        // 填充 map
        for i in 0..elements_per_cycle {
            let h = map.allocate_handle();
            map.insert(h, cycle * elements_per_cycle + i).unwrap();
        }
        
        assert_eq!(map.len(), elements_per_cycle);
        
        // Clear
        // 清空
        map.clear();
        assert_eq!(map.len(), 0);
        assert_eq!(map.capacity(), 0);
    }
}

#[test]
fn test_mixed_type_sizes_stress() {
    // Test with different sized types
    // 测试不同大小的类型
    
    // Small type
    // 小类型
    let mut map_u8 = DeferredMap::new();
    for i in 0..1_000u8 {
        let h = map_u8.allocate_handle();
        map_u8.insert(h, i).unwrap();
    }
    assert_eq!(map_u8.len(), 1_000);
    
    // Large type
    // 大类型
    let mut map_array = DeferredMap::new();
    for i in 0..100 {
        let h = map_array.allocate_handle();
        map_array.insert(h, [i; 64]).unwrap();
    }
    assert_eq!(map_array.len(), 100);
}

#[test]
fn test_clone_stress() {
    let mut map = DeferredMap::new();
    
    let count = 1_000;
    
    for i in 0..count {
        let h = map.allocate_handle();
        map.insert(h, i).unwrap();
    }
    
    // Clone multiple times
    // 多次克隆
    for _ in 0..10 {
        let cloned = map.clone();
        assert_eq!(cloned.len(), count);
        
        // Verify all values in clone
        // 验证克隆中的所有值
        let cloned_sum: usize = cloned.iter().map(|(_, &v)| v).sum();
        let original_sum: usize = map.iter().map(|(_, &v)| v).sum();
        assert_eq!(cloned_sum, original_sum);
    }
}

#[test]
fn test_get_mut_stress() {
    let mut map = DeferredMap::new();
    
    let count = 1_000;
    let mut keys = Vec::new();
    
    for i in 0..count {
        let h = map.allocate_handle();
        let k = h.key();
        map.insert(h, i).unwrap();
        keys.push(k);
    }
    
    // Modify all values through get_mut
    // 通过 get_mut 修改所有值
    for &key in &keys {
        if let Some(value) = map.get_mut(key) {
            *value *= 2;
        }
    }
    
    // Verify modifications
    // 验证修改
    for (i, &key) in keys.iter().enumerate() {
        assert_eq!(map.get(key), Some(&(i * 2)));
    }
}

#[test]
fn test_contains_key_stress() {
    let mut map = DeferredMap::new();
    
    let count = 1_000;
    let mut keys = Vec::new();
    
    for i in 0..count {
        let h = map.allocate_handle();
        let k = h.key();
        map.insert(h, i).unwrap();
        keys.push(k);
    }
    
    // Check all keys exist
    // 检查所有 key 存在
    for &key in &keys {
        assert!(map.contains_key(key));
    }
    
    // Remove half
    // 删除一半
    for i in 0..count/2 {
        map.remove(keys[i]);
    }
    
    // Check removed keys don't exist
    // 检查已删除的 key 不存在
    for i in 0..count/2 {
        assert!(!map.contains_key(keys[i]));
    }
    
    // Check remaining keys still exist
    // 检查剩余的 key 仍然存在
    for i in count/2..count {
        assert!(map.contains_key(keys[i]));
    }
}

#[test]
fn test_capacity_growth_pattern() {
    let mut map = DeferredMap::new();
    
    let mut prev_capacity = 0;
    let max_elements = 1_000;
    
    for i in 0..max_elements {
        let h = map.allocate_handle();
        map.insert(h, i).unwrap();
        
        let curr_capacity = map.capacity();
        
        // Capacity should only increase, never decrease
        // 容量应该只增加，不减少
        assert!(curr_capacity >= prev_capacity);
        prev_capacity = curr_capacity;
    }
    
    assert_eq!(map.capacity(), max_elements);
}

#[test]
fn test_sparse_allocation_pattern() {
    let mut map = DeferredMap::new();
    
    // Create sparse pattern by removing middle elements
    // 通过删除中间元素创建稀疏模式
    let mut keys = Vec::new();
    for i in 0..100 {
        let h = map.allocate_handle();
        let k = h.key();
        map.insert(h, i).unwrap();
        keys.push(k);
    }
    
    // Remove every 3rd element
    // 删除每第3个元素
    for i in (0..100).step_by(3) {
        map.remove(keys[i]);
    }
    
    // Insert new elements to fill gaps
    // 插入新元素以填充间隙
    for i in 0..33 {
        let h = map.allocate_handle();
        map.insert(h, i + 1000).unwrap();
    }
    
    // Should have close to original count
    // 应该接近原始数量
    assert_eq!(map.len(), 100);
}

#[test]
fn test_sequential_vs_random_access() {
    let mut map = DeferredMap::new();
    
    let count = 1_000;
    let mut keys = Vec::new();
    
    for i in 0..count {
        let h = map.allocate_handle();
        let k = h.key();
        map.insert(h, i).unwrap();
        keys.push(k);
    }
    
    // Sequential access
    // 顺序访问
    for (i, &key) in keys.iter().enumerate() {
        assert_eq!(map.get(key), Some(&i));
    }
    
    // Random-like access pattern
    // 类随机访问模式
    let access_pattern = vec![500, 50, 999, 0, 750, 250, 100, 900];
    for &idx in &access_pattern {
        assert_eq!(map.get(keys[idx]), Some(&idx));
    }
}

#[test]
fn test_all_operations_combined_stress() {
    let mut map = DeferredMap::new();
    
    let operations = 5_000;
    let mut keys = Vec::new();
    let mut counter = 0;
    
    for i in 0..operations {
        match i % 5 {
            0 => {
                // Insert
                // 插入
                let h = map.allocate_handle();
                let k = h.key();
                map.insert(h, counter).unwrap();
                keys.push(k);
                counter += 1;
            },
            1 => {
                // Remove
                // 删除
                if !keys.is_empty() {
                    let idx = keys.len() / 2;
                    let k = keys.remove(idx);
                    map.remove(k);
                }
            },
            2 => {
                // Get
                // 获取
                if !keys.is_empty() {
                    let k = keys[keys.len() / 2];
                    let _ = map.get(k);
                }
            },
            3 => {
                // Get mut
                // 可变获取
                if !keys.is_empty() {
                    let k = keys[keys.len() / 2];
                    if let Some(value) = map.get_mut(k) {
                        *value += 1;
                    }
                }
            },
            _ => {
                // Contains key
                // 检查 key
                if !keys.is_empty() {
                    let k = keys[0];
                    let _ = map.contains_key(k);
                }
            }
        }
    }
    
    // Final consistency check
    // 最终一致性检查
    assert_eq!(map.len(), keys.len());
}

#[test]
fn test_memory_efficiency() {
    let mut map = DeferredMap::new();
    
    // Test that removed slots are reused efficiently
    // 测试删除的 slot 是否高效复用
    let initial_insertions = 1_000;
    let mut keys = Vec::new();
    
    for i in 0..initial_insertions {
        let h = map.allocate_handle();
        let k = h.key();
        map.insert(h, i).unwrap();
        keys.push(k);
    }
    
    let capacity_after_initial = map.capacity();
    
    // Remove all
    // 全部删除
    for key in keys {
        map.remove(key);
    }
    
    // Reinsert same amount
    // 重新插入相同数量
    for i in 0..initial_insertions {
        let h = map.allocate_handle();
        map.insert(h, i).unwrap();
    }
    
    let capacity_after_reinsert = map.capacity();
    
    // Capacity should not grow significantly
    // 容量不应该显著增长
    assert_eq!(capacity_after_reinsert, capacity_after_initial);
}

