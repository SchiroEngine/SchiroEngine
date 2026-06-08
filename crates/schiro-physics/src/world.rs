use glam::Vec3;
use rapier3d::prelude::*;

pub struct PhysicsWorld {
    gravity: Vec3,
    integration_parameters: IntegrationParameters,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            integration_parameters: IntegrationParameters::default(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
        }
    }

    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity;
    }

    pub fn step(&mut self, delta_seconds: f32) {
        self.integration_parameters.dt = delta_seconds;
        // Physics step to be piped through Rapier's pipeline
    }

    pub fn rigid_bodies(&self) -> &RigidBodySet {
        &self.rigid_body_set
    }

    pub fn colliders(&self) -> &ColliderSet {
        &self.collider_set
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}
