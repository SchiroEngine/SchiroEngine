use glam::Vec3;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub max_distance: f32,
}

#[derive(Debug, Clone)]
pub struct RayHit {
    pub entity: bevy_ecs::entity::Entity,
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}
