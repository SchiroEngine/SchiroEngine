//! Re-export of the `egui_wgpu` renderer used by the editor.
//!
//! The renderer is instantiated in [`crate::Renderer::new`]; this
//! module exists so that downstream code can write
//! `schiro_render::egui_renderer::...` without depending on
//! `egui_wgpu` directly.
pub use egui_wgpu;
