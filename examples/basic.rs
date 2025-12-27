use deferred_map::{DeferredMap, Key};

fn main() {
    // 1. Create a new DeferredMap
    // 1. 创建一个新的 DeferredMap
    let mut map = DeferredMap::new();

    // 2. Allocate handles first (phase 1)
    // 2. 先分配 handle（第一阶段）
    println!("Allocating handles...");
    let handle1 = map.allocate_handle();
    let handle2 = map.allocate_handle();
    let handle3 = map.allocate_handle();

    // Store keys for later access
    // 保存 key 以便后续访问
    let key1 = handle1.key();
    let key2 = handle2.key();
    let key3 = handle3.key();

    // 3. Insert values using handles (phase 2)
    // 3. 使用 handle 插入值（第二阶段）
    println!("Inserting values...");
    map.insert(handle1, "Alice");
    map.insert(handle2, "Bob");
    map.insert(handle3, "Charlie");

    // 4. Access values using keys
    // 4. 使用 key 访问值
    println!("\nAccessing values:");
    println!("Key1: {:?}", map.get(key1));
    println!("Key2: {:?}", map.get(key2));
    println!("Key3: {:?}", map.get(key3));

    // 5. Modify a value
    // 5. 修改值
    if let Some(val) = map.get_mut(key2) {
        *val = "Bob Updated";
    }
    println!("Key2 after update: {:?}", map.get(key2));

    // 6. Iterate over the map
    // 6. 遍历 map
    println!("\nIterating:");
    for (key, value) in map.iter() {
        println!("Key: {:?}, Value: {}", key, value);
    }

    // 7. Remove a value
    // 7. 删除值
    println!("\nRemoving Key1...");
    let removed = map.remove(key1);
    println!("Removed: {:?}", removed);
    println!("Key1 exists? {}", map.contains_key(key1));

    // 8. Demonstrate reuse of slots
    // 8. 演示 slot 复用
    println!("\nAllocating new handle (should reuse slot)...");
    let handle4 = map.allocate_handle();
    let key4 = handle4.key();
    map.insert(handle4, "Dave");

    // key1 and key4 have the same index but different generations
    // key1 和 key4 具有相同的 index 但 generation 不同
    println!("Key1: index={}, gen={}", key1.index(), key1.generation());
    println!("Key4: index={}, gen={}", key4.index(), key4.generation());

    println!("Key4 value: {:?}", map.get(key4));
    // Trying to access with old key1 will return None
    // 尝试使用旧的 key1 访问将返回 None
    println!("Accessing with old Key1: {:?}", map.get(key1));
}
