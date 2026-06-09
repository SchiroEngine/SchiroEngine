//! Loaders and error types used by the asset server.
//!
//! Defines the [`AssetLoader`] trait that any custom loader can
//! implement, along with the [`AssetLoadError`] enum returned by
//! every loader shipped with the engine.

use crate::Asset;

/// Trait implemented by every concrete loader registered with the
/// [`crate::AssetServer`].
///
/// Loaders are stateless and are called with the raw bytes of the file
/// to import.
pub trait AssetLoader: Send + Sync {
    /// Concrete asset type produced by this loader.
    type Asset: crate::Asset;
    /// Parses the supplied bytes and produces an asset.
    fn load(&self, data: &[u8]) -> Result<Self::Asset, AssetLoadError>;
    /// File extensions this loader accepts, without the leading dot.
    fn extensions(&self) -> &[&str];
}

/// Errors returned by the various loaders.
#[derive(Debug, thiserror::Error)]
pub enum AssetLoadError {
    /// I/O failure while reading the source file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// The file was read but could not be parsed.
    #[error("parse error: {0}")]
    Parse(String),

    /// The file extension is not associated with any registered loader.
    #[error("unsupported format")]
    UnsupportedFormat,

    /// The glTF importer returned an error.
    #[error("glTF error: {0}")]
    Gltf(String),
}
