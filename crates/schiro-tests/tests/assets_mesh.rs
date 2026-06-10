//! Tests for `schiro_assets::MeshAsset` and the procedural generators.

use schiro_assets::procedural::{create_cylinder, create_sphere};
use schiro_assets::types::MeshAsset;

mod common;
use common::assert_approx_eq;

const EPS: f32 = 1e-3;

#[test]
fn new_mesh_is_empty() {
    let m = MeshAsset::new("Box");
    assert_eq!(m.name, "Box");
    assert_eq!(m.vertex_count(), 0);
    assert_eq!(m.triangle_count(), 0);
    assert!(m.positions.is_empty());
    assert!(m.indices.is_empty());
}

#[test]
fn compute_normals_on_empty_does_not_panic() {
    let mut m = MeshAsset::new("empty");
    m.compute_normals();
    assert!(m.normals.is_empty());
}

#[test]
fn compute_normals_produces_unit_vectors() {
    // A cube has no degenerate normals: every triangle contributes
    // a non-zero cross product, so the result is always well-defined.
    let mut m = schiro_assets::types::MeshAsset::new("cube");
    m.positions = vec![
        [-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.5, 0.5, 0.0], [-0.5, 0.5, 0.0],
        [-0.5, -0.5, 1.0], [0.5, -0.5, 1.0], [0.5, 0.5, 1.0], [-0.5, 0.5, 1.0],
    ];
    m.indices = vec![
        0, 1, 2, 0, 2, 3, // front
        4, 6, 5, 4, 7, 6, // back
        0, 4, 5, 0, 5, 1, // bottom
        2, 6, 7, 2, 7, 3, // top
        0, 3, 7, 0, 7, 4, // left
        1, 5, 6, 1, 6, 2, // right
    ];
    m.compute_normals();
    for n in &m.normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!(len.is_finite(), "normal must be finite, got {n:?}");
        assert_approx_eq(len, 1.0, 1e-4, "normal length");
    }
}

#[test]
fn compute_tangents_without_uvs_returns_default() {
    let mut m = MeshAsset::new("no_uv");
    m.positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    m.indices = vec![0, 1, 2];
    m.compute_tangents();
    assert_eq!(m.tangents, vec![[1.0, 0.0, 0.0, 1.0]; 3]);
}

#[test]
fn sphere_has_expected_counts() {
    let m = create_sphere(1.0, 8, 4);
    // vertices = (segments + 1) * (rings + 1) = 9 * 5
    assert_eq!(m.vertex_count(), 9 * 5);
    // indices = segments * rings * 6 (two triangles per quad)
    assert_eq!(m.indices.len(), 8 * 4 * 6);
}

#[test]
fn sphere_normals_point_outward() {
    let m = create_sphere(1.0, 16, 8);
    for (i, p) in m.positions.iter().enumerate() {
        let n = m.normals[i];
        let p = glam::Vec3::from(*p);
        let n = glam::Vec3::from(n);
        let dot = p.dot(n);
        assert!(dot > 0.0, "normal at vertex {i} does not point outward: {dot}");
    }
}

#[test]
fn cylinder_has_two_vertices_per_segment() {
    let m = create_cylinder(1.0, 2.0, 8);
    // 8 segments + 1 column, 2 verts per column = 18
    assert_eq!(m.vertex_count(), 9 * 2);
    assert_eq!(m.indices.len(), 8 * 6);
}

#[test]
fn cylinder_radius_matches_along_height() {
    let r = 2.5_f32;
    let m = create_cylinder(r, 1.0, 16);
    for p in &m.positions {
        let xz = (p[0] * p[0] + p[2] * p[2]).sqrt();
        assert_approx_eq(xz, r, 0.01, "radius");
    }
}
