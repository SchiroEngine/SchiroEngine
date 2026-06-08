use bevy_ecs::prelude::*;
use glam::Vec3;

#[derive(Component, Debug, Clone)]
pub struct Collider {
    pub shape: ColliderShape,
    pub is_sensor: bool,
    pub friction: f32,
    pub restitution: f32,
}

#[derive(Debug, Clone)]
pub enum ColliderShape {
    Sphere { radius: f32 },
    Cuboid { half_extents: Vec3 },
    Capsule { half_height: f32, radius: f32 },
    Cylinder { half_height: f32, radius: f32 },
    Trimesh { vertices: Box<[Vec3]>, indices: Box<[u32]> },
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
