//! Status bar: shows the current play state and the active gizmo
//! tool.

use crate::app::{EditorApp, EditorTool};

impl EditorApp {
    /// Builds the bottom status bar.
    pub fn build_status_bar(&mut self, _ctx: &egui::Context) {
        let state = if self.playing { "PLAYING" } else { "EDIT" };
        let tool_name = match self.current_tool {
            EditorTool::Translate => "Translate [W]",
            EditorTool::Rotate => "Rotate [E]",
            EditorTool::Scale => "Scale [R]",
        };
        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::none()
                    .fill(crate::theme::panel_header_bg())
                    .inner_margin(egui::vec2(12.0, 2.0)),
            )
            .show(_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "SchiroEngine v0.1  |  {}  |  {}",
                            state, tool_name
                        ))
                        .color(crate::theme::text_dim())
                        .size(11.0),
                    );
                });
            });
    }
}
