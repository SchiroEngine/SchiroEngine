use schiro_core::profiling;
use schiro_editor::app::EditorApp;
use tracing::info;

fn main() {
    profiling::init();
    info!("SchiroEngine Editor starting");

    let app = EditorApp::new();
    if let Err(e) = app.run() {
        eprintln!("fatal: {e}");
        std::process::exit(1);
    }
}
