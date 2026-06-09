//! 3D physics world wrapper around the Rapier simulation sets.
//!
//! The world owns the rigid body, collider and integration parameter
//! storage. The actual solver pipeline will be plugged in here once the
//! shape -> collider conversion is finalized.

use glam::Vec3;
use rapier3d::prelude::*;

/// Owns the Rapier simulation sets used by the runtime and editor.
///
/// A [`PhysicsWorld`] is cheap to clone because every member is either
/// `Copy` or backed by an internal arena.
pub struct PhysicsWorld {
    /// Gravity vector, in m/s^2. Defaults to Earth gravity on -Y.
    gravity: Vec3,
    /// Rapier integration parameters (fixed step, contact tolerance...).
    integration_parameters: IntegrationParameters,
    /// All rigid bodies currently registered with the world.
    rigid_body_set: RigidBodySet,
    /// All colliders currently registered with the world.
    collider_set: ColliderSet,
}

impl PhysicsWorld {
    /// Builds an empty world with the default gravity vector.
    pub fn new() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            integration_parameters: IntegrationParameters::default(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
        }
    }

    /// Updates the gravity vector used by the next call to [`Self::step`].
    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity;
    }

    /// Returns the current gravity vector.
    pub fn gravity(&self) -> Vec3 {
        self.gravity
    }

    /// Advances the simulation by `delta_seconds` of virtual time.
    ///
    /// The current implementation only updates the cached step size;
    /// the full Rapier pipeline is plugged in here as soon as the
    /// collider conversion is ready.
    pub fn step(&mut self, delta_seconds: f32) {
        self.integration_parameters.dt = delta_seconds;
        // Physics step to be piped through Rapier's pipeline.
    }

    /// Read only access to the underlying rigid body set.
    pub fn rigid_bodies(&self) -> &RigidBodySet {
        &self.rigid_body_set
    }

    /// Read only access to the underlying collider set.
    pub fn colliders(&self) -> &ColliderSet {
        &self.collider_set
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}
