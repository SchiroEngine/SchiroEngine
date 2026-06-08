use glam::{Mat4, Vec2, Vec3, Vec4};
use schiro_core::Ray;
use schiro_render::camera::CameraUniform;

#[derive(Debug, Clone)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub dragging: bool,
    last_mouse: (f32, f32),
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
    pub fn position(&self) -> Vec3 {
        let dir = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        );
        self.target + dir * self.distance
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position(), self.target, Vec3::Y)
    }

    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov, aspect, self.near, self.far)
    }

    pub fn to_uniform(&self, aspect: f32) -> CameraUniform {
        let view = self.view_matrix();
        let proj = self.projection_matrix(aspect);
        let mut uniform = CameraUniform::new();
        uniform.update(&view, &proj, self.position());
        uniform
    }

    pub fn screen_to_ray(
        &self,
        screen_pos: Vec2,
        viewport_size: Vec2,
        aspect: f32,
    ) -> Ray {
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

    pub fn on_mouse_press(&mut self, x: f32, y: f32) {
        self.dragging = true;
        self.last_mouse = (x, y);
    }

    pub fn on_mouse_release(&mut self) {
        self.dragging = false;
    }

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
            MouseButton::Right => {
                // Could also orbit or do something else
            }
        }
    }

    pub fn on_scroll(&mut self, delta: f32) {
        self.distance = (self.distance - delta * 0.3).clamp(0.5, 100.0);
    }

    fn right(&self) -> Vec3 {
        let forward = (self.target - self.position()).normalize();
        forward.cross(Vec3::Y).normalize()
    }

    fn up(&self) -> Vec3 {
        let forward = (self.target - self.position()).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        right.cross(forward).normalize()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

pub struct ViewportPanel {
    pub camera: OrbitCamera,
    pub rect: egui::Rect,
    pub focused: bool,
    pub clicked_pos: Option<(f32, f32)>,
    pub mouse_delta: (f32, f32),
    pub gizmo_hovered: bool,
    pub gizmo_held: bool,
    pub gizmo_press_pos: Option<(f32, f32)>,
    pub keys_pressed: Vec<egui::Key>,
    is_dragging: bool,
    press_pos: (f32, f32),
    pub prev_mouse: (f32, f32),
}

impl ViewportPanel {
    pub fn new() -> Self {
        Self {
            camera: OrbitCamera::default(),
            rect: egui::Rect::ZERO,
            focused: false,
            clicked_pos: None,
            mouse_delta: (0.0, 0.0),
            gizmo_hovered: false,
            gizmo_held: false,
            gizmo_press_pos: None,
            keys_pressed: Vec::new(),
            is_dragging: false,
            press_pos: (0.0, 0.0),
            prev_mouse: (0.0, 0.0),
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        viewport_texture_id: Option<egui::TextureId>,
        viewport_size: (u32, u32),
    ) -> egui::Response {
        let available = ui.available_size();
        let aspect = available.x / available.y.max(1.0);

        let (tex_id, tex_size) = if let Some(id) = viewport_texture_id {
            (id, egui::Vec2::new(viewport_size.0 as f32, viewport_size.1 as f32))
        } else {
            (egui::TextureId::default(), available)
        };

        let response = egui::Frame::dark_canvas(ui.style())
            .show(ui, |ui| {
                let rect = ui.max_rect();
                self.rect = rect;

                let image = egui::Image::new(egui::ImageSource::Texture(
                    egui::load::SizedTexture::new(tex_id, tex_size),
                ))
                .fit_to_exact_size(rect.size());

                ui.put(rect, image);

                if ui.rect_contains_pointer(rect) {
                    self.handle_input(ui, aspect);
                }
            })
            .response;

        self.focused = response.hovered();

        response
    }

    fn handle_input(&mut self, ui: &egui::Ui, _aspect: f32) {
        let input = ui.input(|i| i.clone());
        let pointer = &input.pointer;

        self.clicked_pos = None;
        self.mouse_delta = (0.0, 0.0);
        self.keys_pressed.clear();
        for key in [egui::Key::W, egui::Key::E, egui::Key::R] {
            if input.key_pressed(key) {
                self.keys_pressed.push(key);
            }
        }

        if let Some(pos) = pointer.interact_pos() {
            let local = pos - self.rect.min.to_vec2();

            self.mouse_delta = (local.x - self.prev_mouse.0, local.y - self.prev_mouse.1);
            self.prev_mouse = (local.x, local.y);

            if pointer.primary_pressed() {
                if self.gizmo_hovered {
                    self.gizmo_held = true;
                    self.gizmo_press_pos = Some((local.x, local.y));
                } else {
                    self.camera.on_mouse_press(local.x, local.y);
                }
                self.press_pos = (local.x, local.y);
                self.is_dragging = false;
            }

            if pointer.primary_released() {
                if self.gizmo_held {
                    self.gizmo_held = false;
                    self.gizmo_press_pos = None;
                } else {
                    if !self.is_dragging && self.camera.dragging {
                        self.clicked_pos = Some((local.x, local.y));
                    }
                    self.camera.on_mouse_release();
                }
            }

            if self.gizmo_held {
                let dx = (self.press_pos.0 - local.x).abs();
                let dy = (self.press_pos.1 - local.y).abs();
                if dx > 2.0 || dy > 2.0 {
                    self.is_dragging = true;
                }
            } else if self.camera.dragging {
                let dx = (self.press_pos.0 - local.x).abs();
                let dy = (self.press_pos.1 - local.y).abs();
                if dx > 2.0 || dy > 2.0 {
                    self.is_dragging = true;
                }

                let button = if pointer.primary_down() {
                    MouseButton::Left
                } else if pointer.middle_down() {
                    MouseButton::Middle
                } else if pointer.secondary_down() {
                    MouseButton::Right
                } else {
                    return;
                };
                self.camera.on_mouse_drag(local.x, local.y, button);
            }
        }

        let scroll = input.raw_scroll_delta;
        if scroll.y != 0.0 {
            self.camera.on_scroll(scroll.y * 0.15);
        }
    }
}

impl Default for ViewportPanel {
    fn default() -> Self {
        Self::new()
    }
}
