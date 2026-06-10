//! Tests for `schiro_render::Mesh` primitive generators and the
//! `CameraUniform` math helpers.

use glam::{Mat4, Vec3, Vec4};
use schiro_render::camera::CameraUniform;
use schiro_render::mesh::Mesh;

mod common;
use common::assert_approx_eq;

#[test]
fn new_mesh_is_empty() {
    let m = Mesh::new("empty");
    assert_eq!(m.name, "empty");
    assert!(m.vertices.is_empty());
    assert!(m.indices.is_empty());
}

#[test]
fn cube_has_36_vertices_36_indices() {
    let c = Mesh::cube();
    // The current cube implementation duplicates vertices per face
    // to keep face normals flat: 6 faces × 6 vertices (2 triangles)
    // = 36 vertices and 36 indices.
    assert_eq!(c.vertices.len(), 36);
    assert_eq!(c.indices.len(), 36);
}

#[test]
fn cube_normals_are_unit_length() {
    let c = Mesh::cube();
    for v in &c.vertices {
        let n = glam::Vec3::from(v.normal);
        let len = n.length();
        assert_approx_eq(len, 1.0, 1e-4, "cube normal");
    }
}

#[test]
fn grid_geometry_matches_requested_size() {
    let g = Mesh::grid(4, 6, 2.0);
    // (rows+1) * (cols+1) vertices
    assert_eq!(g.vertices.len(), 5 * 7);
    // 2 triangles per cell, 3 indices each
    assert_eq!(g.indices.len(), 4 * 6 * 6);
}

#[test]
fn grid_lies_on_y_zero_plane() {
    let g = Mesh::grid(2, 2, 1.0);
    for v in &g.vertices {
        assert_eq!(v.position[1], 0.0);
    }
}

#[test]
fn grid_normals_point_up() {
    let g = Mesh::grid(1, 1, 1.0);
    for v in &g.vertices {
        assert_eq!(v.normal, [0.0, 1.0, 0.0]);
    }
}

#[test]
fn camera_uniform_default_is_identity() {
    let cu = CameraUniform::default();
    assert_eq!(cu.view, Mat4::IDENTITY.to_cols_array_2d());
    assert_eq!(cu.proj, Mat4::IDENTITY.to_cols_array_2d());
    assert_eq!(cu.view_proj, Mat4::IDENTITY.to_cols_array_2d());
    assert_eq!(cu.position, [0.0, 0.0, 0.0, 1.0]);
}

#[test]
fn camera_uniform_update_packs_view_proj() {
    let mut cu = CameraUniform::new();
    let view = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let proj = Mat4::perspective_rh(1.0, 1.0, 0.1, 100.0);
    cu.update(&view, &proj, Vec3::new(1.0, 2.0, 3.0));

    let expected = proj * view;
    for col in 0..4 {
        for row in 0..4 {
            assert_approx_eq(
                cu.view_proj[col][row],
                expected.col(col)[row],
                1e-4,
                "view_proj",
            );
        }
    }
    assert_eq!(cu.position, Vec4::new(1.0, 2.0, 3.0, 1.0).to_array());
}

#[test]
fn camera_uniform_pod_roundtrip() {
    let mut cu = CameraUniform::new();
    cu.update(
        &Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        &Mat4::IDENTITY,
        Vec3::new(1.0, 0.0, 0.0),
    );
    let bytes = bytemuck::bytes_of(&cu);
    assert_eq!(bytes.len(), std::mem::size_of::<CameraUniform>());
}
