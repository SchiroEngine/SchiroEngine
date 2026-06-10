//! Camera types shared between the renderer, editor and any
//! future headless tools.
//!
//! - [`OrbitCamera`] — CPU-side camera model used by the editor
//!   viewport (orbit around a target, yaw/pitch/distance).
//! - [`CameraUniform`] — GPU uniform block uploaded to wgpu.
//! - [`LightUniform`] — single directional light uniform block.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4};
use schiro_core::Ray;

// ---------------------------------------------------------------------------
// Orbit camera (CPU side)
// ---------------------------------------------------------------------------

/// Orbit camera that revolves around a target point.
#[derive(Debug, Clone)]
pub struct OrbitCamera {
    /// Point the camera looks at.
    pub target: Vec3,
    /// Distance from `target` to the camera, in meters.
    pub distance: f32,
    /// Horizontal rotation, in radians.
    pub yaw: f32,
    /// Vertical rotation, in radians. Clamped on mouse drag.
    pub pitch: f32,
    /// Vertical field of view, in radians.
    pub fov: f32,
    /// Near plane distance, in meters.
    pub near: f32,
    /// Far plane distance, in meters.
    pub far: f32,
    /// `true` while the user is dragging the camera.
    pub dragging: bool,
    /// Last pointer position seen by the camera input handler.
    pub last_mouse: (f32, f32),
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Vec3::new(0.0, 0.5, 0.0),
            distance: 5.0,
            yaw: 45.0_f32.to_radians(),
            pitch: 30.0_f32.to_radians(),
            fov: 60.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            dragging: false,
            last_mouse: (0.0, 0.0),
        }
    }
}

impl OrbitCamera {
    /// Returns the world space position of the camera.
    pub fn position(&self) -> Vec3 {
        let dir = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        );
        self.target + dir * self.distance
    }

    /// Computes the view matrix of this camera.
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position(), self.target, Vec3::Y)
    }

    /// Computes the perspective projection matrix of this camera.
    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov, aspect, self.near, self.far)
    }

    /// Builds the [`CameraUniform`] uploaded to the GPU every frame.
    pub fn to_uniform(&self, aspect: f32) -> CameraUniform {
        let view = self.view_matrix();
        let proj = self.projection_matrix(aspect);
        let mut uniform = CameraUniform::new();
        uniform.update(&view, &proj, self.position());
        uniform
    }

    /// Converts a screen space position into a world space [`Ray`].
    pub fn screen_to_ray(&self, screen_pos: Vec2, viewport_size: Vec2, aspect: f32) -> Ray {
        let view = self.view_matrix();
        let proj = self.projection_matrix(aspect);
        let inv_vp = (proj * view).inverse();

        let ndc_x = (2.0 * screen_pos.x) / viewport_size.x - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y) / viewport_size.y;

        let near_ndc = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let far_ndc = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

        let near_world = inv_vp * near_ndc;
        let far_world = inv_vp * far_ndc;

        let near = near_world.truncate() / near_world.w;
        let far = far_world.truncate() / far_world.w;

        Ray::new(near, (far - near).normalize())
    }

    /// Starts a drag at the supplied pointer position.
    pub fn on_mouse_press(&mut self, x: f32, y: f32) {
        self.dragging = true;
        self.last_mouse = (x, y);
    }

    /// Ends the current drag, if any.
    pub fn on_mouse_release(&mut self) {
        self.dragging = false;
    }

    /// Applies pointer drag deltas to the camera.
    pub fn on_mouse_drag(&mut self, x: f32, y: f32, button: MouseButton) {
        if !self.dragging {
            return;
        }
        let dx = x - self.last_mouse.0;
        let dy = y - self.last_mouse.1;
        self.last_mouse = (x, y);

        match button {
            MouseButton::Left => {
                self.yaw -= dx * 0.005;
                self.pitch = (self.pitch - dy * 0.005).clamp(-1.5, 1.5);
            }
            MouseButton::Middle => {
                let right = self.right();
                let up = self.up();
                self.target -= right * dx * 0.01 * self.distance;
                self.target += up * dy * 0.01 * self.distance;
            }
            MouseButton::Right => {}
        }
    }

    /// Applies a scroll wheel delta to the orbit distance.
    pub fn on_scroll(&mut self, delta: f32) {
        self.distance = (self.distance - delta * 0.3).clamp(0.5, 100.0);
    }

    /// Returns the world space right vector of the camera.
    fn right(&self) -> Vec3 {
        let forward = (self.target - self.position()).normalize();
        forward.cross(Vec3::Y).normalize()
    }

    /// Returns the world space up vector of the camera.
    fn up(&self) -> Vec3 {
        let forward = (self.target - self.position()).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        right.cross(forward).normalize()
    }
}

// ---------------------------------------------------------------------------
// Mouse button
// ---------------------------------------------------------------------------

/// Mouse button identifier used by the camera input handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Primary button, used for orbiting.
    Left,
    /// Middle button, used for panning.
    Middle,
    /// Secondary button, currently unused.
    Right,
}

// ---------------------------------------------------------------------------
// GPU uniforms (paired with the PBR shader)
// ---------------------------------------------------------------------------

/// Camera data uploaded to the GPU each frame.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CameraUniform {
    /// Combined view * projection matrix, in column major order.
    pub view_proj: [[f32; 4]; 4],
    /// View matrix, in column major order.
    pub view: [[f32; 4]; 4],
    /// Projection matrix, in column major order.
    pub proj: [[f32; 4]; 4],
    /// World space camera position, with `w = 1`.
    pub position: [f32; 4],
}

impl CameraUniform {
    /// Builds an identity camera uniform.
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            view: Mat4::IDENTITY.to_cols_array_2d(),
            proj: Mat4::IDENTITY.to_cols_array_2d(),
            position: [0.0, 0.0, 0.0, 1.0],
        }
    }

    /// Recomputes every field from the supplied matrices and camera
    /// world position.
    pub fn update(&mut self, view: &Mat4, proj: &Mat4, position: Vec3) {
        self.view = view.to_cols_array_2d();
        self.proj = proj.to_cols_array_2d();
        self.view_proj = (*proj * *view).to_cols_array_2d();
        self.position = [position.x, position.y, position.z, 1.0];
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}

/// Directional light data uploaded to the GPU.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct LightUniform {
    /// World space light direction, normalized. `w` is the intensity.
    pub direction: [f32; 4],
    /// Light color in linear space. `w` is unused and set to `1`.
    pub color: [f32; 4],
    /// Ambient color in linear space. `w` is unused and set to `1`.
    pub ambient: [f32; 4],
}

impl LightUniform {
    /// Builds a directional light with the given direction, color and
    /// ambient term.
    pub fn new(direction: Vec3, color: [f32; 3], intensity: f32, ambient: [f32; 3]) -> Self {
        let d = direction.normalize();
        Self {
            direction: [d.x, d.y, d.z, intensity],
            color: [color[0], color[1], color[2], 1.0],
            ambient: [ambient[0], ambient[1], ambient[2], 1.0],
        }
    }
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            direction: [-0.3, -0.8, -0.2, 1.0],
            color: [0.98, 0.95, 0.9, 1.0],
            ambient: [0.15, 0.15, 0.2, 1.0],
        }
    }
}
