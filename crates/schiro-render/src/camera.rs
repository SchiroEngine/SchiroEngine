use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub position: [f32; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            view: Mat4::IDENTITY.to_cols_array_2d(),
            proj: Mat4::IDENTITY.to_cols_array_2d(),
            position: [0.0, 0.0, 0.0, 1.0],
        }
    }

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

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct LightUniform {
    pub direction: [f32; 4],
    pub color: [f32; 4],
    pub ambient: [f32; 4],
}

impl LightUniform {
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
