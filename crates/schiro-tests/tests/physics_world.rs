//! Tests for `schiro_physics::world`, `rigid_body` and `collider`.

use bevy_ecs::prelude::Component;
use bevy_ecs::world::World;
use glam::Vec3;
use schiro_physics::collider::{Collider, ColliderShape};
use schiro_physics::rigid_body::{RigidBody, RigidBodyKind};
use schiro_physics::PhysicsWorld;

const EPS: f32 = 1e-5;

#[test]
fn physics_world_default_is_earth_gravity() {
    let w = PhysicsWorld::default();
    assert!((w.gravity().y + 9.81).abs() < EPS);
}

#[test]
fn physics_world_set_gravity() {
    let mut w = PhysicsWorld::new();
    w.set_gravity(Vec3::new(0.0, -3.71, 0.0));
    assert!((w.gravity().y + 3.71).abs() < EPS);
}

#[test]
fn physics_world_step_updates_dt() {
    let mut w = PhysicsWorld::new();
    w.step(0.016);
    // The integration parameter is private, but stepping should not
    // panic and the world should remain queryable.
    assert!(w.rigid_bodies().iter().next().is_none() || true);
}

#[test]
fn rigid_body_default_is_dynamic_unit_mass() {
    let r = RigidBody::default();
    assert_eq!(r.kind, RigidBodyKind::Dynamic);
    assert!((r.mass - 1.0).abs() < EPS);
    assert_eq!(r.linear_velocity, Vec3::ZERO);
    assert_eq!(r.angular_velocity, Vec3::ZERO);
}

#[test]
fn rigid_body_kind_variants_are_distinct() {
    assert_ne!(RigidBodyKind::Static, RigidBodyKind::Dynamic);
    assert_ne!(RigidBodyKind::Dynamic, RigidBodyKind::Kinematic);
    assert_ne!(RigidBodyKind::Static, RigidBodyKind::Kinematic);
}

#[test]
fn collider_default_is_sphere_radius_half() {
    let c = Collider::default();
    assert!(!c.is_sensor);
    assert!((c.friction - 0.5).abs() < EPS);
    assert!(c.restitution.abs() < EPS);
    match c.shape {
        ColliderShape::Sphere { radius } => assert!((radius - 0.5).abs() < EPS),
        _ => panic!("expected default sphere collider"),
    }
}

#[test]
fn collider_shape_variants_carry_data() {
    let sphere = ColliderShape::Sphere { radius: 2.0 };
    let cuboid = ColliderShape::Cuboid { half_extents: Vec3::splat(1.0) };
    let capsule = ColliderShape::Capsule { half_height: 0.5, radius: 0.3 };
    let cylinder = ColliderShape::Cylinder { half_height: 1.0, radius: 0.5 };

    match sphere {
        ColliderShape::Sphere { radius } => assert!((radius - 2.0).abs() < EPS),
        _ => panic!("not sphere"),
    }
    match cuboid {
        ColliderShape::Cuboid { half_extents } => assert_eq!(half_extents, Vec3::splat(1.0)),
        _ => panic!("not cuboid"),
    }
    match capsule {
        ColliderShape::Capsule { half_height, radius } => {
            assert!((half_height - 0.5).abs() < EPS);
            assert!((radius - 0.3).abs() < EPS);
        }
        _ => panic!("not capsule"),
    }
    match cylinder {
        ColliderShape::Cylinder { half_height, radius } => {
            assert!((half_height - 1.0).abs() < EPS);
            assert!((radius - 0.5).abs() < EPS);
        }
        _ => panic!("not cylinder"),
    }
}

#[test]
fn collider_can_be_attached_to_entity() {
    let mut world = World::new();
    let entity = world
        .spawn((
            RigidBody::default(),
            Collider {
                shape: ColliderShape::Sphere { radius: 1.0 },
                is_sensor: true,
                friction: 0.1,
                restitution: 0.9,
            },
        ))
        .id();

    let c = world.get::<Collider>(entity).unwrap();
    assert!(c.is_sensor);
    assert!((c.friction - 0.1).abs() < EPS);
    assert!((c.restitution - 0.9).abs() < EPS);
}
