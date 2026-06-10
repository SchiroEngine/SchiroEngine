//! Cross-platform input handling powered by egui.

#![deny(unsafe_code)]

pub mod action;
pub mod event;
pub mod gamepad;
pub mod keyboard;
pub mod mouse;

use parking_lot::RwLock;

pub struct InputSystem {
    keyboard: RwLock<keyboard::KeyboardState>,
    mouse: RwLock<mouse::MouseState>,
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            keyboard: RwLock::new(keyboard::KeyboardState::new()),
            mouse: RwLock::new(mouse::MouseState::default()),
        }
    }

    pub fn update_from_egui(&self, ctx: &egui::Context) {
        self.keyboard.write().update_from_egui(ctx);
        self.mouse.write().update_from_egui(ctx);
    }

    pub fn mouse(&self) -> mouse::MouseState {
        *self.mouse.read()
    }
    pub fn is_key_held(&self, key: egui::Key) -> bool {
        self.keyboard.read().is_held(key)
    }

    /// Backward-compatible stub — the editor still calls this.
    /// Prefer [`Self::update_from_egui`].
    pub fn process_window_event(&self, _event: &winit::event::WindowEvent) {}
}

impl Default for InputSystem {
    fn default() -> Self {
        Self::new()
    }
}
