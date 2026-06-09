//! Math facade re-exporting the [`glam`] crate.
//!
//! Every SchiroEngine crate that needs vector or matrix types should
//! import them from here so that swapping the underlying math library
//! later only requires touching this module.

pub use glam::*;
