use crate::{DeferredMap, SecondaryMap};

#[test]
fn test_secondary_map_basic() {
    let mut map = DeferredMap::new();
    let h1 = map.allocate_handle();
    let k1 = h1.key();
    map.insert(h1, 1);

    let h2 = map.allocate_handle();
    let k2 = h2.key();
    map.insert(h2, 2);

    let mut sec = SecondaryMap::new();

    assert!(sec.is_empty());

    // Insert
    sec.insert(k1, "one");
    sec.insert(k2, "two");

    assert_eq!(sec.len(), 2);
    assert!(sec.contains_key(k1));
    assert!(sec.contains_key(k2));

    // Get
    assert_eq!(sec.get(k1), Some(&"one"));
    assert_eq!(sec.get(k2), Some(&"two"));

    // Mutation
    if let Some(v) = sec.get_mut(k1) {
        *v = "one_modified";
    }
    assert_eq!(sec.get(k1), Some(&"one_modified"));

    // Remove
    assert_eq!(sec.remove(k1), Some("one_modified"));
    assert!(!sec.contains_key(k1));
    assert_eq!(sec.len(), 1);

    // Remove non-existent
    assert_eq!(sec.remove(k1), None);
}

#[test]
fn test_generation_cycle() {
    let mut map = DeferredMap::new();
    let mut sec = SecondaryMap::new();

    // 1. Allocate first handle (Gen A)
    let h1 = map.allocate_handle();
    let k1 = h1.key();
    map.insert(h1, 100);

    // Insert into SecondaryMap
    sec.insert(k1, 10);
    assert_eq!(sec.get(k1), Some(&10));

    // 2. Remove from DeferredMap to free the slot
    map.remove(k1);

    // 3. Allocate second handle (Gen B, should reuse same slot index)
    let h2 = map.allocate_handle();
    let k2 = h2.key();
    map.insert(h2, 200);

    // Verify indexes are same but keys are different (different generations)
    // Note: We can't easily check index without public helper, but behavior implies it.
    assert_ne!(k1, k2);

    // 4. Access SecondaryMap
    // Old key k1 should still work because we haven't touched SecondaryMap yet
    // Note: SecondaryMap doesn't know DeferredMap freed it unless we tell it or overwrite.
    // Standard SlotMap's SecondaryMap behaves this way: it keeps data until overwrite.
    assert_eq!(sec.get(k1), Some(&10));

    // New key k2 should NOT match yet (generation mismatch: stored=Gen A, k2=Gen B)
    assert_eq!(sec.get(k2), None);

    // 5. Insert new key k2
    // This should detect stored generation < k2 generation, and overwrite.
    let old = sec.insert(k2, 20);
    assert_eq!(old, None); // Should return None because generation mismatch (overwrite)

    // Now k2 is valid
    assert_eq!(sec.get(k2), Some(&20));

    // k1 should be invalid (generation mismatch: stored=Gen B, k1=Gen A)
    assert_eq!(sec.get(k1), None);

    // 6. Try to use old key k1 to insert
    // Should be ignored because k1 generation < stored generation
    let res = sec.insert(k1, 999);
    assert_eq!(res, None);
    assert_eq!(sec.get(k2), Some(&20)); // Should not be changed
}

#[test]
fn test_retain_and_clear() {
    let mut map = DeferredMap::new();
    let mut sec = SecondaryMap::new();

    let h1 = map.allocate_handle();
    let k1 = h1.key();
    map.insert(h1, 1);

    let h2 = map.allocate_handle();
    let k2 = h2.key();
    map.insert(h2, 2);

    let h3 = map.allocate_handle();
    let k3 = h3.key();
    map.insert(h3, 3);

    sec.insert(k1, 10);
    sec.insert(k2, 20);
    sec.insert(k3, 30);

    assert_eq!(sec.len(), 3);

    // Retain only values > 15
    sec.retain(|k, v| {
        // Just checking we can access key and value
        assert!(k == k1 || k == k2 || k == k3);
        *v > 15
    });

    assert_eq!(sec.len(), 2);
    assert!(!sec.contains_key(k1));
    assert_eq!(sec.get(k2), Some(&20));
    assert_eq!(sec.get(k3), Some(&30));

    // Clear
    sec.clear();
    assert!(sec.is_empty());
    assert!(!sec.contains_key(k2));
}

#[test]
fn test_iterators() {
    let mut map = DeferredMap::new();
    let mut sec = SecondaryMap::new();

    let h1 = map.allocate_handle();
    let k1 = h1.key();
    map.insert(h1, 0);

    let h2 = map.allocate_handle();
    let k2 = h2.key();
    map.insert(h2, 0);

    sec.insert(k1, 100);
    sec.insert(k2, 200);

    // Iter
    let pairs: std::collections::HashMap<crate::DefaultKey, i32> =
        sec.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(pairs.len(), 2);
    assert_eq!(pairs.get(&k1), Some(&100));
    assert_eq!(pairs.get(&k2), Some(&200));

    // Iter Mut
    for (_, v) in sec.iter_mut() {
        *v += 1;
    }

    assert_eq!(sec.get(k1), Some(&101));
    assert_eq!(sec.get(k2), Some(&201));
}
