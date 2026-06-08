#![deny(unsafe_code)]

pub mod action;
pub mod event;
pub mod gamepad;
pub mod keyboard;
pub mod mouse;

use glam::Vec2;
use parking_lot::RwLock;
use winit::event::WindowEvent;

#[derive(Debug, Clone)]
pub struct InputState {
    pub cursor_position: Vec2,
    pub cursor_delta: Vec2,
    pub scroll_delta: Vec2,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            cursor_position: Vec2::ZERO,
            cursor_delta: Vec2::ZERO,
            scroll_delta: Vec2::ZERO,
        }
    }
}

pub struct InputSystem {
    state: RwLock<InputState>,
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(InputState::default()),
        }
    }

    pub fn process_window_event(&self, _event: &WindowEvent) {}

    pub fn state(&self) -> InputState {
        self.state.read().clone()
    }
}

impl Default for InputSystem {
    fn default() -> Self {
        Self::new()
    }
}
