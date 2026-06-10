//! Tests for `schiro_core::Color`.

use schiro_core::Color;

mod common;
use common::assert_approx_eq;

const EPS: f32 = 1e-5;

#[test]
fn new_sets_channels() {
    let c = Color::new(0.1, 0.2, 0.3, 0.4);
    assert_eq!(c.r, 0.1);
    assert_eq!(c.g, 0.2);
    assert_eq!(c.b, 0.3);
    assert_eq!(c.a, 0.4);
}

#[test]
fn from_hex_decodes_rgba() {
    let c = Color::from_hex(0xFF_80_40_20);
    assert_approx_eq(c.r, 1.0, EPS, "r");
    assert_approx_eq(c.g, 0x80 as f32 / 255.0, EPS, "g");
    assert_approx_eq(c.b, 0x40 as f32 / 255.0, EPS, "b");
    assert_approx_eq(c.a, 0x20 as f32 / 255.0, EPS, "a");
}

#[test]
fn from_hex_zero_is_transparent_black() {
    let c = Color::from_hex(0);
    assert_eq!(c.r, 0.0);
    assert_eq!(c.g, 0.0);
    assert_eq!(c.b, 0.0);
    assert_eq!(c.a, 0.0);
}

#[test]
fn from_hex_white_is_opaque_white() {
    let c = Color::from_hex(0xFF_FF_FF_FF);
    assert_eq!(c.r, 1.0);
    assert_eq!(c.g, 1.0);
    assert_eq!(c.b, 1.0);
    assert_eq!(c.a, 1.0);
}

#[test]
fn with_alpha_preserves_rgb() {
    let c = Color::WHITE.with_alpha(0.5);
    assert_eq!(c.r, 1.0);
    assert_eq!(c.g, 1.0);
    assert_eq!(c.b, 1.0);
    assert_eq!(c.a, 0.5);
}

#[test]
fn srgb_to_linear_roundtrip_is_stable() {
    let original = Color::new(0.25, 0.5, 0.75, 1.0);
    let roundtrip = original.to_linear().to_srgb();
    assert_approx_eq(original.r, roundtrip.r, EPS, "r");
    assert_approx_eq(original.g, roundtrip.g, EPS, "g");
    assert_approx_eq(original.b, roundtrip.b, EPS, "b");
}

#[test]
fn srgb_black_stays_black() {
    let c = Color::BLACK.to_linear();
    assert_eq!(c.r, 0.0);
    assert_eq!(c.g, 0.0);
    assert_eq!(c.b, 0.0);
}

#[test]
fn srgb_white_stays_white() {
    let c = Color::WHITE.to_srgb();
    assert_approx_eq(c.r, 1.0, EPS, "r");
    assert_approx_eq(c.g, 1.0, EPS, "g");
    assert_approx_eq(c.b, 1.0, EPS, "b");
}

#[test]
fn srgb_alpha_is_preserved() {
    let c = Color::new(0.1, 0.2, 0.3, 0.42).to_srgb();
    assert_approx_eq(c.a, 0.42, EPS, "a");

    let c = Color::new(0.1, 0.2, 0.3, 0.42).to_linear();
    assert_approx_eq(c.a, 0.42, EPS, "a");
}

#[test]
fn to_linear_dark_uses_linear_segment() {
    // sRGB <= 0.04045 maps via c / 12.92
    let c = Color::new(0.02, 0.02, 0.02, 1.0).to_linear();
    let expected = 0.02 / 12.92;
    assert_approx_eq(c.r, expected, EPS, "dark r");
}

#[test]
fn into_vec4_preserves_channels() {
    let c = Color::new(0.1, 0.2, 0.3, 0.4);
    let v: glam::Vec4 = c.into();
    assert_eq!(v.x, 0.1);
    assert_eq!(v.y, 0.2);
    assert_eq!(v.z, 0.3);
    assert_eq!(v.w, 0.4);
}

#[test]
fn into_vec3_drops_alpha() {
    let c = Color::new(0.1, 0.2, 0.3, 0.4);
    let v: glam::Vec3 = c.into();
    assert_eq!(v.x, 0.1);
    assert_eq!(v.y, 0.2);
    assert_eq!(v.z, 0.3);
}

#[test]
fn vec4_into_color_preserves_channels() {
    let v = glam::Vec4::new(0.1, 0.2, 0.3, 0.4);
    let c: Color = v.into();
    assert_eq!(c.r, 0.1);
    assert_eq!(c.g, 0.2);
    assert_eq!(c.b, 0.3);
    assert_eq!(c.a, 0.4);
}

#[test]
fn constants_have_expected_values() {
    assert_eq!(Color::WHITE, Color::new(1.0, 1.0, 1.0, 1.0));
    assert_eq!(Color::BLACK, Color::new(0.0, 0.0, 0.0, 1.0));
    assert_eq!(Color::RED, Color::new(1.0, 0.0, 0.0, 1.0));
    assert_eq!(Color::GREEN, Color::new(0.0, 1.0, 0.0, 1.0));
    assert_eq!(Color::BLUE, Color::new(0.0, 0.0, 1.0, 1.0));
    assert_eq!(Color::TRANSPARENT, Color::new(0.0, 0.0, 0.0, 0.0));
}

#[test]
fn color_is_pod_with_size_16() {
    let c = Color::new(1.0, 0.5, 0.25, 1.0);
    let bytes = bytemuck::bytes_of(&c);
    assert_eq!(bytes.len(), 16);
}
