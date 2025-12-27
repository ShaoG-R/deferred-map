# deferred-map

[![Crates.io](https://img.shields.io/crates/v/deferred-map.svg)](https://crates.io/crates/deferred-map)
[![Documentation](https://docs.rs/deferred-map/badge.svg)](https://docs.rs/deferred-map)
[![License](https://img.shields.io/crates/l/deferred-map.svg)](https://github.com/ShaoG-R/deferred-map#license)

[English](README.md) | [中文](README_CN.md)

A high-performance generational arena (slotmap) with **handle-based deferred insertion** for Rust.

## Features

- **🚀 O(1) Operations**: Constant-time insertion, lookup, and removal
- **🔒 Memory Safety**: Generational indices prevent use-after-free bugs
- **⏳ Deferred Insertion**: Separate handle allocation from value insertion
- **💾 Memory Efficient**: Union-based slot storage optimizes memory usage
- **🎯 Type Safe**: Handles cannot be cloned, ensuring single-use semantics
- **⚡ Zero-Copy**: Direct access to stored values without copying

## Why Deferred Insertion?

Traditional slot maps require you to have the value ready when allocating space. `DeferredMap` separates these concerns:

1. **Allocate Handle** - Reserve a slot and get a handle (cheap, no value needed)
2. **Insert Value** - Later, use the handle to insert the actual value

This is particularly useful when:
- Building complex data structures with circular references
- Need to know the key before constructing the value
- Want to reserve capacity before expensive computation
- Coordinating multi-step initialization processes

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
deferred-map = "0.2"
```

## Quick Start

```rust
use deferred_map::DeferredMap;

fn main() {
    let mut map = DeferredMap::new();
    
    // Step 1: Allocate a handle (reserves a slot)
    let handle = map.allocate_handle();
    
    // Step 2: Get the key before inserting
    let key = handle.key();
    
    // Step 3: Insert value using the handle
    map.insert(handle, "Hello, World!");
    
    // Access the value
    assert_eq!(map.get(key), Some(&"Hello, World!"));
    
    // Remove the value
    assert_eq!(map.remove(key), Some("Hello, World!"));
}
```

## Usage Examples

### Basic Operations

```rust
use deferred_map::DeferredMap;

let mut map = DeferredMap::new();

// Allocate and insert
let handle = map.allocate_handle();
let key = handle.key();
map.insert(handle, 42);

// Get immutable reference
assert_eq!(map.get(key), Some(&42));

// Get mutable reference
if let Some(value) = map.get_mut(key) {
    *value = 100;
}
assert_eq!(map.get(key), Some(&100));

// Check existence
assert!(map.contains_key(key));

// Remove value
assert_eq!(map.remove(key), Some(100));
assert_eq!(map.get(key), None);
```

### Building Self-Referential Structures

```rust
use deferred_map::DeferredMap;

struct Node {
    value: i32,
    next: Option<u64>, // Key to next node
}

let mut graph = DeferredMap::new();

// Allocate handles first
let handle1 = graph.allocate_handle();
let handle2 = graph.allocate_handle();

// Get the keys before inserting
let key1 = handle1.key();
let key2 = handle2.key();

// Now we can create nodes that reference each other
let node1 = Node { value: 1, next: Some(key2) };
let node2 = Node { value: 2, next: Some(key1) };

// Insert the nodes
graph.insert(handle1, node1);
graph.insert(handle2, node2);
```

### Iteration

```rust
use deferred_map::DeferredMap;

let mut map = DeferredMap::new();

for i in 0..5 {
    let handle = map.allocate_handle();
    map.insert(handle, i * 10);
}

// Iterate over all entries
for (key, value) in map.iter() {
    println!("Key: {}, Value: {}", key, value);
}

// Mutable iteration
for (_, value) in map.iter_mut() {
    *value *= 2;
}
```

### Releasing Unused Handles

```rust
use deferred_map::DeferredMap;

let mut map = DeferredMap::<String>::new();

// Allocate a handle
let handle = map.allocate_handle();

// Decide not to use it
map.release_handle(handle);

// The slot is returned to the free list
```

### Secondary Map

`SecondaryMap` allows you to associate additional data with keys from a `DeferredMap` without modifying the original map or key structure.

```rust
use deferred_map::{DeferredMap, SecondaryMap};

let mut map = DeferredMap::new();
let mut sec = SecondaryMap::new();

let h1 = map.allocate_handle();
let k1 = h1.key();
map.insert(h1, "Player 1");

// Associate extra data
sec.insert(k1, 100); // Health points

assert_eq!(sec.get(k1), Some(&100));

// If the key is removed from the main map and reused, SecondaryMap handles it safe
map.remove(k1);
// ... later ...
let h2 = map.allocate_handle(); // Reuses slot
let k2 = h2.key();
map.insert(h2, "Player 2");

// k1 is invalid in sec
assert_eq!(sec.get(k1), None);
// k2 is valid (but empty until inserted)
assert_eq!(sec.get(k2), None);

sec.insert(k2, 200);
assert_eq!(sec.get(k2), Some(&200));
```

## API Overview

### Core Types

- **`DeferredMap<T>`**: The main map structure
- **`Handle`**: A one-time token for deferred insertion (cannot be cloned)
- **`DeferredMapError`**: Error types for handle operations

### Main Methods

#### Creating a Map

```rust
DeferredMap::new() -> Self
DeferredMap::with_capacity(capacity: usize) -> Self
```

#### Handle Operations

```rust
allocate_handle(&mut self) -> Handle
insert(&mut self, handle: Handle, value: T)
release_handle(&mut self, handle: Handle)
```

#### Handle Methods

```rust
handle.key() -> u64           // Get the key (before insertion)
handle.index() -> u32         // Get the index part
handle.generation() -> u32    // Get the generation part
```

#### Value Access

```rust
get(&self, key: u64) -> Option<&T>
get_mut(&mut self, key: u64) -> Option<&mut T>
remove(&mut self, key: u64) -> Option<T>
contains_key(&self, key: u64) -> bool
```

#### Metadata & Iteration

```rust
len(&self) -> usize
is_empty(&self) -> bool
capacity(&self) -> usize
clear(&mut self)
iter(&self) -> impl Iterator<Item = (u64, &T)>
iter_mut(&mut self) -> impl Iterator<Item = (u64, &mut T)>
```

## How It Works

### Three-State Slot System

Each slot in the map can be in one of three states:

1. **Vacant** (0b00): Empty slot, part of the free list
2. **Reserved** (0b01): Handle allocated, awaiting value insertion
3. **Occupied** (0b11): Contains a valid value

### Generational Indices

Keys are 64-bit values encoding:
- **Lower 32 bits**: Slot index
- **Upper 32 bits**: Version (including state bits)

This prevents the ABA problem: if you remove a value and reuse the slot, the generation increments, invalidating old keys.

### Memory Layout

Slots use a union for memory efficiency:

```rust
union SlotUnion<T> {
    value: ManuallyDrop<T>,  // When occupied
    next_free: u32,           // When vacant (free list pointer)
}
```

## Performance Characteristics

- **Insertion**: O(1) - Pop from free list
- **Lookup**: O(1) - Direct array indexing with generation check
- **Removal**: O(1) - Push to free list
- **Memory**: Slots are reused, minimal allocation overhead
- **Cache Friendly**: Contiguous memory layout

## Safety Guarantees

- **No Use-After-Free**: Generational indices ensure old keys are rejected
- **No Double-Use**: Handles are consumed on use (move semantics)
- **No Leaks**: Proper `Drop` implementation for occupied slots
- **Debug Safety**: In debug mode, handles are verified to belong to the correct map instance
- **Memory Safe**: All unsafe code is carefully encapsulated and documented

## Comparison with Other Crates

| Feature | deferred-map | slotmap | slab |
|---------|--------------|---------|------|
| Deferred Insertion | ✅ | ❌ | ❌ |
| Generational Indices | ✅ | ✅ | ❌ |
| O(1) Operations | ✅ | ✅ | ✅ |
| Handle-based API | ✅ | ❌ | ❌ |
| Memory Efficient | ✅ | ✅ | ✅ |

## Minimum Supported Rust Version (MSRV)

Rust 1.75 or later (edition 2024)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Author

ShaoG <shaog.rs@gmail.com>

## Links

- [Repository](https://github.com/ShaoG-R/deferred-map)
- [Documentation](https://docs.rs/deferred-map)
- [Crates.io](https://crates.io/crates/deferred-map)

