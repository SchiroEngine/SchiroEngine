//! Entry point of the `schiro-editor` binary.
//!
//! Initializes the global tracing subscriber, constructs the
//! [`EditorApp`] and starts the winit event loop.

use schiro_core::profiling;
use schiro_editor::app::EditorApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    profiling::init();
    let app = EditorApp::new();
    app.run()
}
