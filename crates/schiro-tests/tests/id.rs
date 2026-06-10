//! Tests for `schiro_core::id`.

use schiro_core::id::{new_id_map, IdMap};

#[test]
fn id_map_starts_empty() {
    let map: IdMap<i32> = new_id_map();
    assert_eq!(map.len(), 0);
    assert!(map.is_empty());
}

#[test]
fn id_map_insert_and_get() {
    let mut map: IdMap<u32> = new_id_map();
    let k1 = map.insert(10);
    let k2 = map.insert(20);

    assert_eq!(map[k1], 10);
    assert_eq!(map[k2], 20);
    assert_eq!(map.len(), 2);
}

#[test]
fn id_map_keys_are_distinct() {
    let mut map: IdMap<()> = new_id_map();
    let mut keys = Vec::new();
    for _ in 0..100 {
        keys.push(map.insert(()));
    }
    let unique: std::collections::HashSet<_> = keys.iter().collect();
    assert_eq!(unique.len(), 100);
}

#[test]
fn id_map_remove_drops_value() {
    let mut map: IdMap<&'static str> = new_id_map();
    let k = map.insert("hello");
    assert!(map.remove(k).is_some());
    assert!(map.is_empty());
    assert!(map.get(k).is_none());
}

#[test]
fn id_map_contains_key() {
    let mut map: IdMap<u8> = new_id_map();
    let k = map.insert(7);
    assert!(map.contains_key(k));
    // Looking up a default-constructed key returns false.
    let default_key = schiro_core::id::Id::default();
    assert!(!map.contains_key(default_key));
}

#[test]
fn id_map_iter_yields_all_values() {
    let mut map: IdMap<u32> = new_id_map();
    for v in 0..10 {
        map.insert(v);
    }
    let total: u32 = map.values().sum();
    assert_eq!(total, 45);
}
