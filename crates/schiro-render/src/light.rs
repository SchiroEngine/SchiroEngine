//! Light source descriptions used by the renderer.
//!
//! The module is intentionally agnostic of the GPU pipeline: lights
//! are described in scene space and converted to GPU friendly
//! representations by the renderer when the actual pipeline lands.

use glam::Vec3;
use schiro_core::Color;

/// Light variant supported by the engine.
pub enum Light {
    /// Infinite light, used for sun and moon.
    Directional(DirectionalLight),
    /// Omnidirectional light with a finite range.
    Point(PointLight),
    /// Cone shaped light source.
    Spot(SpotLight),
}

/// Directional light: parallel rays coming from `direction` toward the
/// scene.
pub struct DirectionalLight {
    /// World space direction the light points toward. Does not need to
    /// be normalized; the renderer normalizes it on upload.
    pub direction: Vec3,
    /// Linear color of the light.
    pub color: Color,
    /// Radiometric intensity multiplier.
    pub intensity: f32,
}

/// Point light: omnidirectional, falls off with distance.
pub struct PointLight {
    /// Linear color of the light.
    pub color: Color,
    /// Radiometric intensity multiplier.
    pub intensity: f32,
    /// Maximum range at which the light has any effect, in meters.
    pub range: f32,
    /// Radius of the light source, used for area light approximations.
    pub radius: f32,
}

/// Spot light: cone defined by inner and outer cutoff angles.
pub struct SpotLight {
    /// World space direction the cone points toward.
    pub direction: Vec3,
    /// Linear color of the light.
    pub color: Color,
    /// Radiometric intensity multiplier.
    pub intensity: f32,
    /// Maximum range at which the light has any effect, in meters.
    pub range: f32,
    /// Radius of the light source for area light approximations.
    pub radius: f32,
    /// Inner cutoff angle, in radians. Inside this cone the light is at
    /// full intensity.
    pub inner_cutoff: f32,
    /// Outer cutoff angle, in radians. Beyond this cone the light has
    /// no effect.
    pub outer_cutoff: f32,
}
