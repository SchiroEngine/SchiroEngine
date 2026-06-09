//! Texture handle and format enumeration.
//!
//! The actual GPU resources live in the renderer; this module only
//! exposes the metadata needed by the rest of the engine to refer to
//! textures without holding a raw `wgpu::Texture`.

/// Opaque identifier for a texture registered with the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(u64);

/// CPU side description of a texture.
pub struct Texture {
    /// Renderer handle for the texture.
    pub handle: TextureHandle,
    /// Texture size, in texels.
    pub size: (u32, u32),
    /// Pixel format of the texture.
    pub format: TextureFormat,
}

/// Pixel formats supported by the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    /// 8 bit per channel unsigned normalized RGBA.
    Rgba8Unorm,
    /// 8 bit per channel sRGB encoded RGBA.
    Rgba8UnormSrgb,
    /// 8 bit per channel unsigned normalized BGRA.
    Bgra8Unorm,
    /// 8 bit per channel sRGB encoded BGRA.
    Bgra8UnormSrgb,
    /// 16 bit floating point per channel RGBA.
    Rgba16Float,
    /// 32 bit floating point per channel RGBA.
    Rgba32Float,
    /// 32 bit floating point depth.
    Depth32Float,
}
