//! Project metadata tracked by the editor.

use std::path::PathBuf;

/// Name and on disk location of the project currently being edited.
pub struct Project {
    /// Display name of the project, shown in the title bar.
    pub name: String,
    /// Filesystem path of the project file. Empty when the project has
    /// not yet been saved.
    pub path: PathBuf,
}

impl Project {
    /// Builds a new project with the given name and no path.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), path: PathBuf::new() }
    }
}
