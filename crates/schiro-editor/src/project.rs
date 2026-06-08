use std::path::PathBuf;

pub struct Project {
    pub name: String,
    pub path: PathBuf,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: PathBuf::new(),
        }
    }
}
