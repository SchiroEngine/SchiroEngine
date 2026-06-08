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

pub trait Asset: Send + Sync + 'static {
    fn type_name() -> &'static str;
}

pub struct AssetServer {
    cache: RwLock<HashMap<(PathBuf, TypeId), Arc<dyn std::any::Any + Send + Sync>>>,
}

impl AssetServer {
    pub fn new() -> Self {
        info!("asset server initialized");
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn load<T: Asset + Clone, F>(&self, path: impl AsRef<Path>, loader: F) -> Result<Arc<T>, AssetLoadError>
    where
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

    pub fn clear(&self) {
        self.cache.write().clear();
    }
}

impl Default for AssetServer {
    fn default() -> Self {
        Self::new()
    }
}
