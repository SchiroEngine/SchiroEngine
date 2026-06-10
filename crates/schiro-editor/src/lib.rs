//! Editor application for the SchiroEngine.
//!
//! The editor is a binary crate that wires together every other crate
//! of the workspace into a runnable application. It owns the egui
//! context, the wgpu renderer, the bevy_ecs world, the asset server,
//! the input and physics systems, and exposes them through a set of
//! panels (hierarchy, inspector, status bar, viewport).
//!
//! # Layout
//!
//! - [`app`] — the `EditorApp` type and its `ApplicationHandler`
//!   implementation.
//! - [`viewport`] — the 3D viewport widget and the orbit camera.
//! - [`panels`] — egui panels (menu bar, hierarchy, inspector, status
//!   bar).
//! - [`editor`] — scene initialization, gizmo logic, project state.
//! - [`project`] — project metadata (name, path).
//! - [`theme`] — colors and egui theme helpers.
//! - [`browser`] — asset browser panel.
//! - [`inspector`] — read-only inspector used when no entity is
//!   selected.

pub mod app;
pub mod browser;
pub mod editor;
pub mod hierarchy;
pub mod inspector;
pub mod panels;
pub mod project;
pub mod scene;
pub mod theme;
pub mod ui;
pub mod viewport;
