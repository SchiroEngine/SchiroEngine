//! Tests for `schiro_ecs::schedule`.

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::Schedule;
use schiro_ecs::schedule::CoreSet;

#[test]
fn core_set_variants_are_distinct() {
    let sets = [
        CoreSet::First,
        CoreSet::PreUpdate,
        CoreSet::Update,
        CoreSet::PostUpdate,
        CoreSet::Last,
    ];
    for (i, a) in sets.iter().enumerate() {
        for (j, b) in sets.iter().enumerate() {
            assert_eq!(i == j, *a == *b, "sets at {i} and {j}");
        }
    }
}

#[test]
fn core_set_hash_is_consistent() {
    use std::collections::HashSet;
    let mut s = HashSet::new();
    s.insert(CoreSet::Update);
    s.insert(CoreSet::Update);
    s.insert(CoreSet::PostUpdate);
    assert_eq!(s.len(), 2);
}

#[test]
fn schedule_with_empty_systemset_runs() {
    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(do_nothing_system);
    schedule.run(&mut world);
}

fn do_nothing_system() {}

#[test]
fn world_supports_resource_insert() {
    use bevy_ecs::prelude::*;
    let mut world = World::new();
    #[derive(Resource, PartialEq, Debug)]
    struct Counter(u32);
    world.insert_resource(Counter(42));
    assert_eq!(world.resource::<Counter>().0, 42);
}
