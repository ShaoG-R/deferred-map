use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use deferred_map::DeferredMap;
use slotmap::SlotMap;
use std::hint::black_box;

// ========== 基础操作测试 ==========

/// 测试插入操作性能
fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("DeferredMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = DeferredMap::new();
                for i in 0..size {
                    let handle = map.allocate_handle();
                    map.insert(handle, black_box(i)).unwrap();
                }
                map
            });
        });
        
        group.bench_with_input(BenchmarkId::new("SlotMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = SlotMap::new();
                for i in 0..size {
                    map.insert(black_box(i));
                }
                map
            });
        });
    }
    
    group.finish();
}

/// 测试预分配 + 插入操作性能
fn bench_preallocated_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("preallocated_insert");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("DeferredMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = DeferredMap::with_capacity(size);
                for i in 0..size {
                    let handle = map.allocate_handle();
                    map.insert(handle, black_box(i)).unwrap();
                }
                map
            });
        });
        
        group.bench_with_input(BenchmarkId::new("SlotMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = SlotMap::with_capacity(size);
                for i in 0..size {
                    map.insert(black_box(i));
                }
                map
            });
        });
    }
    
    group.finish();
}

/// 测试查询操作性能
fn bench_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("get");
    
    for size in [100, 1000, 10000].iter() {
        // 准备 DeferredMap
        let mut deferred_map = DeferredMap::new();
        let mut keys = Vec::new();
        for i in 0..*size {
            let handle = deferred_map.allocate_handle();
            let key = deferred_map.insert(handle, i).unwrap();
            keys.push(key);
        }
        
        group.bench_with_input(BenchmarkId::new("DeferredMap", size), size, |b, _| {
            b.iter(|| {
                for &key in &keys {
                    black_box(deferred_map.get(key));
                }
            });
        });
        
        // 准备 SlotMap
        let mut slot_map = SlotMap::new();
        let mut slot_keys = Vec::new();
        for i in 0..*size {
            let key = slot_map.insert(i);
            slot_keys.push(key);
        }
        
        group.bench_with_input(BenchmarkId::new("SlotMap", size), size, |b, _| {
            b.iter(|| {
                for &key in &slot_keys {
                    black_box(slot_map.get(key));
                }
            });
        });
    }
    
    group.finish();
}

/// 测试删除操作性能
fn bench_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("DeferredMap", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut map = DeferredMap::new();
                    let mut keys = Vec::new();
                    for i in 0..size {
                        let handle = map.allocate_handle();
                        let key = map.insert(handle, i).unwrap();
                        keys.push(key);
                    }
                    (map, keys)
                },
                |(mut map, keys)| {
                    for key in keys {
                        black_box(map.remove(key));
                    }
                    map
                },
                criterion::BatchSize::SmallInput
            );
        });
        
        group.bench_with_input(BenchmarkId::new("SlotMap", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut map = SlotMap::new();
                    let mut keys = Vec::new();
                    for i in 0..size {
                        let key = map.insert(i);
                        keys.push(key);
                    }
                    (map, keys)
                },
                |(mut map, keys)| {
                    for key in keys {
                        black_box(map.remove(key));
                    }
                    map
                },
                criterion::BatchSize::SmallInput
            );
        });
    }
    
    group.finish();
}

/// 测试迭代操作性能
fn bench_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter");
    
    for size in [100, 1000, 10000].iter() {
        // 准备 DeferredMap
        let mut deferred_map = DeferredMap::new();
        for i in 0..*size {
            let handle = deferred_map.allocate_handle();
            deferred_map.insert(handle, i).unwrap();
        }
        
        group.bench_with_input(BenchmarkId::new("DeferredMap", size), size, |b, _| {
            b.iter(|| {
                for (key, value) in deferred_map.iter() {
                    black_box(key);
                    black_box(value);
                }
            });
        });
        
        // 准备 SlotMap
        let mut slot_map = SlotMap::new();
        for i in 0..*size {
            slot_map.insert(i);
        }
        
        group.bench_with_input(BenchmarkId::new("SlotMap", size), size, |b, _| {
            b.iter(|| {
                for (key, value) in &slot_map {
                    black_box(key);
                    black_box(value);
                }
            });
        });
    }
    
    group.finish();
}

// ========== 真实场景测试 ==========

