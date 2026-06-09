//! Hierarchy panel: lists every scene entity and lets the user select
//! one by clicking on it.

use crate::app::EditorApp;

impl EditorApp {
    /// Builds the left-hand hierarchy panel.
    pub fn build_hierarchy_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("hierarchy_panel")
            .resizable(true)
            .default_width(260.0)
            .min_width(180.0)
            .frame(
                egui::Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::vec2(8.0, 6.0)),
            )
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("Scene Hierarchy")
                        .color(crate::theme::text_bright())
                        .size(13.0)
                        .strong(),
                );
                ui.add_space(6.0);
                egui::ScrollArea::vertical().id_salt("h_scroll").show(ui, |ui| {
                    if self.scene_entities.is_empty() {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("  (empty scene)")
                                .color(crate::theme::text_dim())
                                .size(12.0),
                        );
                        return;
                    }
                    for &entity in &self.scene_entities {
                        let name = self.get_entity_name(entity);
                        let selected = self.selected_entity == Some(entity);
                        let icon = if name.contains("Sphere") {
                            "\u{25C9}"
                        } else if name.contains("Grid") {
                            "\u{25A6}"
                        } else {
                            "\u{25A0}"
                        };
                        let (rect, resp) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 24.0),
                            egui::Sense::click(),
                        );
                        if ui.is_rect_visible(rect) {
                            let vis = if selected {
                                let mut v = ui.style().visuals.widgets.active.clone();
                                v.bg_fill = crate::theme::faint_bg_color();
                                v.bg_stroke =
                                    egui::Stroke::new(1.0_f32, crate::theme::accent_color());
                                v
                            } else if resp.hovered() {
                                ui.style().visuals.widgets.hovered.clone()
                            } else {
                                ui.style().visuals.widgets.inactive.clone()
                            };
                            ui.painter().rect(
                                rect.shrink(1.0),
                                egui::CornerRadius::same(4),
                                vis.bg_fill,
                                vis.bg_stroke,
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().text(
                                rect.left_center() + egui::vec2(12.0, 0.0),
                                egui::Align2::LEFT_CENTER,
                                format!("{}  {}", icon, name),
                                egui::FontId::monospace(12.5),
                                vis.fg_stroke.color,
                            );
                        }
                        if resp.clicked() {
                            self.selected_entity = Some(entity);
                        }
                    }
                });
            });
    }
}
