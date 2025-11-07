// Handle related comprehensive tests
// Handle 相关的全面测试

use crate::{DeferredMap, Handle, DeferredMapError};

#[test]
fn test_handle_key() {
    let mut map = DeferredMap::<i32>::new();
    let handle = map.allocate_handle();
    
    let key = handle.key();
    assert!(key > 0);
}

#[test]
fn test_handle_index_extraction() {
    let mut map = DeferredMap::<i32>::new();
    let handle = map.allocate_handle();
    
    let index = handle.index();
    // First real slot should be at index 1 (index 0 is sentinel)
    // 第一个真实 slot 应该在索引 1（索引 0 是 sentinel）
    assert_eq!(index, 1);
}

#[test]
fn test_handle_generation_extraction() {
    let mut map = DeferredMap::<i32>::new();
    let handle = map.allocate_handle();
    
    let generation = handle.generation();
    // First generation should be 0 (upper 30 bits of version 0b01)
    // 第一次 generation 应该是 0（version 0b01 的高 30 位）
    assert_eq!(generation, 0);
}

#[test]
fn test_handle_encoding_consistency() {
    let mut map = DeferredMap::<i32>::new();
    let handle = map.allocate_handle();
    
    let key = handle.key();
    let index = handle.index();
    let generation = handle.generation();
    
    // Verify index extraction
    // 验证 index 提取
    assert_eq!(key & 0xFFFFFFFF, index as u64);
    
    // Verify generation extraction (upper 30 bits of upper 32 bits)
    // 验证 generation 提取（高 32 位中的高 30 位）
    assert_eq!((key >> 34), generation as u64);
}

#[test]
fn test_multiple_handles_different_indices() {
    let mut map = DeferredMap::<i32>::new();
    
    let handle1 = map.allocate_handle();
    let handle2 = map.allocate_handle();
    let handle3 = map.allocate_handle();
    
    let index1 = handle1.index();
    let index2 = handle2.index();
    let index3 = handle3.index();
    
    // Each handle should have different index
    // 每个 handle 应该有不同的索引
    assert_ne!(index1, index2);
    assert_ne!(index2, index3);
    assert_ne!(index1, index3);
    
    // Indices should be sequential
    // 索引应该是连续的
    assert_eq!(index1 + 1, index2);
    assert_eq!(index2 + 1, index3);
}

#[test]
fn test_handle_generation_after_reuse() {
    let mut map = DeferredMap::new();
    
    // Allocate and insert
    // 分配并插入
    let handle1 = map.allocate_handle();
    let gen1 = handle1.generation();
    let key1 = handle1.key();
    map.insert(handle1, 42).unwrap();
    
    // Remove to free the slot
    // 移除以释放 slot
    map.remove(key1);
    
    // Allocate again, should reuse same slot with incremented generation
    // 再次分配，应该复用相同的 slot，generation 递增
    let handle2 = map.allocate_handle();
    let gen2 = handle2.generation();
    
    // Generation (high 30 bits) should be incremented by 1
    // Generation（高 30 位）应该递增 1
    assert_eq!(gen2, gen1 + 1);
}

#[test]
fn test_handle_cannot_be_cloned() {
    // This test verifies at compile time that Handle doesn't implement Clone
    // 此测试在编译时验证 Handle 不实现 Clone
    // If this compiles, the test passes
    // 如果编译通过，测试就通过
    
    let mut map = DeferredMap::<i32>::new();
    let handle = map.allocate_handle();
    
    // This should consume the handle
    // 这应该消耗 handle
    map.insert(handle, 42).unwrap();
    
    // Uncommenting the following line should cause a compile error:
    // 取消注释以下行应该导致编译错误：
    // let _key2 = map.insert(handle, 100); // Error: value used after move
}

#[test]
fn test_handle_equality() {
    let mut map = DeferredMap::<i32>::new();
    let handle1 = map.allocate_handle();
    let handle2 = map.allocate_handle();
    
    // Handles should not be equal
    // Handle 不应该相等
    assert_ne!(handle1, handle2);
    
    // Create handle with same key value
    // 使用相同的 key 值创建 handle
    let key = handle1.key();
    let handle3 = Handle::new(key);
    assert_eq!(handle1, handle3);
}

#[test]
fn test_handle_debug_format() {
    let mut map = DeferredMap::<i32>::new();
    let handle = map.allocate_handle();
    
    let debug_str = format!("{:?}", handle);
    assert!(debug_str.contains("Handle"));
}

