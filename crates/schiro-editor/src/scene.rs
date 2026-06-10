//! Scene save/load logic backed by [`schiro_scene`] data types.

use schiro_scene::{EntityDesc, MeshDesc, SceneFile};

use crate::app::EditorApp;

const CURRENT_VERSION: u32 = 1;

impl EditorApp {
    /// Saves the current scene to `path` as a JSON file.
    pub fn save_scene(&self, path: impl AsRef<std::path::Path>) -> Result<(), SaveError> {
        let path = path.as_ref();
        let mut entities = Vec::new();

        for &e in &self.scene_entities {
            let name = self.get_entity_name(e);
            let t = self.get_entity_transform(e);
            let rotator = self
                .world
                .get::<schiro_ecs::components::Rotator>(e)
                .map(|r| [r.speed.x, r.speed.y, r.speed.z]);

            // Determine the mesh kind from the entity name (current
            // convention — a future AABB-based heuristic will be
            // more robust).
            let has_renderer = self.world.get::<schiro_ecs::components::MeshRenderer>(e).is_some();
            let mesh = if has_renderer {
                if name.contains("Sphere") {
                    Some(MeshDesc::Sphere { segments: 32, rings: 16 })
                } else if name.contains("Grid") {
                    Some(MeshDesc::Grid { rows: 10, cols: 10, spacing: 1.0 })
                } else if name.contains("Cube") {
                    Some(MeshDesc::Cube)
                } else if name.contains("Plane") {
                    Some(MeshDesc::Plane)
                } else {
                    Some(MeshDesc::Sphere { segments: 16, rings: 8 })
                }
            } else {
                None
            };
            let _ = has_renderer;

            entities.push(EntityDesc {
                name,
                translation: t.translation.into(),
                rotation: [t.rotation.x, t.rotation.y, t.rotation.z, t.rotation.w],
                scale: t.scale.into(),
                rotator,
                mesh,
            });
        }

        let file = SceneFile { version: CURRENT_VERSION, entities };
        let json =
            serde_json::to_string_pretty(&file).map_err(|e| SaveError::Json(e.to_string()))?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Loads a scene from `path`, clears the current editor scene
    /// and recreates every entity with the saved components and
    /// meshes.
    pub fn load_scene(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), LoadError> {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path)?;
        let file: SceneFile =
            serde_json::from_str(&json).map_err(|e| LoadError::Json(e.to_string()))?;

        if file.version != CURRENT_VERSION {
            return Err(LoadError::Version { expected: CURRENT_VERSION, found: file.version });
        }

        // Wipe the old scene.
        self.clear_scene();

        for desc in &file.entities {
            let transform = glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::from(desc.scale),
                glam::Quat::from_array(desc.rotation),
                glam::Vec3::from(desc.translation),
            );
            let mi = if let Some(ref mesh_desc) = desc.mesh {
                let renderer = self.renderer.as_mut().ok_or(LoadError::NoRenderer)?;
                let mesh_data = mesh_desc_to_render(mesh_desc);
                let idx = renderer.mesh_count();
                renderer.add_mesh(&mesh_data, &transform);
                Some(idx)
            } else {
                None
            };

            let mut entity_cmd = self.world.spawn((
                schiro_ecs::components::Name(desc.name.clone()),
                schiro_ecs::components::Transform {
                    translation: desc.translation.into(),
                    rotation: glam::Quat::from_array(desc.rotation),
                    scale: desc.scale.into(),
                },
                schiro_ecs::components::GlobalTransform::default(),
            ));
            if let Some(mi) = mi {
                entity_cmd.insert(schiro_ecs::components::MeshRenderer {
                    mesh_handle: Some(mi),
                    visible: true,
                });
            }
            if let Some(speed) = desc.rotator {
                entity_cmd
                    .insert(schiro_ecs::components::Rotator { speed: glam::Vec3::from(speed) });
            }
            let entity = entity_cmd.id();
            self.scene_entities.push(entity);
            if let Some(mi) = mi {
                self.entity_mesh_map.insert(entity, mi);
            }
        }

