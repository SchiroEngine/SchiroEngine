//! Tests for `schiro_ecs::systems` (propagate_transforms, rotate_entities).

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::Schedule;
use glam::Quat;
use schiro_ecs::components::{GlobalTransform, Rotator, Transform};
use schiro_ecs::systems::{propagate_transforms, rotate_entities, Time};

mod common;
use common::assert_approx_eq;

const EPS: f32 = 1e-5;

#[test]
fn propagate_transforms_copies_into_global() {
    let mut world = World::new();
    let t = Transform {
        translation: glam::Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::IDENTITY,
        scale: glam::Vec3::ONE,
    };
    let e = world.spawn((t, GlobalTransform::default())).id();

    let mut schedule = Schedule::default();
    schedule.add_systems(propagate_transforms);
    schedule.run(&mut world);

    let g = world.get::<GlobalTransform>(e).unwrap().0;
    let p = g.transform_point3(glam::Vec3::ZERO);
    assert_approx_eq(p.x, 1.0, EPS, "gx");
    assert_approx_eq(p.y, 2.0, EPS, "gy");
    assert_approx_eq(p.z, 3.0, EPS, "gz");
}

#[test]
fn propagate_transforms_skips_unchanged() {
    let mut world = World::new();
    let e = world.spawn((Transform::default(), GlobalTransform::default())).id();
    let mut schedule = Schedule::default();
    schedule.add_systems(propagate_transforms);
    schedule.run(&mut world);
    let after = world.get::<GlobalTransform>(e).unwrap().0;
    assert_eq!(after, glam::Mat4::IDENTITY);
}

#[test]
fn rotate_entities_advances_rotation_with_delta() {
    let mut world = World::new();
    world.insert_resource(Time::default());
    let mut t = Time::default();
    t.update(0.0);
    // We bypass the schedule and call the system with a manually
    // controlled delta by inserting a pre-warmed time resource.
    world.insert_resource(t);

    let e = world
        .spawn((
            Transform::default(),
            Rotator { speed: glam::Vec3::new(0.0, std::f32::consts::PI, 0.0) },
        ))
        .id();

    {
        let mut time = world.resource_mut::<Time>();
        time.update(0.5);
    }
    let mut schedule = Schedule::default();
    schedule.add_systems(rotate_entities);
    schedule.run(&mut world);

    let r = world.get::<Transform>(e).unwrap().rotation;
    let expected = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let dot = r.dot(expected).abs();
    assert!(
        (dot - 1.0).abs() < 1e-4,
        "rotation mismatch, dot = {dot}"
    );
}

#[test]
fn rotate_entities_zero_speed_does_nothing() {
    let mut world = World::new();
    world.insert_resource(Time::default());
    let e = world
        .spawn((
            Transform::default(),
            Rotator { speed: glam::Vec3::ZERO },
        ))
        .id();
    {
        let mut t = world.resource_mut::<Time>();
        t.update(1.0);
    }
    let mut schedule = Schedule::default();
    schedule.add_systems(rotate_entities);
    schedule.run(&mut world);
    let r = world.get::<Transform>(e).unwrap().rotation;
    assert_approx_eq(r.x, 0.0, EPS, "rx");
    assert_approx_eq(r.y, 0.0, EPS, "ry");
    assert_approx_eq(r.z, 0.0, EPS, "rz");
    assert_approx_eq(r.w, 1.0, EPS, "rw");
}
