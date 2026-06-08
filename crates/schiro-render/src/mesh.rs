use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wgpu::util::DeviceExt;

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
}

impl Vertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32x2,
            3 => Float32x4,
        ],
    };
}

pub struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub model_buffer: wgpu::Buffer,
    pub model_bind_group: wgpu::BindGroup,
}

impl GpuMesh {
    pub fn new(
        device: &wgpu::Device,
        mesh: &Mesh,
        transform: &Mat4,
        model_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("VB: {}", mesh.name)),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("IB: {}", mesh.name)),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let normal_matrix = transform.inverse().transpose();
        let model_uniform = ModelUniform {
            model: transform.to_cols_array_2d(),
            normal_matrix: normal_matrix.to_cols_array_2d(),
        };

        let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Model: {}", mesh.name)),
            contents: bytemuck::bytes_of(&model_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Model BG: {}", mesh.name)),
            layout: model_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: model_buffer.as_entire_binding(),
            }],
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
            model_buffer,
            model_bind_group,
        }
    }

    pub fn update_transform(&self, queue: &wgpu::Queue, transform: &Mat4) {
        let normal_matrix = transform.inverse().transpose();
        let model_uniform = ModelUniform {
            model: transform.to_cols_array_2d(),
            normal_matrix: normal_matrix.to_cols_array_2d(),
        };
        queue.write_buffer(&self.model_buffer, 0, bytemuck::bytes_of(&model_uniform));
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(1, &self.model_bind_group, &[]);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ModelUniform {
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub name: String,
}

impl Mesh {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            name: name.into(),
        }
    }

pub fn cube() -> Self {
    Self::cube_at_y(0.0)
}

pub fn cube_at_y(y_offset: f32) -> Self {
    let mut mesh = Self::new("Cube");
    let half = 0.5_f32;
        let positions = [
            // Front
            [-half, -half, half],
            [half, -half, half],
            [half, half, half],
            [-half, half, half],
            // Back
            [-half, -half, -half],
            [half, -half, -half],
            [half, half, -half],
            [-half, half, -half],
        ];
        let normals = [
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
        ];
        let faces = [
            (0, 1, 2, 0, 2, 3, 0), // Front  +Z (norm = +Z ✓)
            (4, 6, 5, 4, 7, 6, 1), // Back   -Z (norm = -Z ✓)
            (1, 5, 6, 1, 6, 2, 2), // Right  +X (norm = +X ✓)
            (4, 0, 3, 4, 3, 7, 3), // Left   -X (norm = -X ✓)
            (2, 6, 7, 2, 7, 3, 4), // Top    +Y (norm = +Y ✓)
            (1, 4, 5, 1, 0, 4, 5), // Bottom -Y (norm = -Y ✓)
        ];
        let uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        for (i0, i1, i2, i3, _i4, _i5, ni) in faces {
            let n = normals[ni];
            let t = tangent_from_normal(n);
            for (idx, uv_idx) in [(i0, 0usize), (i1, 1), (i2, 2), (i1, 1), (i2, 2), (i3, 3)] {
                mesh.vertices.push(Vertex {
                    position: positions[idx],
                    normal: n,
                    uv: uvs[uv_idx],
                    tangent: t,
                });
            }
        }
        mesh.indices = (0..36).collect();

        for v in &mut mesh.vertices {
            v.position[1] += y_offset;
        }
        mesh
    }

    pub fn grid(rows: u32, cols: u32, spacing: f32) -> Self {
        let mut mesh = Self::new("Grid");
        let half_w = cols as f32 * spacing * 0.5;
        let half_h = rows as f32 * spacing * 0.5;
        let n = [0.0, 1.0, 0.0];
        let t = tangent_from_normal(n);

        for r in 0..=rows {
            for c in 0..=cols {
                let x = c as f32 * spacing - half_w;
                let z = r as f32 * spacing - half_h;
                mesh.vertices.push(Vertex {
                    position: [x, 0.0, z],
                    normal: n,
                    uv: [
                        c as f32 / cols as f32,
                        r as f32 / rows as f32,
                    ],
                    tangent: t,
                });
            }
        }
        for r in 0..rows {
            for c in 0..cols {
                let i = r * (cols + 1) + c;
                mesh.indices.push(i);
                mesh.indices.push(i + cols + 1);
                mesh.indices.push(i + 1);

                mesh.indices.push(i + 1);
                mesh.indices.push(i + cols + 1);
                mesh.indices.push(i + cols + 2);
            }
        }
        mesh
    }
}

fn tangent_from_normal(normal: [f32; 3]) -> [f32; 4] {
    let n = glam::Vec3::from_array(normal);
    let t = if n.x.abs() > 0.9 {
        glam::Vec3::new(0.0, n.z, -n.y).normalize()
    } else {
        glam::Vec3::new(n.y, -n.x, 0.0).normalize()
    };
    let b = n.cross(t);
    let handedness = if n.cross(t).dot(b) < 0.0 { -1.0 } else { 1.0 };
    [t.x, t.y, t.z, handedness]
}
