//! 3D and 2D physics integration backed by the Rapier engine.
//!
//! The crate exposes:
//!
//! - A [`PhysicsWorld`] wrapper that owns the simulation sets and
//!   advances them by a fixed time step.
//! - The [`rigid_body::RigidBody`] and [`collider::Collider`] ECS components
//!   used to describe dynamic objects.
//! - Joint helpers in [`joint`] and shape-specific queries in
//!   [`query`].
//!
//! # World units
//!
//! SchiroEngine uses a left-handed coordinate system with the Y axis
//! pointing up. Distances are expressed in meters and forces in
//! newtons; the default gravity is `(0, -9.81, 0)`.

#![deny(unsafe_code)]

pub mod collider;
pub mod joint;
pub mod query;
pub mod rigid_body;
pub mod world;

use tracing::info;
pub use world::PhysicsWorld;

/// Logs a startup line. Kept for symmetry with the other engine
/// subsystems; the actual Rapier pipeline is built lazily inside
/// [`PhysicsWorld`].
pub fn init() {
    info!("initializing physics engine");
}
