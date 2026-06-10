//! Keyboard state tracking via egui input (since the editor
//! already routes events through egui-winit).
//!
//! For the runtime, a future implementation will use winit
//! events directly.

use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct KeyboardState {
    held: HashSet<egui::Key>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn is_held(&self, key: egui::Key) -> bool {
        self.held.contains(&key)
    }
    pub fn update_from_egui(&mut self, ctx: &egui::Context) {
        self.held.clear();
        ctx.input(|i| {
            for key in i.keys_down.iter() {
                self.held.insert(*key);
            }
        });
    }
}
