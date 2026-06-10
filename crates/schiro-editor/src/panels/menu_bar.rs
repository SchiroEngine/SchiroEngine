//! Menu bar and toolbar (Blender-style).

use crate::app::{EditorApp, EditorTool};

impl EditorApp {
    pub fn build_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar")
            .frame(
                egui::Frame::new()
                    .fill(crate::theme::panel_header_bg())
                    .inner_margin(egui::vec2(8.0, 1.0)),
            )
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New Scene").clicked() {
                            self.clear_scene();
                            self.project.path = std::path::PathBuf::new();
                            ui.close_menu();
                        }
                        if ui.button("Open Scene").clicked() {
                            ui.close_menu();
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("SchiroEngine scene", &["srn-scene", "srn"])
                                .pick_file()
                            {
                                if let Err(e) = self.load_scene(&path) {
                                    tracing::error!("load failed: {e}");
                                } else {
                                    self.project.path = path;
                                }
                            }
                        }
                        if ui.button("Save Scene").clicked() {
                            ui.close_menu();
                            let path = if self.project.path.as_os_str().is_empty() {
                                rfd::FileDialog::new()
                                    .add_filter("SchiroEngine scene", &["srn-scene"])
                                    .save_file()
                            } else {
                                Some(self.project.path.clone())
                            };
                            if let Some(ref p) = path {
                                if let Err(e) = self.save_scene(p) {
                                    tracing::error!("save failed: {e}");
                                } else {
                                    self.project.path = p.clone();
                                }
                            }
                        }
                        ui.separator();
                        if ui.button("Exit").clicked() {
                            std::process::exit(0);
                        }
                    });
                    ui.menu_button("Edit", |ui| {
                        if ui.button("Undo  [Ctrl+Z]").clicked() {
                            self.undo();
                            ui.close_menu();
                        }
                        if ui.button("Redo  [Ctrl+Y]").clicked() {
                            self.redo();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Duplicate  [Ctrl+D]").clicked() {
                            self.duplicate_entity();
                            ui.close_menu();
                        }
                    });
                });
            });

        self.build_toolbar(ctx);
    }

    fn build_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::new()
                    .fill(crate::theme::panel_header_bg())
                    .inner_margin(egui::vec2(8.0, 1.0)),
            )
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    self.draw_tool_button(ui, "\u{2194}", "Translate", EditorTool::Translate);
                    self.draw_tool_button(ui, "\u{21BB}", "Rotate", EditorTool::Rotate);
                    self.draw_tool_button(ui, "\u{25A1}", "Scale", EditorTool::Scale);
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    let (pr, resp) =
                        ui.allocate_exact_size(egui::vec2(64.0, 22.0), egui::Sense::click());
                    if ui.is_rect_visible(pr) {
                        let fill = if self.playing {
                            Color32::from_rgb(0xB0, 0x30, 0x30)
                        } else {
                            Color32::from_rgb(0x2D, 0x6E, 0x3A)
                        };
                        ui.painter().rect_filled(pr, egui::CornerRadius::same(3), fill);
                        let icon = if self.playing { "\u{25A0}" } else { "\u{25B6}" };
                        let lbl = if self.playing { " Stop" } else { " Play" };
                        ui.painter().text(
                            pr.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{}{}", icon, lbl),
                            egui::FontId::proportional(12.0),
                            Color32::WHITE,
                        );
                    }
                    if resp.clicked() {
                        self.playing = !self.playing;
                    }
                });
            });
    }

    pub fn draw_tool_button(
        &mut self,
        ui: &mut egui::Ui,
        icon: &str,
        label: &str,
        tool: EditorTool,
    ) {
        use EditorTool::*;
        let selected = self.current_tool == tool;
        let (rect, response) = ui.allocate_exact_size(egui::vec2(30.0, 20.0), egui::Sense::click());
        if ui.is_rect_visible(rect) {
            let fill = if selected {
                crate::theme::accent_color()
            } else if response.hovered() {
                crate::theme::hover()
            } else {
                Color32::TRANSPARENT
            };
            if fill != Color32::TRANSPARENT {
                ui.painter().rect_filled(rect, egui::CornerRadius::same(3), fill);
            }
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(14.0),
                if selected { Color32::WHITE } else { crate::theme::text_dim() },
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

use egui::Color32;
