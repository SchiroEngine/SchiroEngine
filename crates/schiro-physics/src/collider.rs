//! ECS component describing a collision shape attached to an entity.

use bevy_ecs::prelude::*;
use glam::Vec3;

/// Component wrapping a collision shape and a few contact properties.
///
/// The shape is owned by the component so that adding or removing a
/// collider is a single ECS mutation away.
#[derive(Component, Debug, Clone)]
pub struct Collider {
    /// Geometric description of the collider.
    pub shape: ColliderShape,
    /// When `true` the collider generates overlap events but no contact
    /// response. Useful for trigger volumes.
    pub is_sensor: bool,
    /// Coulomb friction coefficient.
    pub friction: f32,
    /// Restitution coefficient (`0` = perfectly inelastic,
    /// `1` = perfectly elastic).
    pub restitution: f32,
}

/// Enumeration of the collision shapes natively supported by the
/// engine.
#[derive(Debug, Clone)]
pub enum ColliderShape {
    /// Sphere defined by its radius.
    Sphere {
        /// Sphere radius, in meters.
        radius: f32,
    },
    /// Axis aligned box defined by its half extents.
    Cuboid {
        /// Half extents along each axis.
        half_extents: Vec3,
    },
    /// Capsule aligned with the Y axis.
    Capsule {
        /// Half distance between the two hemispherical caps.
        half_height: f32,
        /// Radius of the cylindrical part and the caps.
        radius: f32,
    },
    /// Cylinder aligned with the Y axis.
    Cylinder {
        /// Half height of the cylinder.
        half_height: f32,
        /// Radius of the cylinder.
        radius: f32,
    },
    /// Arbitrary triangle mesh, used for static level geometry.
    Trimesh {
        /// Vertex positions of the mesh.
        vertices: Box<[Vec3]>,
        /// Triangle indices into `vertices`.
        indices: Box<[u32]>,
    },
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: ColliderShape::Sphere { radius: 0.5 },
            is_sensor: false,
            friction: 0.5,
            restitution: 0.0,
        }
    }
}
