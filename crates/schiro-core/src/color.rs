//! Color value used by materials, lights, gizmos and editor UI.
//!
//! Colors are stored in linear floating point space. Conversion helpers
//! are provided to and from sRGB encoded `u32` literals (HTML-style).

use glam::{Vec3, Vec4};

/// RGBA color stored in linear space, ready to be uploaded to a GPU uniform
/// or sampled from a material.
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Color {
    /// Red channel, linear.
    pub r: f32,
    /// Green channel, linear.
    pub g: f32,
    /// Blue channel, linear.
    pub b: f32,
    /// Alpha channel, linear, not gamma corrected.
    pub a: f32,
}

impl Color {
    /// Solid white `(1, 1, 1, 1)`.
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);

    /// Solid black `(0, 0, 0, 1)`.
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    /// Solid red `(1, 0, 0, 1)`.
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);

    /// Solid green `(0, 1, 0, 1)`.
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);

    /// Solid blue `(0, 0, 1, 1)`.
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);

    /// Fully transparent black.
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Builds a color from raw linear channels.
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Decodes a `0xRRGGBBAA` value (most significant byte is red) into a
    /// linear color.
    ///
    /// The input is interpreted as sRGB, the output is linear.
    #[inline]
    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 24) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let b = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let a = (hex & 0xFF) as f32 / 255.0;
        Self { r, g, b, a }
    }

    /// Returns a copy of this color with a different alpha value.
    #[inline]
    pub fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    /// Converts the color from sRGB to linear space.
    ///
    /// The alpha channel is left untouched.
    #[inline]
    pub fn to_linear(self) -> Self {
        Self {
            r: srgb_to_linear(self.r),
            g: srgb_to_linear(self.g),
            b: srgb_to_linear(self.b),
            a: self.a,
        }
    }

    /// Converts the color from linear to sRGB space.
    ///
    /// The alpha channel is left untouched.
    #[inline]
    pub fn to_srgb(self) -> Self {
        Self {
            r: linear_to_srgb(self.r),
            g: linear_to_srgb(self.g),
            b: linear_to_srgb(self.b),
            a: self.a,
        }
    }
}

impl From<Color> for Vec4 {
    #[inline]
    fn from(c: Color) -> Self {
        Vec4::new(c.r, c.g, c.b, c.a)
    }
}

impl From<Color> for Vec3 {
    #[inline]
    fn from(c: Color) -> Self {
        Vec3::new(c.r, c.g, c.b)
    }
}

impl From<Vec4> for Color {
    #[inline]
    fn from(v: Vec4) -> Self {
        Self::new(v.x, v.y, v.z, v.w)
    }
}

#[inline]
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}
