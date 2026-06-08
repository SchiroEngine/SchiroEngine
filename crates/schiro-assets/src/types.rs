use glam::{Vec2, Vec3};

use crate::Asset;

#[derive(Debug, Clone)]
pub struct MeshAsset {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub tangents: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl MeshAsset {
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

    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

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

#[derive(Debug, Clone)]
pub struct TextureAsset {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub channels: u8,
}

impl Asset for TextureAsset {
    fn type_name() -> &'static str {
        "Texture"
    }
}

#[derive(Debug, Clone)]
pub struct MaterialAsset {
    pub name: String,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
}

impl Asset for MaterialAsset {
    fn type_name() -> &'static str {
        "Material"
    }
}
