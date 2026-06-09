//! Asset loading, processing and storage for the SchiroEngine.
//!
//! Provides:
//!
//! - The [`Asset`] trait, implemented by every loadable resource.
//! - The [`AssetServer`] type, a thread-safe cache that deduplicates loads
//!   by path and type.
//! - The [`Handle<T>`] smart pointer returned to callers, which can be
//!   resolved once the underlying data is ready.
//! - Concrete asset types ([`MeshAsset`], [`TextureAsset`],
//!   [`MaterialAsset`]) and loaders for them, including a glTF importer
//!   and procedural mesh generators.

#![deny(unsafe_code)]

pub mod gltf;
pub mod handle;
pub mod loader;
pub mod procedural;
pub mod server;
pub mod types;

use std::any::TypeId;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;
use tracing::info;

pub use handle::Handle;
pub use loader::AssetLoadError;

pub use types::{MaterialAsset, MeshAsset, TextureAsset};

/// Trait implemented by every type that can be stored in an
/// [`AssetServer`].
///
/// Implementors must be `Send + Sync + 'static` and should also be
/// `Clone` so the server can hand out cheap copies.
pub trait Asset: Send + Sync + 'static {
    /// Short, human readable type name used in log messages.
    fn type_name() -> &'static str;
}

/// Thread safe asset cache indexed by path and asset type.
///
/// The server deduplicates loads: calling [`AssetServer::load`] twice with
/// the same path and type returns clones of the same asset without
/// re-reading the file.
pub struct AssetServer {
    cache: RwLock<HashMap<(PathBuf, TypeId), Arc<dyn std::any::Any + Send + Sync>>>,
}

impl AssetServer {
    /// Creates a new, empty asset server.
    pub fn new() -> Self {
        info!("asset server initialized");
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Loads (or fetches from cache) the asset at `path`.
    ///
    /// The `loader` closure is only called on a cache miss and is given
    /// the canonicalized path of the file. It must produce a fresh value
    /// of type `T` or return an [`AssetLoadError`].
    pub fn load<T, F>(&self, path: impl AsRef<Path>, loader: F) -> Result<Arc<T>, AssetLoadError>
    where
        T: Asset + Clone,
        F: FnOnce(&Path) -> Result<T, AssetLoadError>,
    {
        let path = path.as_ref().to_path_buf();
        let key = (path.clone(), TypeId::of::<T>());

        {
            let cache = self.cache.read();
            if let Some(entry) = cache.get(&key) {
                if let Some(data) = entry.downcast_ref::<T>() {
                    return Ok(Arc::new(data.clone()));
                }
            }
        }

        info!("loading asset: {} ({})", path.display(), T::type_name());
        let asset = loader(&path)?;
        let arc: Arc<T> = Arc::new(asset);
        self.cache
            .write()
            .insert(key, arc.clone() as Arc<dyn std::any::Any + Send + Sync>);
        Ok(arc)
    }

    /// Drops every cached asset. Subsequent loads will re-read the files.
    pub fn clear(&self) {
        self.cache.write().clear();
    }
}

impl Default for AssetServer {
    fn default() -> Self {
        Self::new()
    }
}