#[test]
fn test_handle_with_large_index() {
    let mut map = DeferredMap::<i32>::new();
    
    // Allocate many handles to get large indices
    // 分配许多 handle 以获得较大的索引
    let mut handles = Vec::new();
    for _ in 0..1000 {
        handles.push(map.allocate_handle());
    }
    
    let last_handle = handles.last().unwrap();
    let index = last_handle.index();
    
    // Index should be around 1000 (starting from 1)
    // 索引应该接近 1000（从 1 开始）
    assert!(index >= 1000);
}

#[test]
fn test_handle_generation_consistency_after_multiple_reuses() {
    let mut map = DeferredMap::new();
    
    let handle1 = map.allocate_handle();
    let index = handle1.index();
    let mut prev_gen = handle1.generation();
    let mut current_key = handle1.key();
    map.insert(handle1, 1).unwrap();
    
    // Reuse the same slot multiple times
    // 多次复用相同的 slot
    for i in 2..=10 {
        map.remove(current_key);
        
        let handle = map.allocate_handle();
        assert_eq!(handle.index(), index); // Should reuse same slot | 应该复用相同的 slot
        
        let curr_gen = handle.generation();
        assert_eq!(curr_gen, prev_gen + 1); // Generation (high 30 bits) should increment by 1 | Generation（高 30 位）应该递增 1
        prev_gen = curr_gen;
        
        current_key = handle.key();
        map.insert(handle, i).unwrap();
    }
}

#[test]
fn test_invalid_handle_with_zero_index() {
    let mut map = DeferredMap::new();
    
    // Create a handle with index 0 (sentinel index)
    // 创建索引为 0 的 handle（sentinel 索引）
    let invalid_handle = Handle::new(0 | (1u64 << 32));
    
    let result = map.insert(invalid_handle, 42);
    assert_eq!(result, Err(DeferredMapError::InvalidHandle));
}

#[test]
fn test_handle_with_max_index() {
    // Test handle with maximum u32 index
    // 测试最大 u32 索引的 handle
    let max_index = u32::MAX;
    let generation = 1u32;
    let key = (generation as u64) << 32 | max_index as u64;
    let handle = Handle::new(key);
    
    assert_eq!(handle.index(), max_index);
    assert_eq!(handle.generation(), generation); // generation is stored directly in high 32 bits
}

#[test]
fn test_handle_with_max_generation() {
    // Test handle with maximum 32-bit generation
    // 测试最大 32 位 generation 的 handle
    let index = 1u32;
    let max_generation = u32::MAX; // Now generation is full 32 bits
    let key = (max_generation as u64) << 32 | index as u64;
    let handle = Handle::new(key);
    
    assert_eq!(handle.index(), index);
    assert_eq!(handle.generation(), max_generation);
}

// ============================================================================
// release_handle API Tests
// release_handle API 测试
// ============================================================================

#[test]
fn test_release_handle_basic() {
    let mut map = DeferredMap::<i32>::new();
    
    // Allocate a handle but don't use it
    // 分配一个 handle 但不使用
    let handle = map.allocate_handle();
    
    // Release it
    // 释放它
    let result = map.release_handle(handle);
    assert!(result.is_ok());
    
    // Map should still be empty
    // Map 应该仍然为空
    assert_eq!(map.len(), 0);
}

#[test]
fn test_release_handle_allows_reuse() {
    let mut map = DeferredMap::<i32>::new();
    
    // Allocate and release
    // 分配并释放
    let handle1 = map.allocate_handle();
    let index1 = handle1.index();
    map.release_handle(handle1).unwrap();
    
    // Allocate again, should reuse the same slot
    // 再次分配，应该复用相同的 slot
    let handle2 = map.allocate_handle();
    let index2 = handle2.index();
    
    assert_eq!(index1, index2);
}

#[test]
fn test_release_handle_increments_generation() {
    let mut map = DeferredMap::<i32>::new();
    
    // Allocate and release
    // 分配并释放
    let handle1 = map.allocate_handle();
    let gen1 = handle1.generation();
    map.release_handle(handle1).unwrap();
    
    // Allocate again, generation should increment
    // 再次分配，generation 应该递增
    let handle2 = map.allocate_handle();
    let gen2 = handle2.generation();
    
    // Generation (high 30 bits) should increment by 1
    // Generation（高 30 位）应该递增 1
    assert_eq!(gen2, gen1 + 1);
}

