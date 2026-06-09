//! Gizmo meshes used by the editor to translate, rotate and scale
//! entities.
//!
//! The meshes are precomputed once and cloned for every selected
//! entity. They are hidden by setting their model matrix to a zero
//! scale, which is cheaper than re-uploading a fresh mesh.

use glam::Vec3;

use crate::mesh::{Mesh, Vertex};

/// Bundle of gizmo meshes, in the order expected by the editor.
///
/// Index layout (12 meshes total):
/// - `0..6` : translate shafts and tips, one per axis.
/// - `6..9` : rotation rings, one per axis.
/// - `9..12`: scale handles, one per axis.
pub struct GizmoMeshes {
    /// Red shaft of the translate gizmo, along +X.
    pub x_shaft: Mesh,
    /// Red tip cone of the translate gizmo, along +X.
    pub x_tip: Mesh,
    /// Green shaft of the translate gizmo, along +Y.
    pub y_shaft: Mesh,
    /// Green tip cone of the translate gizmo, along +Y.
    pub y_tip: Mesh,
    /// Blue shaft of the translate gizmo, along +Z.
    pub z_shaft: Mesh,
    /// Blue tip cone of the translate gizmo, along +Z.
    pub z_tip: Mesh,
    /// Red rotation ring, around the X axis.
    pub rot_x: Mesh,
    /// Green rotation ring, around the Y axis.
    pub rot_y: Mesh,
    /// Blue rotation ring, around the Z axis.
    pub rot_z: Mesh,
    /// Red scale handle, along +X.
    pub scale_x: Mesh,
    /// Green scale handle, along +Y.
    pub scale_y: Mesh,
    /// Blue scale handle, along +Z.
    pub scale_z: Mesh,
}

impl GizmoMeshes {
    /// Builds the default set of gizmo meshes.
    pub fn new() -> Self {
        let shaft_len = 0.8;
        let shaft_half = 0.03;
        let tip_len = 0.2;
        let tip_half = 0.06;

        Self {
            x_shaft: axis_box([1.0, 0.0, 0.0], shaft_len, shaft_half),
            x_tip: axis_pyramid([1.0, 0.0, 0.0], shaft_len, tip_len, tip_half),
            y_shaft: axis_box([0.0, 1.0, 0.0], shaft_len, shaft_half),
            y_tip: axis_pyramid([0.0, 1.0, 0.0], shaft_len, tip_len, tip_half),
            z_shaft: axis_box([0.0, 0.0, 1.0], shaft_len, shaft_half),
            z_tip: axis_pyramid([0.0, 0.0, 1.0], shaft_len, tip_len, tip_half),
            rot_x: rotation_ring(1.0, [1.0, 0.0, 0.0], 64),
            rot_y: rotation_ring(1.0, [0.0, 1.0, 0.0], 64),
            rot_z: rotation_ring(1.0, [0.0, 0.0, 1.0], 64),
            scale_x: scale_handle([1.0, 0.0, 0.0], 0.9, 0.06),
            scale_y: scale_handle([0.0, 1.0, 0.0], 0.9, 0.06),
            scale_z: scale_handle([0.0, 0.0, 1.0], 0.9, 0.06),
        }
    }
}

