//! ECS component describing a single rigid body.

use bevy_ecs::prelude::*;

/// Simulation body attached to an entity.
///
/// `linear_velocity` and `angular_velocity` are kept on the component
/// so that the editor can display them without having to query the
/// underlying Rapier state.
#[derive(Component, Debug, Clone)]
pub struct RigidBody {
    /// Dynamic vs. static vs. kinematic behavior of the body.
    pub kind: RigidBodyKind,
    /// Mass in kilograms. Ignored for static and kinematic bodies.
    pub mass: f32,
    /// Current linear velocity, in m/s.
    pub linear_velocity: glam::Vec3,
    /// Current angular velocity, in rad/s, around each world axis.
    pub angular_velocity: glam::Vec3,
}

/// Behavior category of a [`RigidBody`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyKind {
    /// Body never moves. Used for level geometry.
    Static,
    /// Body is fully simulated by the physics engine.
    Dynamic,
    /// Body can be moved by user code but still affects dynamic bodies.
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
