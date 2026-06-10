//! Status bar: shows engine version, play state and active tool.

use crate::app::{EditorApp, EditorTool};

impl EditorApp {
    pub fn build_status_bar(&mut self, ctx: &egui::Context) {
        let state = if self.playing { "PLAYING" } else { "EDIT" };
        let tool = match self.current_tool {
            EditorTool::Translate => "Translate [W]",
            EditorTool::Rotate => "Rotate [E]",
            EditorTool::Scale => "Scale [R]",
        };

        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::new()
                    .fill(crate::theme::panel_header_bg())
                    .inner_margin(egui::vec2(12.0, 3.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("SchiroEngine  |  {state}  |  {tool}"))
                            .color(crate::theme::text_dim())
                            .size(10.5),
                    );
                });
            });
    }
}
