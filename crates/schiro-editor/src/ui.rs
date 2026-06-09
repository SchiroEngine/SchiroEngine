//! Top level UI orchestration.
//!
//! The free function [`build_editor_ui`] is the entry point used by
//! the egui run callback. It delegates to the per-panel builders
//! defined in [`crate::panels`].

use crate::app::EditorApp;

/// Builds a complete editor frame: menu bar, hierarchy, inspector,
/// viewport and status bar.
pub fn build_editor_ui(app: &mut EditorApp, ctx: &egui::Context) {
    app.build_editor_ui(ctx);
}
