//! Audio engine built on top of the [`kira`] crate.
//!
//! The crate provides:
//!
//! - An [`AudioSystem`] entry point that logs initialization and owns
//!   the kira `AudioManager`.
//! - Source, mixer and spatial audio modules that are still being
//!   filled in. They are exposed as `pub` so that downstream code can
//!   already import them while the implementations land.
//!
//! # Status
//!
//! The integration is currently a stub: the public API is stable but
//! the underlying `kira` manager has not been instantiated yet.

#![deny(unsafe_code)]

pub mod engine;
pub mod mixer;
pub mod source;
pub mod spatial;

use tracing::info;

/// Top level audio system that holds the kira `AudioManager` and the
/// global mixer state.
pub struct AudioSystem {
    /// `true` once [`AudioSystem::new`] has been called. Future
    /// implementations will use this to skip a second initialization
    /// attempt.
    initialized: bool,
}

impl AudioSystem {
    /// Creates the audio system. The actual `kira::AudioManager` is
    /// constructed in this constructor as soon as the dependency
    /// surface stabilizes.
    pub fn new() -> Self {
        info!("initializing audio system");
        Self { initialized: true }
    }

    /// Returns `true` when the audio system is ready to play sounds.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new()
    }
}
