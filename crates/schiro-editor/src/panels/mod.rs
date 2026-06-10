//! Editor UI panels.
//!
//! Each submodule defines a single egui panel. The top-level
//! [`EditorApp::build_editor_ui`] orchestrator lives here and is the
//! single entry point used by the egui run callback.

pub mod hierarchy;
pub mod inspector;
pub mod menu_bar;
pub mod status_bar;

use crate::app::EditorApp;

impl EditorApp {
    pub fn build_editor_ui(&mut self, ctx: &egui::Context) {
        self.build_menu_bar(ctx);
        self.build_hierarchy_panel(ctx);
        self.build_inspector_panel(ctx);
        self.build_status_bar(ctx);
        self.build_viewport(ctx);
    }

    fn build_viewport(&mut self, ctx: &egui::Context) {
        let border = crate::theme::border();
        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(&ctx.style())
                    .inner_margin(egui::Margin::same(0))
                    .fill(crate::theme::darkest()),
            )
            .show(ctx, |ui| {
                let tex_id = self
                    .renderer
                    .as_ref()
                    .and_then(|r| r.viewport.as_ref())
                    .and_then(|vp| vp.egui_texture_id);
                let vp_size = self
                    .renderer
                    .as_ref()
                    .and_then(|r| r.viewport.as_ref())
                    .map(|vp| vp.size)
                    .unwrap_or((1280, 720));

                let rect = ui.max_rect();
                ui.painter().rect_stroke(
                    rect,
                    egui::CornerRadius::ZERO,
                    egui::Stroke::new(1.0, border),
                    egui::StrokeKind::Inside,
                );

                let inner = egui::Frame::new()
                    .fill(crate::theme::darkest())
                    .inner_margin(egui::Margin::same(1));
                inner.show(ui, |ui| {
                    if tex_id.is_some() {
                        self.viewport_panel.show(ui, tex_id, vp_size);
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                egui::RichText::new("Viewport")
                                    .color(crate::theme::text_dim())
                                    .size(16.0),
                            );
                        });
                    }
                });
            });
    }
}
