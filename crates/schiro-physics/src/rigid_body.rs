use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct RigidBody {
    pub kind: RigidBodyKind,
    pub mass: f32,
    pub linear_velocity: glam::Vec3,
    pub angular_velocity: glam::Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyKind {
    Static,
    Dynamic,
    Kinematic,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            kind: RigidBodyKind::Dynamic,
            mass: 1.0,
            linear_velocity: glam::Vec3::ZERO,
            angular_velocity: glam::Vec3::ZERO,
        }
    }
}
