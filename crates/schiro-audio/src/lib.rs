#![deny(unsafe_code)]

pub mod engine;
pub mod mixer;
pub mod source;
pub mod spatial;

use tracing::info;

pub struct AudioSystem {
    initialized: bool,
}

impl AudioSystem {
    pub fn new() -> Self {
        info!("initializing audio system");
        Self { initialized: true }
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new()
    }
}
