//! GPU camera and light uniform layouts.
//!
//! The two structs in this module are uploaded as uniform buffers and
//! consumed by the PBR shader. They are kept `#[repr(C)]` and use
//! [`bytemuck`] so that they can be serialised byte-for-byte.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

/// Camera data uploaded to the GPU each frame.
///
/// Contains the view, projection and view*projection matrices so the
/// shader can pick the one it needs without recomputing it.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CameraUniform {
    /// Combined view * projection matrix, in column major order.
    pub view_proj: [[f32; 4]; 4],
    /// View matrix, in column major order.
    pub view: [[f32; 4]; 4],
    /// Projection matrix, in column major order.
    pub proj: [[f32; 4]; 4],
    /// World space camera position, with `w = 1`.
    pub position: [f32; 4],
}

impl CameraUniform {
    /// Builds an identity camera uniform. Useful as a placeholder.
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            view: Mat4::IDENTITY.to_cols_array_2d(),
            proj: Mat4::IDENTITY.to_cols_array_2d(),
            position: [0.0, 0.0, 0.0, 1.0],
        }
    }

    /// Recomputes every field from the supplied matrices and camera
    /// world position.
    pub fn update(&mut self, view: &Mat4, proj: &Mat4, position: Vec3) {
        self.view = view.to_cols_array_2d();
        self.proj = proj.to_cols_array_2d();
        self.view_proj = (*proj * *view).to_cols_array_2d();
        self.position = [position.x, position.y, position.z, 1.0];
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}

/// Directional light data uploaded to the GPU.
///
/// The fourth component of the `direction` field carries the light
/// intensity; this avoids wasting an entire `vec4` for a single float.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct LightUniform {
    /// World space light direction, normalized. `w` is the intensity.
    pub direction: [f32; 4],
    /// Light color in linear space. `w` is unused and set to `1`.
    pub color: [f32; 4],
    /// Ambient color in linear space. `w` is unused and set to `1`.
    pub ambient: [f32; 4],
}

impl LightUniform {
    /// Builds a directional light with the given direction, color and
    /// ambient term.
    pub fn new(direction: Vec3, color: [f32; 3], intensity: f32, ambient: [f32; 3]) -> Self {
        let d = direction.normalize();
        Self {
            direction: [d.x, d.y, d.z, intensity],
            color: [color[0], color[1], color[2], 1.0],
            ambient: [ambient[0], ambient[1], ambient[2], 1.0],
        }
    }
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            direction: [-0.3, -0.8, -0.2, 1.0],
            color: [0.98, 0.95, 0.9, 1.0],
            ambient: [0.15, 0.15, 0.2, 1.0],
        }
    }
}
