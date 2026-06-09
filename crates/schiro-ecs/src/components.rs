//! ECS components used by the SchiroEngine runtime and editor.
//!
//! These components are intentionally tiny: they are pure data and never
//! contain logic. The matching behavior lives in [`crate::systems`].

use bevy_ecs::prelude::*;
use glam::{Mat4, Quat, Vec3};

/// Position, rotation and scale of an entity in its local frame.
///
/// Convert to a world matrix through [`Transform::compute_matrix`].
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// Local position.
    pub translation: Vec3,
    /// Local rotation.
    pub rotation: Quat,
    /// Local scale.
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
    /// Builds a transform located at `translation` with the identity
    /// rotation and unit scale.
    #[inline]
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Default::default()
        }
    }

    /// Builds a transform with the given rotation, located at the origin
    /// with unit scale.
    #[inline]
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    /// Builds a transform with the given scale, located at the origin
    /// with the identity rotation.
    #[inline]
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    /// Composes the local position, rotation and scale into a single
    /// 4x4 matrix.
    #[inline]
    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Rotates the transform so it points toward `target` using `up` as
    /// the world up vector.
    #[inline]
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let direction = (target - self.translation).normalize();
        self.rotation = Quat::from_mat4(&Mat4::look_at_rh(self.translation, target, up).inverse());
        _ = direction;
    }
}

/// Cached world transform.
///
/// The [`crate::systems::propagate_transforms`] system updates this value
/// from [`Transform`].
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GlobalTransform(pub Mat4);

impl Default for GlobalTransform {
    fn default() -> Self {
        Self(Mat4::IDENTITY)
    }
}

/// Human readable name attached to an entity, displayed in the editor.
#[derive(Component, Debug, Clone)]
pub struct Name(pub String);

/// Marks an entity as a renderable mesh.
///
/// `mesh_handle` is an opaque index used by the renderer to look up the
/// matching GPU mesh. The field is optional so the component can be
/// inserted before the mesh is uploaded.
#[derive(Component, Debug, Clone)]
pub struct MeshRenderer {
    /// Index of the mesh in the renderer's storage. `None` when no mesh
    /// is currently bound.
    pub mesh_handle: Option<usize>,
    /// When `false`, the entity is skipped during the render sync.
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

/// Component that rotates the attached entity every frame.
#[derive(Component, Debug, Clone)]
pub struct Rotator {
    /// Per-axis angular speed, in radians per second.
    pub speed: Vec3,
}

impl Default for Rotator {
    fn default() -> Self {
        Self {
            speed: Vec3::new(0.0, 1.0, 0.0),
        }
    }
}

/// Convenience bundle grouping a [`Transform`] with its
/// [`GlobalTransform`] cache.
#[derive(Bundle)]
pub struct SpatialBundle {
    /// Local transform.
    pub transform: Transform,
    /// Cached global transform.
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