#[test]
fn test_release_handle_already_used_fails() {
    let mut map = DeferredMap::new();
    
    // Allocate and insert (use the handle)
    // 分配并插入（使用 handle）
    let handle = map.allocate_handle();
    let key = handle.key();
    map.insert(handle, 42).unwrap();
    
    // Try to release using the key (which is now occupied)
    // 尝试使用 key 释放（现在已被占用）
    let used_handle = Handle::new(key);
    let result = map.release_handle(used_handle);
    
    assert_eq!(result, Err(DeferredMapError::HandleAlreadyUsed));
    
    // Value should still be accessible
    // 值应该仍然可访问
    assert_eq!(map.get(key), Some(&42));
}

#[test]
fn test_release_handle_invalid_handle() {
    let mut map = DeferredMap::<i32>::new();
    
    // Try to release a handle with sentinel index (0)
    // 尝试释放索引为 0 的 handle（sentinel）
    let invalid_handle = Handle::new(1u64 << 32);
    let result = map.release_handle(invalid_handle);
    
    assert_eq!(result, Err(DeferredMapError::InvalidHandle));
}

#[test]
fn test_release_handle_out_of_bounds() {
    let mut map = DeferredMap::<i32>::new();
    
    // Try to release a handle with index that doesn't exist
    // 尝试释放不存在索引的 handle
    let large_index = 1000u32;
    let version = 1u32;
    let invalid_handle = Handle::new((version as u64) << 32 | large_index as u64);
    
    let result = map.release_handle(invalid_handle);
    assert_eq!(result, Err(DeferredMapError::InvalidHandle));
}

#[test]
fn test_release_handle_generation_mismatch() {
    let mut map = DeferredMap::new();
    
    // Allocate, insert, and remove to increment generation
    // 分配、插入和删除以递增 generation
    let handle1 = map.allocate_handle();
    let old_key = handle1.key();
    let key1 = handle1.key();
    map.insert(handle1, 42).unwrap();
    map.remove(key1);
    
    // Allocate again (same slot, new generation)
    // 再次分配（相同的 slot，新的 generation）
    let handle2 = map.allocate_handle();
    map.release_handle(handle2).unwrap();
    
    // Try to release with outdated handle
    // 尝试使用过时的 handle 释放
    let outdated_handle = Handle::new(old_key);
    let result = map.release_handle(outdated_handle);
    
    assert_eq!(result, Err(DeferredMapError::GenerationMismatch));
}

#[test]
fn test_release_handle_multiple_handles() {
    let mut map = DeferredMap::<i32>::new();
    
    // Allocate multiple handles
    // 分配多个 handle
    let handles: Vec<_> = (0..10).map(|_| map.allocate_handle()).collect();
    
    // Release all of them
    // 全部释放
    for handle in handles {
        let result = map.release_handle(handle);
        assert!(result.is_ok());
    }
    
    // Map should still be empty
    // Map 应该仍然为空
    assert_eq!(map.len(), 0);
}

#[test]
fn test_release_handle_doesnt_affect_len() {
    let mut map = DeferredMap::new();
    
    // Insert some elements
    // 插入一些元素
    let h1 = map.allocate_handle();
    map.insert(h1, 1).unwrap();
    
    let h2 = map.allocate_handle();
    map.insert(h2, 2).unwrap();
    
    assert_eq!(map.len(), 2);
    
    // Allocate and release a handle
    // 分配并释放一个 handle
    let h3 = map.allocate_handle();
    map.release_handle(h3).unwrap();
    
    // Length should remain unchanged
    // 长度应该保持不变
    assert_eq!(map.len(), 2);
}

#[test]
fn test_release_handle_interleaved_with_insertions() {
    let mut map = DeferredMap::new();
    
    // Allocate multiple handles
    // 分配多个 handle
    let h1 = map.allocate_handle();
    let k1 = h1.key();
    let h2 = map.allocate_handle();
    let h3 = map.allocate_handle();
    let k3 = h3.key();
    let h4 = map.allocate_handle();
    
    // Insert some, release others
    // 插入一些，释放其他
    map.insert(h1, 1).unwrap();
    map.release_handle(h2).unwrap(); // Release unused | 释放未使用的
    map.insert(h3, 3).unwrap();
    map.release_handle(h4).unwrap(); // Release unused | 释放未使用的
    
    // Verify only inserted values are accessible
    // 验证只有插入的值可访问
    assert_eq!(map.get(k1), Some(&1));
    assert_eq!(map.get(k3), Some(&3));
    assert_eq!(map.len(), 2);
}

