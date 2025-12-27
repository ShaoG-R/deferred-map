use crate::DeferredMap;

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "Handle used with wrong map instance")]
fn test_insert_with_wrong_map_handle() {
    let mut map1 = DeferredMap::<i32>::new();
    let mut map2 = DeferredMap::<i32>::new();

    let handle1 = map1.allocate_handle();
    // Try to use handle from map1 on map2
    map2.insert(handle1, 42);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "Handle used with wrong map instance")]
fn test_release_wrong_map_handle() {
    let mut map1 = DeferredMap::<i32>::new();
    let mut map2 = DeferredMap::<i32>::new();

    let handle1 = map1.allocate_handle();
    // Try to release handle from map1 on map2
    map2.release_handle(handle1);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "Handle used with wrong map instance")]
fn test_insert_on_cloned_map_fails() {
    let mut map1 = DeferredMap::<i32>::new();
    let handle1 = map1.allocate_handle();

    // map2 gets a new ID
    let mut map2 = map1.clone();

    // handle1 has map1's ID, so this should panic
    map2.insert(handle1, 42);
}
