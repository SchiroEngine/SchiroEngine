//! Menu bar and toolbar.

use crate::app::{EditorApp, EditorTool};

impl EditorApp {
    /// Builds the top menu bar (File, Edit) and the toolbar with the
    /// gizmo tool buttons and the Play/Stop button.
    pub fn build_menu_bar(&mut self, _ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar")
            .frame(
                egui::Frame::none()
                    .fill(crate::theme::panel_header_bg())
                    .inner_margin(egui::vec2(8.0, 1.0)),
            )
            .show(_ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New Scene").clicked() {
                            self.clear_scene();
                            ui.close_menu();
                        }
                        if ui.button("Open Scene").clicked() {
                            self.load_scene("scene.srn-scene").ok();
                            ui.close_menu();
                        }
                        if ui.button("Save Scene").clicked() {
                            self.save_scene("scene.srn-scene").ok();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Exit").clicked() {
                            std::process::exit(0);
                        }
                    });
                    ui.menu_button("Edit", |ui| {
                        if ui.button("Undo").clicked() {}
                        if ui.button("Redo").clicked() {}
                    });
                });
            });

        self.build_toolbar(_ctx);
    }

    /// Builds the toolbar with the gizmo tool buttons and the
    /// Play/Stop button.
    fn build_toolbar(&mut self, _ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::none()
                    .fill(crate::theme::panel_header_bg())
                    .inner_margin(egui::vec2(8.0, 1.0)),
            )
            .show_separator_line(false)
            .show(_ctx, |ui| {
                ui.horizontal(|ui| {
                    self.draw_tool_button(ui, "\u{2194}", "Translate", EditorTool::Translate);
                    self.draw_tool_button(ui, "\u{21BB}", "Rotate", EditorTool::Rotate);
                    self.draw_tool_button(ui, "\u{25A1}", "Scale", EditorTool::Scale);
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    let (pr, resp) =
                        ui.allocate_exact_size(egui::vec2(64.0, 24.0), egui::Sense::click());
                    if ui.is_rect_visible(pr) {
                        let fill = if self.playing {
                            egui::Color32::from_rgb(0xCC, 0x44, 0x44)
                        } else {
                            egui::Color32::from_rgb(0x3A, 0x8C, 0x4A)
                        };
                        ui.painter().rect(
                            pr,
                            egui::CornerRadius::same(4),
                            fill,
                            egui::Stroke::NONE,
                            egui::StrokeKind::Inside,
                        );
                        let icon = if self.playing { "\u{25A0}" } else { "\u{25B6}" };
                        let label = if self.playing { " Stop" } else { " Play" };
                        ui.painter().text(
                            pr.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{}{}", icon, label),
                            egui::FontId::proportional(13.0),
                            egui::Color32::WHITE,
                        );
                    }
                    if resp.clicked() {
                        self.playing = !self.playing;
                    }
                });
            });
    }

    /// Draws a single toolbar button for a gizmo tool.
    ///
    /// `icon` is the Unicode glyph to display, `label` the tooltip
    /// text and `tool` the tool the button activates.
    pub fn draw_tool_button(
        &mut self,
        ui: &mut egui::Ui,
        icon: &str,
        label: &str,
        tool: EditorTool,
    ) {
        use EditorTool::*;
        let selected = self.current_tool == tool;
        let (rect, response) = ui.allocate_exact_size(egui::vec2(32.0, 22.0), egui::Sense::click());
        if ui.is_rect_visible(rect) {
            let fill = if selected {
                crate::theme::accent_color()
            } else if response.hovered() {
                ui.style().visuals.widgets.hovered.bg_fill
            } else {
                ui.style().visuals.widgets.inactive.bg_fill
            };
            ui.painter().rect(
                rect,
                egui::CornerRadius::same(4),
                fill,
                egui::Stroke::NONE,
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(14.0),
                if selected { crate::theme::text_bright() } else { crate::theme::text_dim() },
            );
        }
        if response.clicked() {
            self.current_tool = tool;
        }
        response.on_hover_text(format!(
            "{} [{}]",
            label,
            match tool {
                Translate => "W",
                Rotate => "E",
                Scale => "R",
            }
        ));
    }
}
