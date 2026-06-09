//! Editor UI panels.
//!
//! Each submodule defines a single egui panel. The top level
//! [`EditorApp::build_editor_ui`] orchestrator lives here and is the
//! single entry point used by the egui run callback.

pub mod hierarchy;
pub mod inspector;
pub mod menu_bar;
pub mod status_bar;

use crate::app::EditorApp;

impl EditorApp {
    /// Builds the entire editor UI for the current frame: menu bar,
    /// hierarchy, inspector, status bar and the central viewport.
    pub fn build_editor_ui(&mut self, ctx: &egui::Context) {
        self.build_menu_bar(ctx);
        self.build_hierarchy_panel(ctx);
        self.build_inspector_panel(ctx);
        self.build_status_bar(ctx);

        self.build_viewport(ctx);
    }

    /// Builds the central viewport panel that displays the rendered
    /// scene and handles camera / gizmo input.
    fn build_viewport(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(0)))
            .show(ctx, |ui| {
                let vf = egui::Frame::none()
                    .fill(crate::theme::faint_bg_color())
                    .stroke(egui::Stroke::new(1.0_f32, crate::theme::faint_bg_color()))
                    .inner_margin(egui::Margin::same(0));
                vf.show(ui, |ui| {
                    let tex_id = self.renderer.as_ref()
                        .and_then(|r| r.viewport.as_ref())
                        .and_then(|vp| vp.egui_texture_id);
                    let vp_size = self.renderer.as_ref()
                        .and_then(|r| r.viewport.as_ref())
                        .map(|vp| vp.size).unwrap_or((1280, 720));
                    if tex_id.is_some() {
                        self.viewport_panel.show(ui, tex_id, vp_size);
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("Viewport")
                                .color(crate::theme::text_dim()).size(16.0));
                        });
                    }
                });
            });
    }
}
