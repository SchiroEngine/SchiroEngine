//! Scene serialization data types.
//!
//! This crate defines the JSON format used by the editor to save
//! and load scenes. The types are pure data (no runtime logic
//! beyond serde derives) so they can be imported by any crate
//! in the workspace without pulling in egui, wgpu or the ECS.

use serde::{Deserialize, Serialize};

/// Top-level layout of a `.srn-scene` JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneFile {
    /// Schema version so the loader can migrate old formats.
    pub version: u32,
    /// Sorted list of entities in the scene.
    pub entities: Vec<EntityDesc>,
}

/// Serializable snapshot of a single entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDesc {
    /// Human-readable entity name.
    pub name: String,
    /// Translation, world units.
    pub translation: [f32; 3],
    /// Rotation, stored as an `(x, y, z, w)` quaternion.
    pub rotation: [f32; 4],
    /// Scale, default `(1, 1, 1)`.
    pub scale: [f32; 3],
    /// Rotator speed, in rad/s per axis. `None` when the entity
    /// has no `Rotator` component.
    pub rotator: Option<[f32; 3]>,
    /// Procedural mesh descriptor. `None` for empty / light
    /// entities that have no mesh renderer.
    pub mesh: Option<MeshDesc>,
}

/// Serializable description of a procedural mesh.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum MeshDesc {
    /// UV sphere.
    Sphere {
        /// Number of slices around the Y axis.
        segments: u32,
        /// Number of stacks from pole to pole.
        rings: u32,
    },
    /// Flat XZ grid.
    Grid {
        /// Number of rows.
        rows: u32,
        /// Number of columns.
        cols: u32,
        /// Distance between two adjacent cells, in world units.
        spacing: f32,
    },
    /// Unit cube.
    Cube,
    /// Unit quad on the XZ plane.
    Plane,
}
