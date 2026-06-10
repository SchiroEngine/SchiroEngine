//! 3D viewport widget.
//!
//! Wraps the [`schiro_camera::OrbitCamera`] with egui pointer event
//! handling to display the rendered scene and forward user input to
//! the gizmo / picking pipeline.

use glam::{Vec2, Vec3};
use schiro_camera::{MouseButton, OrbitCamera};

/// Embedded 3D viewport widget.
pub struct ViewportPanel {
    /// Orbit camera used to render the scene.
    pub camera: OrbitCamera,
    /// Rectangle occupied by the viewport inside the egui layout, in
    /// egui coordinates.
    pub rect: egui::Rect,
    /// `true` when the cursor is over the viewport.
    pub focused: bool,
    /// Click position consumed by
    /// [`crate::editor::gizmo::handle_viewport_input`] to drive
    /// entity picking.
    pub clicked_pos: Option<(f32, f32)>,
    /// Pointer delta accumulated since the previous frame.
    pub mouse_delta: (f32, f32),
    /// `true` when the cursor is currently over a gizmo handle.
    pub gizmo_hovered: bool,
    /// `true` while the user is dragging a gizmo handle.
    pub gizmo_held: bool,
    /// Pointer position at which the gizmo drag started.
    pub gizmo_press_pos: Option<(f32, f32)>,
    /// Keys pressed since the previous frame.
    pub keys_pressed: Vec<egui::Key>,
    is_dragging: bool,
    press_pos: (f32, f32),
    /// Pointer position at the end of the previous frame.
    pub prev_mouse: (f32, f32),
}

impl ViewportPanel {
    /// Builds a viewport with the default camera and an empty
    /// rectangle.
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

    /// Draws the viewport inside `ui` and processes pointer events.
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

    /// Captures pointer, scroll and key events from the supplied egui
    /// `Ui` and updates the internal state of the viewport.
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
