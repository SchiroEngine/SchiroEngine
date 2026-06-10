//! Tests for the inspector mutation helpers.
//!
//! The editor inspector writes back into the ECS world through
//! `world.get_mut::<Transform>(entity)`. These tests exercise the
//! same code path that the inspector widgets use, without bringing
//! up an egui context.

use bevy_ecs::world::World;
use glam::{Quat, Vec3};
use schiro_ecs::components::{Name, Rotator, Transform};

mod common;
use common::assert_approx_eq;

const EPS: f32 = 1e-5;

#[test]
fn translation_can_be_edited_through_get_mut() {
    let mut world = World::new();
    let e = world.spawn(Transform::from_translation(Vec3::new(1.0, 2.0, 3.0))).id();

    {
        let mut t = world.get_mut::<Transform>(e).unwrap();
        t.translation = Vec3::new(10.0, 20.0, 30.0);
    }
    let t = world.get::<Transform>(e).unwrap();
    assert_eq!(t.translation, Vec3::new(10.0, 20.0, 30.0));
}

#[test]
fn rotation_can_be_edited_through_get_mut() {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id();

    let target = Quat::from_rotation_y(std::f32::consts::FRAC_PI_3);
    {
        let mut t = world.get_mut::<Transform>(e).unwrap();
        t.rotation = target;
    }
    let t = world.get::<Transform>(e).unwrap();
    let dot = t.rotation.dot(target).abs();
    assert!((dot - 1.0).abs() < 1e-5, "rotation drift: {dot}");
}

#[test]
fn scale_can_be_edited_through_get_mut() {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id();

    {
        let mut t = world.get_mut::<Transform>(e).unwrap();
        t.scale = Vec3::new(2.0, 3.0, 4.0);
    }
    let t = world.get::<Transform>(e).unwrap();
    assert_eq!(t.scale, Vec3::new(2.0, 3.0, 4.0));
}

#[test]
fn name_can_be_added_or_edited() {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id();

    // No name yet -> insert.
    assert!(world.get::<Name>(e).is_none());
    world.entity_mut(e).insert(Name("Sphere".into()));
    assert_eq!(world.get::<Name>(e).unwrap().0, "Sphere");

    // Existing name -> update via get_mut.
    {
        let mut n = world.get_mut::<Name>(e).unwrap();
        n.0 = "Cube".into();
    }
    assert_eq!(world.get::<Name>(e).unwrap().0, "Cube");
}

#[test]
fn rotator_can_be_toggled() {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id();
    assert!(world.get::<Rotator>(e).is_none());

    // Add.
    world.entity_mut(e).insert(Rotator::default());
    assert!(world.get::<Rotator>(e).is_some());

    // Remove.
    world.entity_mut(e).remove::<Rotator>();
    assert!(world.get::<Rotator>(e).is_none());
}

#[test]
fn rotation_euler_roundtrip_preserves_value() {
    // The inspector edits rotation as YXZ Euler degrees. The
    // round-trip is not lossless for all orientations, but it
    // should be exact for axis-aligned quaternions.
    let cases: [(f32, f32, f32); 5] =
        [(0.0, 0.0, 0.0), (90.0, 0.0, 0.0), (0.0, 90.0, 0.0), (0.0, 0.0, 90.0), (45.0, 30.0, 60.0)];
    for (yaw, pitch, roll) in cases {
        let q = Quat::from_euler(
            glam::EulerRot::YXZ,
            yaw.to_radians(),
            pitch.to_radians(),
            roll.to_radians(),
        );
        let (y, p, r) = q.to_euler(glam::EulerRot::YXZ);
        assert_approx_eq(y.to_degrees(), yaw, 1e-3, "yaw");
        assert_approx_eq(p.to_degrees(), pitch, 1e-3, "pitch");
        assert_approx_eq(r.to_degrees(), roll, 1e-3, "roll");
    }
}

#[test]
fn inspector_translation_path_does_not_drop_other_components() {
    let mut world = World::new();
    let e = world
        .spawn((
            Name("Ball".into()),
            Transform::from_translation(Vec3::ZERO),
            Rotator { speed: Vec3::new(0.0, 1.0, 0.0) },
        ))
        .id();

    {
        let mut t = world.get_mut::<Transform>(e).unwrap();
        t.translation.y = 5.0;
    }

    assert_eq!(world.get::<Name>(e).unwrap().0, "Ball");
    assert_eq!(world.get::<Transform>(e).unwrap().translation.y, 5.0);
    assert!(world.get::<Rotator>(e).is_some());
}

#[test]
fn transform_can_be_removed() {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id();
    world.entity_mut(e).remove::<Transform>();
    assert!(world.get::<Transform>(e).is_none());
}
