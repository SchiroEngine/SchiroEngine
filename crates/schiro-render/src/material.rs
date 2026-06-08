use schiro_core::Color;

pub struct Material {
    pub base_color: Color,
    pub base_color_texture: Option<super::texture::TextureHandle>,
    pub metallic: f32,
    pub roughness: f32,
    pub normal_map: Option<super::texture::TextureHandle>,
    pub emissive: Color,
    pub emissive_texture: Option<super::texture::TextureHandle>,
    pub alpha_mode: AlphaMode,
    pub double_sided: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaMode {
    Opaque,
    Mask,
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
