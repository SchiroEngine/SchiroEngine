pub trait AssetLoader: Send + Sync {
    type Asset: crate::Asset;
    fn load(&self, data: &[u8]) -> Result<Self::Asset, AssetLoadError>;
    fn extensions(&self) -> &[&str];
}

#[derive(Debug, thiserror::Error)]
pub enum AssetLoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("unsupported format")]
    UnsupportedFormat,
    #[error("glTF error: {0}")]
    Gltf(String),
}
