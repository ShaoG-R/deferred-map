# deferred-map

[![Crates.io](https://img.shields.io/crates/v/deferred-map.svg)](https://crates.io/crates/deferred-map)
[![Documentation](https://docs.rs/deferred-map/badge.svg)](https://docs.rs/deferred-map)
[![License](https://img.shields.io/crates/l/deferred-map.svg)](https://github.com/ShaoG-R/deferred-map#license)

[English](README.md) | [中文](README_CN.md)

一个基于**延迟插入句柄**的高性能 Rust 代数型 arena（slotmap）。

## 特性

- **🚀 O(1) 操作**：常数时间的插入、查找和删除
- **🔒 内存安全**：代数索引防止释放后使用（use-after-free）
- **⏳ 延迟插入**：将句柄分配与值插入分离
- **💾 内存高效**：基于 union 的槽位存储优化内存使用
- **🎯 类型安全**：句柄不可克隆，确保单次使用语义
- **⚡ 零拷贝**：直接访问存储的值，无需拷贝

## 为什么需要延迟插入？

传统的 slot map 要求你在分配空间时就准备好值。`DeferredMap` 将这两个步骤分离：

1. **分配句柄** - 预留一个槽位并获取句柄（低成本，不需要值）
2. **插入值** - 稍后使用句柄插入实际的值

这在以下场景特别有用：
- 构建具有循环引用的复杂数据结构
- 在构造值之前需要知道键
- 在执行昂贵计算之前想要预留容量
- 协调多步骤初始化过程

## 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
deferred-map = "0.2"
```

## 快速开始

```rust
use deferred_map::DeferredMap;

fn main() {
    let mut map = DeferredMap::new();
    
    // 步骤 1：分配一个句柄（预留槽位）
    let handle = map.allocate_handle();
    
    // 步骤 2：在插入前获取键
    let key = handle.key();
    
    // 步骤 3：使用句柄插入值
    map.insert(handle, "你好，世界！");
    
    // 访问值
    assert_eq!(map.get(key), Some(&"你好，世界！"));
    
    // 删除值
    assert_eq!(map.remove(key), Some("你好，世界！"));
}
```

## 使用示例

### 基本操作

```rust
use deferred_map::DeferredMap;

let mut map = DeferredMap::new();

// 分配并插入
let handle = map.allocate_handle();
let key = handle.key();
map.insert(handle, 42);

// 获取不可变引用
assert_eq!(map.get(key), Some(&42));

// 获取可变引用
if let Some(value) = map.get_mut(key) {
    *value = 100;
}
assert_eq!(map.get(key), Some(&100));

// 检查是否存在
assert!(map.contains_key(key));

// 删除值
assert_eq!(map.remove(key), Some(100));
assert_eq!(map.get(key), None);
```

### 构建自引用结构

```rust
use deferred_map::DeferredMap;

struct Node {
    value: i32,
    next: Option<u64>, // 指向下一个节点的键
}

let mut graph = DeferredMap::new();

// 先分配句柄
let handle1 = graph.allocate_handle();
let handle2 = graph.allocate_handle();

// 在插入前获取键
let key1 = handle1.key();
let key2 = handle2.key();

// 现在我们可以创建相互引用的节点
let node1 = Node { value: 1, next: Some(key2) };
let node2 = Node { value: 2, next: Some(key1) };

// 插入节点
graph.insert(handle1, node1);
graph.insert(handle2, node2);
```

### 迭代

```rust
use deferred_map::DeferredMap;

let mut map = DeferredMap::new();

for i in 0..5 {
    let handle = map.allocate_handle();
    map.insert(handle, i * 10);
}

// 遍历所有条目
for (key, value) in map.iter() {
    println!("键: {}, 值: {}", key, value);
}

// 可变迭代
for (_, value) in map.iter_mut() {
    *value *= 2;
}
```

### 释放未使用的句柄

```rust
use deferred_map::DeferredMap;

let mut map = DeferredMap::<String>::new();

// 分配一个句柄
let handle = map.allocate_handle();

// 决定不使用它
map.release_handle(handle);

// 槽位被返回到空闲列表
```

### 辅助映射 (Secondary Map)

`SecondaryMap` 允许你将额外数据与 `DeferredMap` 的键相关联，而无需修改原始映射或键结构。

```rust
use deferred_map::{DeferredMap, SecondaryMap};

let mut map = DeferredMap::new();
let mut sec = SecondaryMap::new();

let h1 = map.allocate_handle();
let k1 = h1.key();
map.insert(h1, "玩家 1");

// 关联额外数据
sec.insert(k1, 100); // 生命值

assert_eq!(sec.get(k1), Some(&100));

// 如果键从主映射中移除并被重用，SecondaryMap 会安全处理
map.remove(k1);
// ... 稍后 ...
let h2 = map.allocate_handle(); // 重用槽位
let k2 = h2.key();
map.insert(h2, "玩家 2");

// k1 在 sec 中无效
assert_eq!(sec.get(k1), None);
// k2 是有效的（但在插入前为空）
assert_eq!(sec.get(k2), None);

sec.insert(k2, 200);
assert_eq!(sec.get(k2), Some(&200));
```

## API 概览

### 核心类型

- **`DeferredMap<T>`**：主映射结构
- **`Handle`**：用于延迟插入的一次性令牌（不可克隆）
- **`DeferredMapError`**：句柄操作的错误类型

### 主要方法

#### 创建映射

```rust
DeferredMap::new() -> Self
DeferredMap::with_capacity(capacity: usize) -> Self
```

#### 句柄操作

```rust
allocate_handle(&mut self) -> Handle
insert(&mut self, handle: Handle, value: T)
release_handle(&mut self, handle: Handle)
```

#### Handle 方法

```rust
handle.key() -> u64           // 获取键（在插入前）
handle.index() -> u32         // 获取索引部分
handle.generation() -> u32    // 获取代数部分
```

#### 值访问

```rust
get(&self, key: u64) -> Option<&T>
get_mut(&mut self, key: u64) -> Option<&mut T>
remove(&mut self, key: u64) -> Option<T>
contains_key(&self, key: u64) -> bool
```

#### 元数据与迭代

```rust
len(&self) -> usize
is_empty(&self) -> bool
capacity(&self) -> usize
clear(&mut self)
iter(&self) -> impl Iterator<Item = (u64, &T)>
iter_mut(&mut self) -> impl Iterator<Item = (u64, &mut T)>
```

## 工作原理

### 三态槽位系统

映射中的每个槽位可以处于三种状态之一：

1. **空闲**（Vacant，0b00）：空槽位，是空闲列表的一部分
2. **预留**（Reserved，0b01）：句柄已分配，等待值插入
3. **占用**（Occupied，0b11）：包含有效值

### 代数索引

键是 64 位值，编码为：
- **低 32 位**：槽位索引
- **高 32 位**：版本号（包括状态位）

这防止了 ABA 问题：如果你删除一个值并重用槽位，代数会递增，使旧键失效。

### 内存布局

槽位使用 union 来实现内存效率：

```rust
union SlotUnion<T> {
    value: ManuallyDrop<T>,  // 占用时
    next_free: u32,           // 空闲时（空闲列表指针）
}
```

## 性能特征

- **插入**：O(1) - 从空闲列表弹出
- **查找**：O(1) - 直接数组索引加代数检查
- **删除**：O(1) - 推入空闲列表
- **内存**：槽位被重用，最小分配开销
- **缓存友好**：连续内存布局

## 安全保证

- **无释放后使用**：代数索引确保旧键被拒绝
- **无重复使用**：句柄在使用时被消耗（移动语义）
- **无泄漏**：占用槽位的正确 `Drop` 实现
- **调试安全**：在调试模式下，会验证句柄是否属于正确的映射实例
- **内存安全**：所有 unsafe 代码都被仔细封装和文档化

## 与其他 Crate 的比较

| 特性 | deferred-map | slotmap | slab |
|------|--------------|---------|------|
| 延迟插入 | ✅ | ❌ | ❌ |
| 代数索引 | ✅ | ✅ | ❌ |
| O(1) 操作 | ✅ | ✅ | ✅ |
| 基于句柄的 API | ✅ | ❌ | ❌ |
| 内存高效 | ✅ | ✅ | ✅ |

## 最低支持的 Rust 版本（MSRV）

Rust 1.75 或更高版本（edition 2024）

## 许可证

可以选择以下任一许可证：

- Apache 许可证 2.0 版本（[LICENSE-APACHE](LICENSE-APACHE) 或 http://www.apache.org/licenses/LICENSE-2.0）
- MIT 许可证（[LICENSE-MIT](LICENSE-MIT) 或 http://opensource.org/licenses/MIT）

## 贡献

欢迎贡献！请随时提交 Pull Request。

## 作者

ShaoG <shaog.rs@gmail.com>

## 链接

- [代码仓库](https://github.com/ShaoG-R/deferred-map)
- [文档](https://docs.rs/deferred-map)
- [Crates.io](https://crates.io/crates/deferred-map)

