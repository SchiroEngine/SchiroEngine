//! Physics queries: raycasts, shape casts and overlap tests.

use glam::Vec3;

/// Infinite ray used for picking and line-of-sight checks.
///
/// The `max_distance` field lets the caller early-out of expensive
/// intersection tests once a maximum range is reached.
pub struct Ray {
    /// World space origin of the ray.
    pub origin: Vec3,
    /// Normalized world space direction.
    pub direction: Vec3,
    /// Maximum distance to test. The query is dropped as soon as this
    /// distance is exceeded.
    pub max_distance: f32,
}

/// Result of a successful raycast.
#[derive(Debug, Clone)]
pub struct RayHit {
    /// Entity that was hit, if any.
    pub entity: bevy_ecs::entity::Entity,
    /// World space point of the intersection.
    pub point: Vec3,
    /// World space normal at the intersection.
    pub normal: Vec3,
    /// Distance from the ray origin to the intersection.
    pub distance: f32,
}