#[test]
fn test_release_handle_then_insert_at_same_slot() {
    let mut map = DeferredMap::new();
    
    // Allocate and release
    // 分配并释放
    let handle1 = map.allocate_handle();
    let index1 = handle1.index();
    map.release_handle(handle1).unwrap();
    
    // Allocate again and insert
    // 再次分配并插入
    let handle2 = map.allocate_handle();
    let index2 = handle2.index();
    let key2 = handle2.key();
    map.insert(handle2, 42).unwrap();
    
    // Should use the same index
    // 应该使用相同的索引
    assert_eq!(index1, index2);
    assert_eq!(map.get(key2), Some(&42));
}

#[test]
fn test_release_handle_lifo_order() {
    let mut map = DeferredMap::<i32>::new();
    
    // Allocate multiple handles first (without releasing)
    // 先分配多个 handle（不释放）
    let h1 = map.allocate_handle();
    let idx1 = h1.index();
    
    let h2 = map.allocate_handle();
    let idx2 = h2.index();
    
    let h3 = map.allocate_handle();
    let idx3 = h3.index();
    
    // Release them in order
    // 按顺序释放它们
    map.release_handle(h1).unwrap();
    map.release_handle(h2).unwrap();
    map.release_handle(h3).unwrap();
    
    // Allocate again, should reuse in LIFO order (last released first)
    // 再次分配，应该以 LIFO 顺序复用（最后释放的先用）
    let h4 = map.allocate_handle();
    assert_eq!(h4.index(), idx3);
    
    let h5 = map.allocate_handle();
    assert_eq!(h5.index(), idx2);
    
    let h6 = map.allocate_handle();
    assert_eq!(h6.index(), idx1);
}

#[test]
fn test_release_handle_double_release_fails() {
    let mut map = DeferredMap::<i32>::new();
    
    // Allocate and release
    // 分配并释放
    let handle = map.allocate_handle();
    let key = handle.key();
    map.release_handle(handle).unwrap();
    
    // Try to release again with the same key value
    // 尝试使用相同的 key 值再次释放
    let duplicate_handle = Handle::new(key);
    let result = map.release_handle(duplicate_handle);
    
    // Should fail with generation mismatch
    // 应该因 generation 不匹配而失败
    assert_eq!(result, Err(DeferredMapError::GenerationMismatch));
}

#[test]
fn test_release_handle_after_remove() {
    let mut map = DeferredMap::new();
    
    // Allocate, insert, and remove
    // 分配、插入和删除
    let handle = map.allocate_handle();
    let key = handle.key();
    map.insert(handle, 42).unwrap();
    map.remove(key);
    
    // Try to release using the removed key
    // 尝试使用已删除的 key 释放
    let removed_handle = Handle::new(key);
    let result = map.release_handle(removed_handle);
    
    // Should fail because it's now vacant, not reserved
    // 应该失败，因为现在是 vacant，不是 reserved
    assert_eq!(result, Err(DeferredMapError::GenerationMismatch));
}

#[test]
fn test_release_handle_with_capacity_check() {
    let mut map = DeferredMap::<i32>::new();
    
    // Allocate multiple handles first (to grow capacity)
    // 先分配多个 handle（以增长容量）
    let handles: Vec<_> = (0..10).map(|_| map.allocate_handle()).collect();
    
    // Capacity should have grown
    // 容量应该增长了
    assert_eq!(map.capacity(), 10);
    
    // Now release all of them
    // 现在全部释放
    for h in handles {
        map.release_handle(h).unwrap();
    }
    
    // Capacity should remain
    // 容量应该保持
    assert_eq!(map.capacity(), 10);
    
    // But map should still be empty
    // 但 map 应该仍然为空
    assert_eq!(map.len(), 0);
}

#[test]
fn test_release_handle_mixed_with_removal() {
    let mut map = DeferredMap::new();
    
    // Allocate and insert
    // 分配并插入
    let h1 = map.allocate_handle();
    let k1 = h1.key();
    map.insert(h1, 1).unwrap();
    
    // Allocate and release
    // 分配并释放
    let h2 = map.allocate_handle();
    map.release_handle(h2).unwrap();
    
    // Remove inserted value
    // 删除插入的值
    map.remove(k1);
    
    // Allocate two new handles
    // 分配两个新 handle
    let h3 = map.allocate_handle();
    let k3 = h3.key();
    let h4 = map.allocate_handle();
    let k4 = h4.key();
    
    // Both should reuse slots
    // 两者都应该复用 slot
    map.insert(h3, 3).unwrap();
    map.insert(h4, 4).unwrap();
    
    assert_eq!(map.get(k3), Some(&3));
    assert_eq!(map.get(k4), Some(&4));
    assert_eq!(map.len(), 2);
}

