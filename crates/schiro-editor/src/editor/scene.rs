//! Default scene setup used when the editor boots up.

use std::collections::HashMap;

use bevy_ecs::prelude::*;
use glam::{Mat4, Vec3};
use schiro_assets::types::MeshAsset;
use schiro_ecs::components::{MeshRenderer, Rotator, Transform};
use schiro_ecs::World;
use schiro_render::Renderer;
use tracing::info;

use crate::app::{EditorTool, GizmoDrag};

/// Spawns the default scene entities (a sphere and a grid), uploads
/// their meshes to the renderer, and pre-allocates the gizmo meshes.
///
/// On return, `entities`, `mesh_map` and `gizmo_start` are populated
/// and the renderer's mesh list contains every scene mesh followed by
/// the 12 gizmo meshes.
pub fn init_scene(
    world: &mut World,
    renderer: &mut Renderer,
    asset_server: &schiro_assets::AssetServer,
    entities: &mut Vec<Entity>,
    mesh_map: &mut HashMap<Entity, usize>,
    gizmo_start: &mut usize,
) {
    let gizmo = schiro_render::GizmoMeshes::new();
    *gizmo_start = renderer.mesh_count();
    let hide = Mat4::from_scale(Vec3::ZERO);
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

    let sphere_asset = schiro_assets::procedural::create_sphere(1.0, 32, 16);
    let sphere = asset_server.load("proc://sphere", |_| Ok(sphere_asset.clone())).unwrap();
    let sm = asset_to_render_mesh(&sphere);
    let t = Mat4::from_translation(Vec3::new(0.0, 1.5, 0.0));
    let mi = renderer.mesh_count();
    renderer.add_mesh(&sm, &t);

    let entity = world
        .spawn((
            schiro_ecs::components::Name("Sphere".into()),
            Transform { translation: Vec3::new(0.0, 1.5, 0.0), ..Default::default() },
            schiro_ecs::components::GlobalTransform::default(),
            MeshRenderer { mesh_handle: Some(mi), visible: true },
            Rotator { speed: Vec3::new(0.0, 1.5, 0.0) },
        ))
        .id();
    entities.push(entity);
    mesh_map.insert(entity, mi);

    let gm = schiro_render::Mesh::grid(10, 10, 1.0);
    let gi = renderer.mesh_count();
    renderer.add_mesh(&gm, &Mat4::IDENTITY);
    let ge = world
        .spawn((
            schiro_ecs::components::Name("Grid".into()),
            Transform::default(),
            schiro_ecs::components::GlobalTransform::default(),
            MeshRenderer { mesh_handle: Some(gi), visible: true },
        ))
        .id();
    entities.push(ge);
    mesh_map.insert(ge, gi);

    info!("scene: {} entities", entities.len());
}

/// Updates the model matrices of the 12 gizmo meshes so that the
/// gizmos for the active tool are positioned at the selected entity,
/// while the others are hidden by setting their scale to zero.
pub fn update_gizmo_transforms(
    renderer: &mut Renderer,
    world: &World,
    selected: Option<Entity>,
    gizmo_start: usize,
    tool: EditorTool,
) {
    let hide = Mat4::from_scale(Vec3::ZERO);
    if let Some(entity) = selected {
        let pos = world.get::<Transform>(entity).map(|t| t.translation).unwrap_or(Vec3::ZERO);
        let t = Mat4::from_translation(pos);
        let (tr, rr, sr) = match tool {
            EditorTool::Translate => (0..6, 6..6, 9..9),
            EditorTool::Rotate => (0..0, 6..9, 9..9),
            EditorTool::Scale => (0..0, 6..6, 9..12),
        };
        for i in 0..12 {
            let idx = gizmo_start + i;
            let show = (i >= tr.start && i < tr.end)
                || (i >= rr.start && i < rr.end)
                || (i >= sr.start && i < sr.end);
            renderer.update_mesh_transform(idx, if show { &t } else { &hide });
        }
    } else {
        for i in 0..12 {
            renderer.update_mesh_transform(gizmo_start + i, &hide);
        }
    }
}

/// Converts a CPU side [`MeshAsset`] into the renderer's
/// [`schiro_render::Mesh`].
fn asset_to_render_mesh(asset: &MeshAsset) -> schiro_render::Mesh {
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