/// 测试混合操作（插入、查询、删除）
fn bench_mixed_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_operations");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("DeferredMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = DeferredMap::new();
                let mut keys = Vec::new();
                
                // 插入
                for i in 0..size {
                    let handle = map.allocate_handle();
                    let key = map.insert(handle, black_box(i)).unwrap();
                    keys.push(key);
                }
                
                // 查询
                for &key in &keys {
                    black_box(map.get(key));
                }
                
                // 删除一半
                for i in (0..size).step_by(2) {
                    map.remove(keys[i]);
                }
                
                // 再插入一半
                for i in 0..size / 2 {
                    let handle = map.allocate_handle();
                    map.insert(handle, black_box(i + size)).unwrap();
                }
                
                map
            });
        });
        
        group.bench_with_input(BenchmarkId::new("SlotMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = SlotMap::new();
                let mut keys = Vec::new();
                
                // 插入
                for i in 0..size {
                    let key = map.insert(black_box(i));
                    keys.push(key);
                }
                
                // 查询
                for &key in &keys {
                    black_box(map.get(key));
                }
                
                // 删除一半
                for i in (0..size).step_by(2) {
                    map.remove(keys[i]);
                }
                
                // 再插入一半
                for i in 0..size / 2 {
                    map.insert(black_box(i + size));
                }
                
                map
            });
        });
    }
    
    group.finish();
}

/// 测试高频插入删除（模拟对象池场景）
fn bench_churn(c: &mut Criterion) {
    let mut group = c.benchmark_group("churn");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("DeferredMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = DeferredMap::new();
                
                // 预热：填充一半容量
                let mut keys = Vec::new();
                for i in 0..size / 2 {
                    let handle = map.allocate_handle();
                    let key = map.insert(handle, black_box(i)).unwrap();
                    keys.push(key);
                }
                
                // 高频插入删除
                for i in 0..size {
                    // 删除一个
                    if i < keys.len() {
                        map.remove(keys[i]);
                    }
                    
                    // 插入一个
                    let handle = map.allocate_handle();
                    let key = map.insert(handle, black_box(i)).unwrap();
                    if i < keys.len() {
                        keys[i] = key;
                    }
                }
                
                map
            });
        });
        
        group.bench_with_input(BenchmarkId::new("SlotMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = SlotMap::new();
                
                // 预热：填充一半容量
                let mut keys = Vec::new();
                for i in 0..size / 2 {
                    let key = map.insert(black_box(i));
                    keys.push(key);
                }
                
                // 高频插入删除
                for i in 0..size {
                    // 删除一个
                    if i < keys.len() {
                        map.remove(keys[i]);
                    }
                    
                    // 插入一个
                    let key = map.insert(black_box(i));
                    if i < keys.len() {
                        keys[i] = key;
                    }
                }
                
                map
            });
        });
    }
    
    group.finish();
}

/// 测试延迟插入场景（DeferredMap 的核心优势）
fn bench_deferred_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("deferred_insertion");
    
    for size in [100, 1000, 10000].iter() {
        // DeferredMap: 先分配所有 handles，然后插入
        group.bench_with_input(BenchmarkId::new("DeferredMap_deferred", size), size, |b, &size| {
            b.iter(|| {
                let mut map = DeferredMap::new();
                let mut handles = Vec::new();
                
                // 第一阶段：分配所有 handles
                for _ in 0..size {
                    let handle = map.allocate_handle();
                    handles.push(handle);
                }
                
                // 第二阶段：使用 handles 插入值
                for (i, handle) in handles.into_iter().enumerate() {
                    map.insert(handle, black_box(i)).unwrap();
                }
                
                map
            });
        });
        
        // DeferredMap: 直接插入（用于对比）
        group.bench_with_input(BenchmarkId::new("DeferredMap_direct", size), size, |b, &size| {
            b.iter(|| {
                let mut map = DeferredMap::new();
                for i in 0..size {
                    let handle = map.allocate_handle();
                    map.insert(handle, black_box(i)).unwrap();
                }
                map
            });
        });
        
        // SlotMap: 只能直接插入
        group.bench_with_input(BenchmarkId::new("SlotMap_direct", size), size, |b, &size| {
            b.iter(|| {
                let mut map = SlotMap::new();
                for i in 0..size {
                    map.insert(black_box(i));
                }
                map
            });
        });
    }
    
    group.finish();
}

/// 测试内存占用（通过克隆操作来间接测试）
fn bench_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone");
    
    for size in [100, 1000, 10000].iter() {
        // 准备 DeferredMap
        let mut deferred_map = DeferredMap::new();
        for i in 0..*size {
            let handle = deferred_map.allocate_handle();
            deferred_map.insert(handle, i).unwrap();
        }
        
        group.bench_with_input(BenchmarkId::new("DeferredMap", size), size, |b, _| {
            b.iter(|| {
                black_box(deferred_map.clone())
            });
        });
        
        // 准备 SlotMap
        let mut slot_map = SlotMap::new();
        for i in 0..*size {
            slot_map.insert(i);
        }
        
        group.bench_with_input(BenchmarkId::new("SlotMap", size), size, |b, _| {
            b.iter(|| {
                black_box(slot_map.clone())
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_insert,
    bench_preallocated_insert,
    bench_get,
    bench_remove,
    bench_iter,
    bench_mixed_operations,
    bench_churn,
    bench_deferred_insertion,
    bench_clone,
);

criterion_main!(benches);

