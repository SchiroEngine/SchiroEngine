//! Shared helpers for the integration tests.
//!
//! Anything in this module is part of the `common` test module
//! rather than a `pub` item of the crate, so it is not linked into
//! the library output.

#![allow(dead_code)]

use std::ops::{Add, Mul, Sub};

/// Asserts that two `f32` values are within `eps` of each other,
/// with a custom message.
pub fn assert_approx_eq(a: f32, b: f32, eps: f32, ctx: &str) {
    assert!((a - b).abs() <= eps, "{ctx}: |{a} - {b}| = {} > {eps}", (a - b).abs(),);
}

/// Asserts that two `glam::Vec3` values are within `eps` per
/// component.
pub fn assert_vec3_approx_eq(a: glam::Vec3, b: glam::Vec3, eps: f32, ctx: &str) {
    assert_approx_eq(a.x, b.x, eps, &format!("{ctx}.x"));
    assert_approx_eq(a.y, b.y, eps, &format!("{ctx}.y"));
    assert_approx_eq(a.z, b.z, eps, &format!("{ctx}.z"));
}

/// Asserts that two 4x4 matrices are within `eps` per component.
pub fn assert_mat4_approx_eq(a: glam::Mat4, b: glam::Mat4, eps: f32, ctx: &str) {
    for col in 0..4 {
        for row in 0..4 {
            let lhs = a.col(col)[row];
            let rhs = b.col(col)[row];
            assert_approx_eq(lhs, rhs, eps, &format!("{ctx}[{col}][{row}]"));
        }
    }
}

/// Tiny statistical helpers used by several tests.
#[derive(Debug, Clone, Copy, Default)]
pub struct Stats {
    /// Sample count.
    pub n: usize,
    /// Sum of the samples.
    pub sum: f64,
    /// Sum of the squared samples.
    pub sum_sq: f64,
}

impl Stats {
    /// Builds an empty statistics accumulator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one sample to the running statistics.
    pub fn push(&mut self, x: f64) {
        self.n += 1;
        self.sum += x;
        self.sum_sq += x * x;
    }

    /// Returns the arithmetic mean, or `0.0` when the sample count is
    /// zero.
    pub fn mean(&self) -> f64 {
        if self.n == 0 {
            0.0
        } else {
            self.sum / self.n as f64
        }
    }

    /// Returns the variance of the samples. Returns `0.0` for an
    /// empty or singleton sample set.
    pub fn variance(&self) -> f64 {
        if self.n < 2 {
            return 0.0;
        }
        let mean = self.mean();
        (self.sum_sq / self.n as f64) - mean * mean
    }
}

impl Add for Stats {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self { n: self.n + rhs.n, sum: self.sum + rhs.sum, sum_sq: self.sum_sq + rhs.sum_sq }
    }
}

impl Sub for Stats {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self { n: self.n - rhs.n, sum: self.sum - rhs.sum, sum_sq: self.sum_sq - rhs.sum_sq }
    }
}

impl Mul<f64> for Stats {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self { n: self.n, sum: self.sum * rhs, sum_sq: self.sum_sq * rhs * rhs }
    }
}
