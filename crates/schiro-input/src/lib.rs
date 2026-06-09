//! Cross platform input handling.
//!
//! The crate centralises the translation of [`winit::event::WindowEvent`]
//! into the high level [`InputState`] used by the rest of the engine.
//! The keyboard, mouse, gamepad and action-mapping modules are
//! currently stubs and will be filled in as the runtime grows.

#![deny(unsafe_code)]

pub mod action;
pub mod event;
pub mod gamepad;
pub mod keyboard;
pub mod mouse;

use glam::Vec2;
use parking_lot::RwLock;
use winit::event::WindowEvent;

/// Snapshot of the current pointer and scroll state.
///
/// The struct is updated once per window event by [`InputSystem`] and
/// read freely by the rest of the engine.
#[derive(Debug, Clone)]
pub struct InputState {
    /// Cursor position in window space, in points.
    pub cursor_position: Vec2,
    /// Cursor movement since the previous frame, in points.
    pub cursor_delta: Vec2,
    /// Scroll wheel delta accumulated since the previous frame.
    pub scroll_delta: Vec2,
}

impl Default for InputState {
    fn default() -> Self {
        Self { cursor_position: Vec2::ZERO, cursor_delta: Vec2::ZERO, scroll_delta: Vec2::ZERO }
    }
}

/// Receives [`WindowEvent`]s from the host application and exposes the
/// resulting [`InputState`].
pub struct InputSystem {
    state: RwLock<InputState>,
}

impl InputSystem {
    /// Builds a new input system with an empty state.
    pub fn new() -> Self {
        Self { state: RwLock::new(InputState::default()) }
    }

    /// Feeds a window event into the system. The current implementation
    /// is a no-op: the matching handlers will be wired up as the
    /// individual sub-modules are filled in.
    pub fn process_window_event(&self, _event: &WindowEvent) {}

    /// Returns a snapshot of the current input state.
    pub fn state(&self) -> InputState {
        self.state.read().clone()
    }
}

impl Default for InputSystem {
    fn default() -> Self {
        Self::new()
    }
}
