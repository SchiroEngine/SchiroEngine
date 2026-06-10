//! Undo / redo command stack and entity duplication.
//!
//! Every user mutation that touches the ECS world records a
//! [`Command`] containing the full before/after state of the
//! affected component.  Undo swaps the state back, redo swaps
//! it forward.  The stack is stored on [`crate::app::EditorApp`].

use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};

use crate::app::EditorApp;

/// A single editable mutation tracked by the undo system.
#[derive(Debug, Clone)]
pub enum Command {
    SetTransform { entity: Entity, before: (Vec3, Quat, Vec3), after: (Vec3, Quat, Vec3) },
    SetName { entity: Entity, before: String, after: String },
    ToggleRotator { entity: Entity, added: bool, speed: Vec3 },
}

impl Command {
    pub fn revert(&self, world: &mut World) {
        match self {
            Command::SetTransform { entity, before, .. } => {
                if let Some(mut t) = world.get_mut::<schiro_ecs::components::Transform>(*entity) {
                    t.translation = before.0;
                    t.rotation = before.1;
                    t.scale = before.2;
                }
            }
            Command::SetName { entity, before, .. } => {
                if let Some(mut n) = world.get_mut::<schiro_ecs::components::Name>(*entity) {
                    n.0.clone_from(before);
                }
            }
            Command::ToggleRotator { entity, added, speed } => {
                let mut em = world.entity_mut(*entity);
                if *added {
                    em.remove::<schiro_ecs::components::Rotator>();
                } else {
                    em.insert(schiro_ecs::components::Rotator { speed: *speed });
                }
            }
        }
    }

    pub fn apply(&self, world: &mut World) {
        match self {
            Command::SetTransform { entity, after, .. } => {
                if let Some(mut t) = world.get_mut::<schiro_ecs::components::Transform>(*entity) {
                    t.translation = after.0;
                    t.rotation = after.1;
                    t.scale = after.2;
                }
            }
            Command::SetName { entity, after, .. } => {
                if let Some(mut n) = world.get_mut::<schiro_ecs::components::Name>(*entity) {
                    n.0.clone_from(after);
                }
            }
            Command::ToggleRotator { entity, added, .. } => {
                let mut em = world.entity_mut(*entity);
                if *added {
                    em.insert(schiro_ecs::components::Rotator { speed: Vec3::ONE });
                } else {
                    em.remove::<schiro_ecs::components::Rotator>();
                }
            }
        }
    }
}

impl EditorApp {
    pub fn push_command(&mut self, cmd: Command) {
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
        if self.undo_stack.len() > 512 {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) {
        if let Some(cmd) = self.undo_stack.pop() {
            cmd.revert(&mut self.world);
            self.redo_stack.push(cmd);
        }
    }

    pub fn redo(&mut self) {
        if let Some(cmd) = self.redo_stack.pop() {
            cmd.apply(&mut self.world);
            self.undo_stack.push(cmd);
        }
    }

    /// Duplicates the selected entity: spawns a fresh entity of
    /// the same type (via add_mesh_entity or add_empty) with
    /// the same Transform offset by `(1, 0, 0)`.
    pub fn duplicate_entity(&mut self) {
        let Some(entity) = self.selected_entity else { return };
        let name = self.get_entity_name(entity);
        let t = self.get_entity_transform(entity);

        let offset = t.translation + Vec3::new(1.0, 0.0, 0.0);

        if name.contains("Cube") {
            self.add_mesh_entity("Cube", &schiro_render::Mesh::cube(), offset, None);
        } else if name.contains("Sphere") {
            let mesh = render_to_mesh(&schiro_assets::procedural::create_sphere(1.0, 32, 16));
            let speed = self.world.get::<schiro_ecs::components::Rotator>(entity).map(|r| r.speed);
            self.add_mesh_entity("Sphere", &mesh, offset, speed);
        } else if name.contains("Plane") {
            self.add_mesh_entity("Plane", &schiro_render::Mesh::plane(), offset, None);
        } else {
            self.add_empty(&format!("{} (copy)", name), offset);
        }
    }
}

fn render_to_mesh(asset: &schiro_assets::types::MeshAsset) -> schiro_render::Mesh {
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