/// Builds a flat ring of given `radius` and `normal`, split into
/// `segments` quads.
fn rotation_ring(radius: f32, normal: [f32; 3], segments: u32) -> Mesh {
    let mut mesh = Mesh::new("rot_ring");
    let n = Vec3::from_array(normal).normalize();
    let perp1 =
        if n.x.abs() < 0.9 { Vec3::X.cross(n).normalize() } else { Vec3::Y.cross(n).normalize() };
    let perp2 = n.cross(perp1);
    let thickness = 0.03;

    for i in 0..=segments {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
        let center = perp1 * angle.cos() * radius + perp2 * angle.sin() * radius;
        let inner = center - n * thickness;
        let outer = center + n * thickness;
        mesh.vertices.push(Vertex {
            position: inner.to_array(),
            normal,
            uv: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        mesh.vertices.push(Vertex {
            position: outer.to_array(),
            normal,
            uv: [1.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
    }

    for i in 0..segments {
        let a = i * 2;
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

/// Builds a small box used as a scale handle at `distance` along the
/// given axis.
fn scale_handle(axis: [f32; 3], distance: f32, half: f32) -> Mesh {
    let d = Vec3::from_array(axis).normalize();
    let center = d * distance;
    let mut mesh = Mesh::new("scale_handle");

    let perp1 =
        if d.x.abs() < 0.9 { Vec3::X.cross(d).normalize() } else { Vec3::Y.cross(d).normalize() };
    let perp2 = d.cross(perp1);

    let corners = [
        center + perp1 * half + perp2 * half,
        center + perp1 * half - perp2 * half,
        center - perp1 * half - perp2 * half,
        center - perp1 * half + perp2 * half,
        center + perp1 * half + perp2 * half + d * half * 2.0,
        center + perp1 * half - perp2 * half + d * half * 2.0,
        center - perp1 * half - perp2 * half + d * half * 2.0,
        center - perp1 * half + perp2 * half + d * half * 2.0,
    ];

    let faces =
        [(0, 1, 2, 3), (4, 7, 6, 5), (0, 4, 5, 1), (1, 5, 6, 2), (2, 6, 7, 3), (3, 7, 4, 0)];

    for &(a, b, c, d) in &faces {
        let n = compute_normal(&corners[a], &corners[b], &corners[c]);
        for &idx in &[a, b, c, a, c, d] {
            mesh.vertices.push(Vertex {
                position: corners[idx].to_array(),
                normal: n,
                uv: [0.0, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });
        }
    }

    mesh.indices = (0..36).collect();
    mesh
}

/// Builds a box aligned with `dir` that extends from the origin to
/// `length` along that axis.
fn axis_box(dir: [f32; 3], length: f32, half_width: f32) -> Mesh {
    let mut mesh = Mesh::new("gizmo_shaft");
    let d = Vec3::from_array(dir);
    let perp1 = if d.x.abs() < 0.9 {
        Vec3::new(1.0, 0.0, 0.0).cross(d).normalize()
    } else {
        Vec3::new(0.0, 1.0, 0.0).cross(d).normalize()
    };
    let perp2 = d.cross(perp1).normalize();

    let v = [
        d * length + perp1 * half_width + perp2 * half_width,
        d * length + perp1 * half_width - perp2 * half_width,
        d * length - perp1 * half_width - perp2 * half_width,
        d * length - perp1 * half_width + perp2 * half_width,
        perp1 * half_width + perp2 * half_width,
        perp1 * half_width - perp2 * half_width,
        -perp1 * half_width - perp2 * half_width,
        -perp1 * half_width + perp2 * half_width,
    ];

    let faces = [(4, 5, 1, 4, 1, 0), (5, 6, 2, 5, 2, 1), (6, 7, 3, 6, 3, 2), (7, 4, 0, 7, 0, 3)];

    for &(a, b, c, d, e, f) in &faces {
        let n = compute_normal(&v[a], &v[b], &v[c]);
        mesh.vertices.push(Vertex {
            position: v[a].to_array(),
            normal: n,
            uv: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        mesh.vertices.push(Vertex {
            position: v[b].to_array(),
            normal: n,
            uv: [1.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        mesh.vertices.push(Vertex {
            position: v[c].to_array(),
            normal: n,
            uv: [1.0, 1.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        mesh.vertices.push(Vertex {
            position: v[d].to_array(),
            normal: n,
            uv: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        mesh.vertices.push(Vertex {
            position: v[e].to_array(),
            normal: n,
            uv: [1.0, 1.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        mesh.vertices.push(Vertex {
            position: v[f].to_array(),
            normal: n,
            uv: [0.0, 1.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
    }
    mesh.indices = (0..24).collect();
    mesh
}

/// Builds a cone aligned with `dir` starting at `start` and extending
/// `length` units.
fn axis_pyramid(dir: [f32; 3], start: f32, length: f32, half_width: f32) -> Mesh {
    let mut mesh = Mesh::new("gizmo_tip");
    let d = Vec3::from_array(dir);
    let perp1 = if d.x.abs() < 0.9 {
        Vec3::new(1.0, 0.0, 0.0).cross(d).normalize()
    } else {
        Vec3::new(0.0, 1.0, 0.0).cross(d).normalize()
    };
    let perp2 = d.cross(perp1).normalize();

    let tip = d * (start + length);
    let base_center = d * start;
    let base = [
        base_center + perp1 * half_width + perp2 * half_width,
        base_center + perp1 * half_width - perp2 * half_width,
        base_center - perp1 * half_width - perp2 * half_width,
        base_center - perp1 * half_width + perp2 * half_width,
    ];

    for i in 0..4 {
        let j = (i + 1) % 4;
        let n = compute_normal(&base[i], &base[j], &tip);
        mesh.vertices.push(Vertex {
            position: base[i].to_array(),
            normal: n,
            uv: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        mesh.vertices.push(Vertex {
            position: base[j].to_array(),
            normal: n,
            uv: [1.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        mesh.vertices.push(Vertex {
            position: tip.to_array(),
            normal: n,
            uv: [0.5, 1.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
    }

    let base_normal = -d;
    for i in (0..4).rev() {
        let j = if i == 0 { 3 } else { i - 1 };
        mesh.vertices.push(Vertex {
            position: base[i].to_array(),
            normal: base_normal.to_array(),
            uv: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        mesh.vertices.push(Vertex {
            position: base[j].to_array(),
            normal: base_normal.to_array(),
            uv: [1.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        mesh.vertices.push(Vertex {
            position: base_center.to_array(),
            normal: base_normal.to_array(),
            uv: [0.5, 0.5],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
    }

    mesh.indices = (0..24).collect();
    mesh
}

/// Computes the unit normal of the triangle `(a, b, c)`.
fn compute_normal(a: &Vec3, b: &Vec3, c: &Vec3) -> [f32; 3] {
    let ab = *b - *a;
    let ac = *c - *a;
    ab.cross(ac).normalize().to_array()
}
