//! Concrete asset data structures produced by the engine loaders and
//! generators.
//!
//! These types are deliberately `Clone`-friendly so the
//! [`crate::AssetServer`] can hand out copies without re-reading the
//! underlying file.

use glam::{Vec2, Vec3};

use crate::Asset;

/// CPU side mesh data produced by the glTF loader or a procedural
/// generator.
///
/// The renderer copies the arrays into GPU buffers; this struct is kept
/// around for editor previews and collision generation.
#[derive(Debug, Clone)]
pub struct MeshAsset {
    /// Display name of the mesh, used by the editor.
    pub name: String,
    /// Vertex positions, in object space.
    pub positions: Vec<[f32; 3]>,
    /// Vertex normals.
    pub normals: Vec<[f32; 3]>,
    /// Vertex UVs.
    pub uvs: Vec<[f32; 2]>,
    /// Vertex tangents. The fourth component encodes the handedness.
    pub tangents: Vec<[f32; 4]>,
    /// Triangle indices, three entries per triangle.
    pub indices: Vec<u32>,
}

impl MeshAsset {
    /// Builds an empty mesh with the supplied name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            tangents: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Returns the number of vertices in the mesh.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Returns the number of triangles in the mesh.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Recomputes smooth vertex normals from the current index buffer.
    pub fn compute_normals(&mut self) {
        self.normals.resize(self.positions.len(), [0.0, 0.0, 0.0]);
        for chunk in self.indices.chunks(3) {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;
            let p0: Vec3 = self.positions[i0].into();
            let p1: Vec3 = self.positions[i1].into();
            let p2: Vec3 = self.positions[i2].into();
            let n = (p1 - p0).cross(p2 - p0).normalize();
            self.normals[i0] = (Vec3::from(self.normals[i0]) + n).to_array();
            self.normals[i1] = (Vec3::from(self.normals[i1]) + n).to_array();
            self.normals[i2] = (Vec3::from(self.normals[i2]) + n).to_array();
        }
        for n in &mut self.normals {
            *n = Vec3::from(*n).normalize().to_array();
        }
    }

    /// Recomputes vertex tangents from the supplied UVs and indices.
    ///
    /// Falls back to a default tangent of `(1, 0, 0, 1)` when the mesh
    /// has no UV coordinates.
    pub fn compute_tangents(&mut self) {
        self.tangents.resize(self.positions.len(), [1.0, 0.0, 0.0, 1.0]);
        if self.uvs.is_empty() {
            return;
        }
        for chunk in self.indices.chunks(3) {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;
            let p0: Vec3 = self.positions[i0].into();
            let p1: Vec3 = self.positions[i1].into();
            let p2: Vec3 = self.positions[i2].into();
            let uv0: Vec2 = self.uvs[i0].into();
            let uv1: Vec2 = self.uvs[i1].into();
            let uv2: Vec2 = self.uvs[i2].into();
            let delta_p1 = p1 - p0;
            let delta_p2 = p2 - p0;
            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;
            let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
            let t = (delta_p1 * delta_uv2.y - delta_p2 * delta_uv1.y) * r;
            let t0 = Vec3::new(self.tangents[i0][0], self.tangents[i0][1], self.tangents[i0][2]);
            let t1 = Vec3::new(self.tangents[i1][0], self.tangents[i1][1], self.tangents[i1][2]);
            let t2 = Vec3::new(self.tangents[i2][0], self.tangents[i2][1], self.tangents[i2][2]);
            self.tangents[i0] = (t0 + t).normalize().extend(1.0).to_array();
            self.tangents[i1] = (t1 + t).normalize().extend(1.0).to_array();
            self.tangents[i2] = (t2 + t).normalize().extend(1.0).to_array();
        }
    }
}

impl Asset for MeshAsset {
    fn type_name() -> &'static str {
        "Mesh"
    }
}

/// CPU side texture data decoded from disk.
#[derive(Debug, Clone)]
pub struct TextureAsset {
    /// Image width, in pixels.
    pub width: u32,
    /// Image height, in pixels.
    pub height: u32,
    /// Raw pixel data, tightly packed.
    pub data: Vec<u8>,
    /// Number of channels per pixel (1, 2, 3 or 4).
    pub channels: u8,
}

impl Asset for TextureAsset {
    fn type_name() -> &'static str {
        "Texture"
    }
}

/// Material description stored as an asset.
#[derive(Debug, Clone)]
pub struct MaterialAsset {
    /// Display name of the material.
    pub name: String,
    /// Base color in linear space.
    pub base_color: [f32; 4],
    /// Metallic factor in the `[0, 1]` range.
    pub metallic: f32,
    /// Roughness factor in the `[0, 1]` range.
    pub roughness: f32,
}

impl Asset for MaterialAsset {
    fn type_name() -> &'static str {
        "Material"
    }
}
