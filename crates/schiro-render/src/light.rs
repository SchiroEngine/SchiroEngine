use glam::Vec3;
use schiro_core::Color;

pub enum Light {
    Directional(DirectionalLight),
    Point(PointLight),
    Spot(SpotLight),
}

pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Color,
    pub intensity: f32,
}

pub struct PointLight {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
}

pub struct SpotLight {
    pub direction: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
    pub inner_cutoff: f32,
    pub outer_cutoff: f32,
}
