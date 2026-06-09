//! Procedural mesh generators that produce [`MeshAsset`] values without
//! touching the file system.
//!
//! Useful for editor placeholders, debug helpers and tests.

use glam::Vec3;

use super::types::MeshAsset;

/// Builds a UV sphere.
///
/// `segments` controls the number of slices around the Y axis, `rings`
/// the number of stacks from pole to pole. Both counts must be greater
/// than zero; larger values yield smoother spheres.
pub fn create_sphere(radius: f32, segments: u32, rings: u32) -> MeshAsset {
    let mut mesh = MeshAsset::new("Sphere");

    for ring in 0..=rings {
        let phi = std::f32::consts::PI * ring as f32 / rings as f32;
        let y = phi.cos() * radius;
        let r = phi.sin() * radius;

        for seg in 0..=segments {
            let theta = 2.0 * std::f32::consts::PI * seg as f32 / segments as f32;
            let x = theta.cos() * r;
            let z = theta.sin() * r;

            mesh.positions.push([x, y, z]);
            mesh.normals.push(Vec3::new(x, y, z).normalize().to_array());
            mesh.uvs.push([seg as f32 / segments as f32, ring as f32 / rings as f32]);
        }
    }

    for ring in 0..rings {
        for seg in 0..segments {
            let a = ring * (segments + 1) + seg;
            let b = a + segments + 1;
            let c = a + 1;
            let d = b + 1;

            mesh.indices.push(a);
            mesh.indices.push(b);
            mesh.indices.push(c);

            mesh.indices.push(c);
            mesh.indices.push(b);
            mesh.indices.push(d);
        }
    }

    mesh
}

/// Builds a cylinder centered on the origin, aligned with the Y axis.
///
/// The cylinder has no caps: it is meant to be combined with additional
/// geometry for the top and bottom faces when those are needed.
pub fn create_cylinder(radius: f32, height: f32, segments: u32) -> MeshAsset {
    let mut mesh = MeshAsset::new("Cylinder");
    let half_h = height * 0.5;

    for seg in 0..=segments {
        let theta = 2.0 * std::f32::consts::PI * seg as f32 / segments as f32;
        let x = theta.cos() * radius;
        let z = theta.sin() * radius;

        mesh.positions.push([x, -half_h, z]);
        mesh.positions.push([x, half_h, z]);
        mesh.normals.push([x / radius, 0.0, z / radius]);
        mesh.normals.push([x / radius, 0.0, z / radius]);
        mesh.uvs.push([seg as f32 / segments as f32, 0.0]);
        mesh.uvs.push([seg as f32 / segments as f32, 1.0]);
    }

    for seg in 0..segments {
        let a = seg * 2;
        let b = a + 2;
        let c = a + 1;
        let d = a + 3;

        mesh.indices.push(a);
        mesh.indices.push(b);
        mesh.indices.push(c);
        mesh.indices.push(c);
        mesh.indices.push(b);
        mesh.indices.push(d);
    }

    mesh
}