        // The gizmo mesh count is recomputed after clearing in
        // clear_scene.  Re-upload gizmo meshes.
        {
            let renderer = self.renderer.as_mut().ok_or(LoadError::NoRenderer)?;
            let gizmo = schiro_render::GizmoMeshes::new();
            self.gizmo_mesh_start = renderer.mesh_count();
            let hide = glam::Mat4::from_scale(glam::Vec3::ZERO);
            for part in [
                &gizmo.x_shaft,
                &gizmo.x_tip,
                &gizmo.y_shaft,
                &gizmo.y_tip,
                &gizmo.z_shaft,
                &gizmo.z_tip,
                &gizmo.rot_x,
                &gizmo.rot_y,
                &gizmo.rot_z,
                &gizmo.scale_x,
                &gizmo.scale_y,
                &gizmo.scale_z,
            ] {
                renderer.add_mesh(part, &hide);
            }
        }

        tracing::info!(
            "loaded scene from {} with {} entities",
            path.display(),
            file.entities.len()
        );
        Ok(())
    }

    /// Builds a [`SceneFile`] from the current editor state (used
    /// for the Play snapshot).
    pub fn scene_as_file(&self) -> schiro_scene::SceneFile {
        let mut entities = Vec::new();
        for &e in &self.scene_entities {
            let name = self.get_entity_name(e);
            let t = self.get_entity_transform(e);
            let rotator = self
                .world
                .get::<schiro_ecs::components::Rotator>(e)
                .map(|r| [r.speed.x, r.speed.y, r.speed.z]);
            let has_renderer = self.world.get::<schiro_ecs::components::MeshRenderer>(e).is_some();
            let mesh = if has_renderer {
                if name.contains("Sphere") {
                    Some(MeshDesc::Sphere { segments: 32, rings: 16 })
                } else if name.contains("Grid") {
                    Some(MeshDesc::Grid { rows: 10, cols: 10, spacing: 1.0 })
                } else if name.contains("Cube") {
                    Some(MeshDesc::Cube)
                } else if name.contains("Plane") {
                    Some(MeshDesc::Plane)
                } else {
                    Some(MeshDesc::Sphere { segments: 16, rings: 8 })
                }
            } else {
                None
            };
            entities.push(EntityDesc {
                name,
                translation: t.translation.into(),
                rotation: [t.rotation.x, t.rotation.y, t.rotation.z, t.rotation.w],
                scale: t.scale.into(),
                rotator,
                mesh,
            });
            let _ = has_renderer;
        }
        schiro_scene::SceneFile { version: CURRENT_VERSION, entities }
    }

    /// Clears every scene entity, the mesh map and the renderer's
    /// mesh list.
    pub fn clear_scene(&mut self) {
        for &e in &self.scene_entities {
            self.world.entity_mut(e).despawn();
        }
        self.scene_entities.clear();
        self.entity_mesh_map.clear();
        self.selected_entity = None;
        self.gizmo_drag = None;
        if let Some(ref mut r) = self.renderer {
            r.meshes.clear();
        }
    }
}

pub fn mesh_desc_to_render(desc: &MeshDesc) -> schiro_render::Mesh {
    match *desc {
        MeshDesc::Sphere { segments, rings } => {
            let asset = schiro_assets::procedural::create_sphere(1.0, segments, rings);
            asset_to_render_mesh(&asset)
        }
        MeshDesc::Grid { rows, cols, spacing } => schiro_render::Mesh::grid(rows, cols, spacing),
        MeshDesc::Cube => schiro_render::Mesh::cube(),
        MeshDesc::Plane => schiro_render::Mesh::plane(),
    }
}

fn asset_to_render_mesh(asset: &schiro_assets::types::MeshAsset) -> schiro_render::Mesh {
    let mut mesh = schiro_render::Mesh::new(&asset.name);
    for i in 0..asset.positions.len() {
        let tangent =
            if i < asset.tangents.len() { asset.tangents[i] } else { [1.0, 0.0, 0.0, 1.0] };
        mesh.vertices.push(schiro_render::mesh::Vertex {
            position: asset.positions[i],
            normal: if i < asset.normals.len() { asset.normals[i] } else { [0.0, 1.0, 0.0] },
            uv: if i < asset.uvs.len() { asset.uvs[i] } else { [0.0, 0.0] },
            tangent,
        });
    }
    mesh.indices = asset.indices.clone();
    mesh
}

/// Errors returned by [`EditorApp::save_scene`].
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON encoding error: {0}")]
    Json(String),
}

/// Errors returned by [`EditorApp::load_scene`].
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON decoding error: {0}")]
    Json(String),
    #[error("unsupported scene version {found} (expected {expected})")]
    Version { expected: u32, found: u32 },
    #[error("no renderer available")]
    NoRenderer,
}
