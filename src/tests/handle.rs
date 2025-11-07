// Handle related comprehensive tests
// Handle 相关的全面测试

use crate::{DeferredMap, Handle, DeferredMapError};

#[test]
fn test_handle_raw_value() {
    let mut map = DeferredMap::<i32>::new();
    let handle = map.allocate_handle();
    
    let raw = handle.raw_value();
    assert!(raw > 0);
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
    
    let raw = handle.raw_value();
    let index = handle.index();
    let generation = handle.generation();
    
    // Verify index extraction
    // 验证 index 提取
    assert_eq!(raw & 0xFFFFFFFF, index as u64);
    
    // Verify generation extraction (upper 30 bits of upper 32 bits)
    // 验证 generation 提取（高 32 位中的高 30 位）
    assert_eq!((raw >> 34), generation as u64);
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
    let key1 = map.insert(handle1, 42).unwrap();
    
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
    let _key = map.insert(handle, 42);
    
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
    
    // Create handle with same raw value
    // 使用相同的 raw 值创建 handle
    let raw = handle1.raw_value();
    let handle3 = Handle::new(raw);
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
    let mut current_key = map.insert(handle1, 1).unwrap();
    
    // Reuse the same slot multiple times
    // 多次复用相同的 slot
    for i in 2..=10 {
        map.remove(current_key);
        
        let handle = map.allocate_handle();
        assert_eq!(handle.index(), index); // Should reuse same slot | 应该复用相同的 slot
        
        let curr_gen = handle.generation();
        assert_eq!(curr_gen, prev_gen + 1); // Generation (high 30 bits) should increment by 1 | Generation（高 30 位）应该递增 1
        prev_gen = curr_gen;
        
        current_key = map.insert(handle, i).unwrap();
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
    let version = 1u32;
    let raw = (version as u64) << 32 | max_index as u64;
    let handle = Handle::new(raw);
    
    assert_eq!(handle.index(), max_index);
    assert_eq!(handle.generation(), version >> 2); // generation is high 30 bits
}

#[test]
fn test_handle_with_max_generation() {
    // Test handle with maximum 30-bit generation
    // 测试最大 30 位 generation 的 handle
    let index = 1u32;
    let max_generation = (1u32 << 30) - 1; // 2^30 - 1, maximum 30-bit value
    let version = (max_generation << 2) | 0b01; // Encode generation in high 30 bits, reserved state in low 2 bits
    let raw = (version as u64) << 32 | index as u64;
    let handle = Handle::new(raw);
    
    assert_eq!(handle.index(), index);
    assert_eq!(handle.generation(), max_generation);
}

