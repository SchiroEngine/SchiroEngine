//! glTF 2.0 importer built on top of the [`gltf`] crate.
//!
//! Walks the document and produces a [`MeshAsset`] per primitive. The
//! function intentionally ignores cameras, animations and skins: the
//! runtime currently consumes geometry and materials only.

use std::path::Path;

use crate::loader::AssetLoadError;
use crate::types::MeshAsset;

/// Loads every mesh found in the glTF file at `path`.
///
/// Returns an [`AssetLoadError::Gltf`] if the document cannot be parsed,
/// or [`AssetLoadError::Parse`] when the file contains no meshes.
pub fn load_gltf(path: &Path) -> Result<Vec<MeshAsset>, AssetLoadError> {
    let (document, buffers, _images) =
        gltf::import(path).map_err(|e| AssetLoadError::Gltf(e.to_string()))?;

    let mut meshes = Vec::new();

    for mesh in document.meshes() {
        let name = mesh.name().unwrap_or("Unnamed").to_string();

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .map(|p| p.collect::<Vec<_>>())
                .unwrap_or_default()
                .into_iter()
                .map(|p| [p[0], p[1], p[2]])
                .collect();

            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|n| n.collect::<Vec<_>>())
                .unwrap_or_default()
                .into_iter()
                .map(|n| [n[0], n[1], n[2]])
                .collect();

            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|uv| uv.into_f32().collect::<Vec<_>>())
                .unwrap_or_default()
                .into_iter()
                .map(|uv| [uv[0], uv[1]])
                .collect();

            let indices: Vec<u32> =
                reader.read_indices().map(|i| i.into_u32().collect::<Vec<_>>()).unwrap_or_default();

            if positions.is_empty() {
                continue;
            }

            let mut asset = MeshAsset::new(&name);
            asset.positions = positions;
            asset.normals = normals;
            asset.uvs = uvs;
            asset.indices = indices;

            if asset.normals.is_empty() {
                asset.compute_normals();
            }
            if asset.uvs.len() == asset.positions.len() && asset.tangents.is_empty() {
                asset.compute_tangents();
            }

            meshes.push(asset);
        }
    }

    if meshes.is_empty() {
        return Err(AssetLoadError::Parse("no meshes found in glTF file".into()));
    }

    Ok(meshes)
}
