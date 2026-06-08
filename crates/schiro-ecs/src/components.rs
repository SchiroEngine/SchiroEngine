use bevy_ecs::prelude::*;
use glam::{Mat4, Quat, Vec3};

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    #[inline]
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Default::default()
        }
    }

    #[inline]
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    #[inline]
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    #[inline]
    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    #[inline]
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let direction = (target - self.translation).normalize();
        self.rotation = Quat::from_mat4(&Mat4::look_at_rh(self.translation, target, up).inverse());
        _ = direction;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GlobalTransform(pub Mat4);

impl Default for GlobalTransform {
    fn default() -> Self {
        Self(Mat4::IDENTITY)
    }
}

#[derive(Component, Debug, Clone)]
pub struct Name(pub String);

#[derive(Component, Debug, Clone)]
pub struct MeshRenderer {
    pub mesh_handle: Option<usize>,
    pub visible: bool,
}

impl Default for MeshRenderer {
    fn default() -> Self {
        Self {
            mesh_handle: None,
            visible: true,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Rotator {
    pub speed: Vec3,
}

impl Default for Rotator {
    fn default() -> Self {
        Self {
            speed: Vec3::new(0.0, 1.0, 0.0),
        }
    }
}

#[derive(Bundle)]
pub struct SpatialBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for SpatialBundle {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
        }
    }
}
