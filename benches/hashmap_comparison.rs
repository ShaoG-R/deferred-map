use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use deferred_map::DeferredMap;
use rustc_hash::FxHashMap;
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
        
        group.bench_with_input(BenchmarkId::new("FxHashMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = FxHashMap::default();
                for i in 0..size {
                    map.insert(i, black_box(i));
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
        
        group.bench_with_input(BenchmarkId::new("FxHashMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = FxHashMap::with_capacity_and_hasher(size, Default::default());
                for i in 0..size {
                    map.insert(i, black_box(i));
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
        
        // 准备 FxHashMap
        let mut fx_map = FxHashMap::default();
        for i in 0..*size {
            fx_map.insert(i, i);
        }
        
        group.bench_with_input(BenchmarkId::new("FxHashMap", size), size, |b, &size| {
            b.iter(|| {
                for i in 0..size {
                    black_box(fx_map.get(&i));
                }
            });
        });
    }
    
    group.finish();
}

/// 测试随机查询操作性能
fn bench_random_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_get");
    
    for size in [100, 1000, 10000].iter() {
        // 生成随机查询序列（使用简单的伪随机）
        let query_indices: Vec<usize> = (0..*size)
            .map(|i| (i * 7919) % size)  // 简单的伪随机
            .collect();
        
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
                for &idx in &query_indices {
                    black_box(deferred_map.get(keys[idx]));
                }
            });
        });
        
        // 准备 FxHashMap
        let mut fx_map = FxHashMap::default();
        for i in 0..*size {
            fx_map.insert(i, i);
        }
        
        group.bench_with_input(BenchmarkId::new("FxHashMap", size), size, |b, _| {
            b.iter(|| {
                for &idx in &query_indices {
                    black_box(fx_map.get(&idx));
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
        
        group.bench_with_input(BenchmarkId::new("FxHashMap", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut map = FxHashMap::default();
                    for i in 0..size {
                        map.insert(i, i);
                    }
                    map
                },
                |mut map| {
                    for i in 0..size {
                        black_box(map.remove(&i));
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
        
        // 准备 FxHashMap
        let mut fx_map = FxHashMap::default();
        for i in 0..*size {
            fx_map.insert(i, i);
        }
        
        group.bench_with_input(BenchmarkId::new("FxHashMap", size), size, |b, _| {
            b.iter(|| {
                for (key, value) in &fx_map {
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
        
        group.bench_with_input(BenchmarkId::new("FxHashMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = FxHashMap::default();
                
                // 插入
                for i in 0..size {
                    map.insert(i, black_box(i));
                }
                
                // 查询
                for i in 0..size {
                    black_box(map.get(&i));
                }
                
                // 删除一半
                for i in (0..size).step_by(2) {
                    map.remove(&i);
                }
                
                // 再插入一半
                for i in 0..size / 2 {
                    map.insert(i + size, black_box(i + size));
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
        
        group.bench_with_input(BenchmarkId::new("FxHashMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = FxHashMap::default();
                let mut next_key = 0usize;
                let mut keys = Vec::new();
                
                // 预热：填充一半容量
                for i in 0..size / 2 {
                    map.insert(next_key, black_box(i));
                    keys.push(next_key);
                    next_key += 1;
                }
                
                // 高频插入删除
                for i in 0..size {
                    // 删除一个
                    if i < keys.len() {
                        map.remove(&keys[i]);
                    }
                    
                    // 插入一个
                    map.insert(next_key, black_box(i));
                    if i < keys.len() {
                        keys[i] = next_key;
                    }
                    next_key += 1;
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
        
        // FxHashMap: 需要预分配 keys，然后插入
        group.bench_with_input(BenchmarkId::new("FxHashMap_deferred", size), size, |b, &size| {
            b.iter(|| {
                let mut map = FxHashMap::default();
                let keys: Vec<usize> = (0..size).collect();
                
                // 第二阶段：使用 keys 插入值
                for (i, key) in keys.into_iter().enumerate() {
                    map.insert(key, black_box(i));
                }
                
                map
            });
        });
        
        // FxHashMap: 只能直接插入
        group.bench_with_input(BenchmarkId::new("FxHashMap_direct", size), size, |b, &size| {
            b.iter(|| {
                let mut map = FxHashMap::default();
                for i in 0..size {
                    map.insert(i, black_box(i));
                }
                map
            });
        });
    }
    
    group.finish();
}

/// 测试克隆操作性能
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
        
        // 准备 FxHashMap
        let mut fx_map = FxHashMap::default();
        for i in 0..*size {
            fx_map.insert(i, i);
        }
        
        group.bench_with_input(BenchmarkId::new("FxHashMap", size), size, |b, _| {
            b.iter(|| {
                black_box(fx_map.clone())
            });
        });
    }
    
    group.finish();
}

/// 测试稀疏数据场景（模拟有大量空洞的情况）
fn bench_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("sparse");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("DeferredMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = DeferredMap::new();
                let mut keys = Vec::new();
                
                // 插入所有元素
                for i in 0..size {
                    let handle = map.allocate_handle();
                    let key = map.insert(handle, black_box(i)).unwrap();
                    keys.push(key);
                }
                
                // 删除 90% 的元素，只保留 10%
                for i in 0..size {
                    if i % 10 != 0 {
                        map.remove(keys[i]);
                    }
                }
                
                // 迭代剩余的 10%
                for (key, value) in map.iter() {
                    black_box(key);
                    black_box(value);
                }
                
                map
            });
        });
        
        group.bench_with_input(BenchmarkId::new("FxHashMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = FxHashMap::default();
                
                // 插入所有元素
                for i in 0..size {
                    map.insert(i, black_box(i));
                }
                
                // 删除 90% 的元素，只保留 10%
                for i in 0..size {
                    if i % 10 != 0 {
                        map.remove(&i);
                    }
                }
                
                // 迭代剩余的 10%
                for (key, value) in &map {
                    black_box(key);
                    black_box(value);
                }
                
                map
            });
        });
    }
    
    group.finish();
}

/// 测试缓存友好性（顺序访问）
fn bench_sequential_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_access");
    
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
                let mut sum = 0;
                for &key in &keys {
                    if let Some(value) = deferred_map.get(key) {
                        sum += value;
                    }
                }
                black_box(sum)
            });
        });
        
        // 准备 FxHashMap
        let mut fx_map = FxHashMap::default();
        for i in 0..*size {
            fx_map.insert(i, i);
        }
        
        group.bench_with_input(BenchmarkId::new("FxHashMap", size), size, |b, &size| {
            b.iter(|| {
                let mut sum = 0;
                for i in 0..size {
                    if let Some(value) = fx_map.get(&i) {
                        sum += value;
                    }
                }
                black_box(sum)
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
    bench_random_get,
    bench_remove,
    bench_iter,
    bench_mixed_operations,
    bench_churn,
    bench_deferred_insertion,
    bench_clone,
    bench_sparse,
    bench_sequential_access,
);

criterion_main!(benches);

