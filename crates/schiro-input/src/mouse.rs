//! Mouse state tracking via egui input.

use glam::Vec2;

#[derive(Debug, Clone, Copy, Default)]
pub struct MouseState {
    pub position: Vec2,
    pub delta: Vec2,
    pub left: bool,
    pub middle: bool,
    pub right: bool,
    pub scroll: Vec2,
}

impl MouseState {
    pub fn update_from_egui(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if let Some(pos) = i.pointer.latest_pos() {
                let new = Vec2::new(pos.x, pos.y);
                self.delta = new - self.position;
                self.position = new;
            }
            self.left = i.pointer.primary_down();
            self.middle = i.pointer.middle_down();
            self.right = i.pointer.secondary_down();
            self.scroll = Vec2::new(i.raw_scroll_delta.x, i.raw_scroll_delta.y);
        });
    }
}
