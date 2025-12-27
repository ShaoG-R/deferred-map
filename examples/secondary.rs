use deferred_map::{DeferredMap, Key, SecondaryMap};

fn main() {
    // 1. Create a primary map (DeferredMap)
    // 1. 创建主映射 (DeferredMap)
    let mut users = DeferredMap::new();

    // 2. Create a secondary map to store user scores
    // 2. 创建辅助映射来存储用户分数
    let mut scores = SecondaryMap::new();

    // 3. Add users and scores
    // 3. 添加用户和分数
    let h_alice = users.allocate_handle();
    let k_alice = h_alice.key();
    users.insert(h_alice, "Alice");
    scores.insert(k_alice, 100);

    let h_bob = users.allocate_handle();
    let k_bob = h_bob.key();
    users.insert(h_bob, "Bob");
    scores.insert(k_bob, 85);

    println!("Users and Scores:");
    for (key, name) in users.iter() {
        if let Some(score) = scores.get(key) {
            println!("{} has score: {}", name, score);
        } else {
            println!("{} has no score", name);
        }
    }

    // 4. Remove a user from primary map
    // 4. 从主映射中删除一个用户
    println!("\nRemoving Alice from primary map...");
    users.remove(k_alice);

    // Note: Removal from DeferredMap does NOT automatically remove data from SecondaryMap.
    // However, since we have the old key 'k_alice', we can still access the old data
    // because the generation stored in SecondaryMap matches k_alice's generation.
    // 注意：从 DeferredMap 中删除不会自动删除 SecondaryMap 中的数据。
    // 但是，由于我们拥有旧 key 'k_alice'，我们仍然可以访问旧数据，
    // 因为 SecondaryMap 中存储的 generation 与 k_alice 的 generation 匹配。
    println!(
        "Can we still access Alice's score with the old key? {:?}",
        scores.get(k_alice)
    );

    // To verify that the key is indeed invalid for the primary map:
    // 验证该 key 对主映射确实无效：
    println!("Is Alice in users map? {}", users.contains_key(k_alice));

    // Manually clean up SecondaryMap
    // 手动清理 SecondaryMap
    scores.remove(k_alice);
    println!("After cleanup, Alice's score: {:?}", scores.get(k_alice));

    // 5. Slot Reuse
    // 5. Slot 复用
    println!("\nAdding Charlie (reuses Alice's slot)...");
    let h_charlie = users.allocate_handle();
    let k_charlie = h_charlie.key();
    users.insert(h_charlie, "Charlie");

    // Compare keys
    println!(
        "Alice Key:   idx={}, gen={}",
        k_alice.index(),
        k_alice.generation()
    );
    println!(
        "Charlie Key: idx={}, gen={}",
        k_charlie.index(),
        k_charlie.generation()
    );

    // Insert score for Charlie.
    // Even if we hadn't removed Alice manually, this would overwrite the slot
    // because Charlie's key has a newer generation.
    // 为 Charlie 插入分数。
    // 即使我们没有手动删除 Alice，这也会覆盖该 slot，
    // 因为 Charlie 的 key 具有更新的 generation。
    scores.insert(k_charlie, 95);

    println!("Charlie's score: {:?}", scores.get(k_charlie));

    // Attempting to access with old key (Alice) will now fail or return None
    // mostly because the slot in SecondaryMap has been updated to the new generation.
    // 尝试使用旧 key (Alice) 访问现在将失败或返回 None，
    // 主要是因为 SecondaryMap 中的 slot 已更新为通过 Charlie 的 key 插入的新 generation。
    println!("Accessing with Alice's old key: {:?}", scores.get(k_alice));
}
