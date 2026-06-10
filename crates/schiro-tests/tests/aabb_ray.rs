//! Tests for `schiro_core::Aabb` and `schiro_core::Ray`.

use schiro_core::{Aabb, Ray};

mod common;
use common::assert_approx_eq;

const EPS: f32 = 1e-5;

#[test]
fn aabb_new_keeps_corners() {
    let a = Aabb::new(glam::Vec3::splat(-1.0), glam::Vec3::splat(2.0));
    assert_eq!(a.min, glam::Vec3::splat(-1.0));
    assert_eq!(a.max, glam::Vec3::splat(2.0));
}

#[test]
fn aabb_zero_is_origin() {
    let z = Aabb::ZERO;
    assert_eq!(z.min, glam::Vec3::ZERO);
    assert_eq!(z.max, glam::Vec3::ZERO);
}

#[test]
fn aabb_from_points_empty_returns_extremes() {
    let a = Aabb::from_points(&[]);
    assert_eq!(a.min, glam::Vec3::splat(f32::MAX));
    assert_eq!(a.max, glam::Vec3::splat(f32::MIN));
}

#[test]
fn aabb_from_points_encloses_all() {
    let pts = [[0.0, 0.0, 0.0], [1.0, -1.0, 2.0], [-2.0, 0.5, 1.0]];
    let a = Aabb::from_points(&pts);
    assert_eq!(a.min, glam::Vec3::new(-2.0, -1.0, 0.0));
    assert_eq!(a.max, glam::Vec3::new(1.0, 0.5, 2.0));
}

#[test]
fn aabb_center_size_radius() {
    let a = Aabb::new(glam::Vec3::new(-1.0, -2.0, -3.0), glam::Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(a.center(), glam::Vec3::ZERO);
    assert_eq!(a.size(), glam::Vec3::new(2.0, 4.0, 6.0));
    // radius = length(size) / 2 = sqrt(4 + 16 + 36) / 2
    let expected_radius = (4.0_f32 + 16.0 + 36.0).sqrt() * 0.5;
    assert_approx_eq(a.radius(), expected_radius, EPS, "radius");
}

#[test]
fn aabb_transform_translation_preserves_shape() {
    let a = Aabb::new(glam::Vec3::ZERO, glam::Vec3::new(1.0, 1.0, 1.0));
    let m = glam::Mat4::from_translation(glam::Vec3::new(5.0, 0.0, -3.0));
    let b = a.transform(&m);
    assert_eq!(b.min, glam::Vec3::new(5.0, 0.0, -3.0));
    assert_eq!(b.max, glam::Vec3::new(6.0, 1.0, -2.0));
}

#[test]
fn aabb_transform_scaling_grows_box() {
    let a = Aabb::new(glam::Vec3::splat(-1.0), glam::Vec3::splat(1.0));
    let m = glam::Mat4::from_scale(glam::Vec3::splat(3.0));
    let b = a.transform(&m);
    assert_eq!(b.min, glam::Vec3::splat(-3.0));
    assert_eq!(b.max, glam::Vec3::splat(3.0));
}

#[test]
fn ray_new_normalizes_direction() {
    let r = Ray::new(glam::Vec3::ZERO, glam::Vec3::new(2.0, 0.0, 0.0));
    assert_approx_eq(r.direction.x, 1.0, EPS, "x");
    assert_approx_eq(r.direction.length(), 1.0, EPS, "length");
}

#[test]
fn ray_point_at() {
    let r = Ray::new(glam::Vec3::ZERO, glam::Vec3::X);
    assert_eq!(r.point_at(0.0), glam::Vec3::ZERO);
    assert_eq!(r.point_at(2.0), glam::Vec3::new(2.0, 0.0, 0.0));
    assert_eq!(r.point_at(-1.0), glam::Vec3::new(-1.0, 0.0, 0.0));
}

#[test]
fn ray_hits_aabb_from_outside() {
    let r = Ray::new(glam::Vec3::new(-5.0, 0.0, 0.0), glam::Vec3::X);
    let a = Aabb::new(glam::Vec3::splat(-1.0), glam::Vec3::splat(1.0));
    let t = r.intersects_aabb(&a).expect("should hit");
    assert_approx_eq(t, 4.0, EPS, "entry distance");
}

#[test]
fn ray_inside_aabb_hits_immediately() {
    let r = Ray::new(glam::Vec3::ZERO, glam::Vec3::X);
    let a = Aabb::new(glam::Vec3::splat(-1.0), glam::Vec3::splat(1.0));
    let t = r.intersects_aabb(&a).expect("should hit");
    assert_approx_eq(t, 0.0, EPS, "inside distance");
}

#[test]
fn ray_misses_box_pointing_away() {
    let r = Ray::new(glam::Vec3::new(5.0, 0.0, 0.0), glam::Vec3::new(1.0, 1.0, 0.0));
    let a = Aabb::new(glam::Vec3::splat(-1.0), glam::Vec3::splat(1.0));
    assert!(r.intersects_aabb(&a).is_none());
}

#[test]
fn ray_aimed_above_box_misses() {
    let r = Ray::new(glam::Vec3::new(0.0, 5.0, 0.0), glam::Vec3::Y);
    let a = Aabb::new(glam::Vec3::splat(-1.0), glam::Vec3::splat(1.0));
    assert!(r.intersects_aabb(&a).is_none());
}

#[test]
fn ray_hits_negative_side() {
    let r = Ray::new(glam::Vec3::new(0.0, 0.0, 5.0), glam::Vec3::new(0.0, 0.0, -1.0));
    let a = Aabb::new(glam::Vec3::splat(-1.0), glam::Vec3::splat(1.0));
    let t = r.intersects_aabb(&a).expect("should hit");
    assert_approx_eq(t, 4.0, EPS, "negative-z entry");
}
