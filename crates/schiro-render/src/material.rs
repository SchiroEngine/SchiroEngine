//! PBR material description.
//!
//! Mirrors the fields consumed by the PBR shader so that the renderer
//! can build a matching uniform buffer or bind group from a [`Material`]
//! value.

use schiro_core::Color;

use super::texture::TextureHandle;

/// PBR material parameters.
///
/// All factors live in the `[0, 1]` range and are interpreted in
/// linear space. Textures, when present, override the matching scalar
/// value.
pub struct Material {
    /// Base color of the surface.
    pub base_color: Color,
    /// Optional base color texture, sampled in the fragment shader.
    pub base_color_texture: Option<super::texture::TextureHandle>,
    /// Metallic factor. `0` = dielectric, `1` = full metal.
    pub metallic: f32,
    /// Roughness factor. `0` = perfectly smooth, `1` = perfectly rough.
    pub roughness: f32,
    /// Optional tangent space normal map.
    pub normal_map: Option<super::texture::TextureHandle>,
    /// Emissive contribution added on top of the lit surface.
    pub emissive: Color,
    /// Optional emissive texture, multiplied with [`Material::emissive`].
    pub emissive_texture: Option<super::texture::TextureHandle>,
    /// How the material handles transparency.
    pub alpha_mode: AlphaMode,
    /// When `true`, the material is rendered on both back and front
    /// faces.
    pub double_sided: bool,
}

/// Alpha blending mode of a [`Material`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaMode {
    /// Fully opaque material. The alpha channel is ignored.
    Opaque,
    /// Alpha tested material. Pixels with `alpha < cutoff` are
    /// discarded.
    Mask,
    /// Alpha blended material. The framebuffer is blended with the
    /// incoming fragment.
    Blend,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: Color::WHITE,
            base_color_texture: None,
            metallic: 0.0,
            roughness: 0.5,
            normal_map: None,
            emissive: Color::BLACK,
            emissive_texture: None,
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
        }
    }
}

// Re-exported here to make the public surface of the module explicit.
#[allow(unused_imports)]
use TextureHandle as _TextureHandleReexport;
