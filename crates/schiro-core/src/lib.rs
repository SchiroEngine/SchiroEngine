//! Foundation types and utilities shared across the SchiroEngine workspace.
//!
//! This crate provides the low-level building blocks used by every other
//! SchiroEngine crate:
//!
//! - [`Color`] — RGBA color value with sRGB <-> linear conversions.
//! - [`Aabb`] and [`Ray`] — basic spatial primitives for picking, culling and
//!   intersection tests.
//! - [`Id`] and `IdMap` — type-safe handle and slot map for stable
//!   identifiers.
//! - [`math`] — re-export of the `glam` linear algebra types so downstream
//!   crates depend on a single math facade.
//! - [`profiling`] — `tracing` initialization helpers.
//!
//! # Conventions
//!
//! Types in this crate are intentionally minimal and `no_std`-friendly where
//! possible. They never depend on a renderer, an ECS or a platform layer.

#![deny(unsafe_code)]

pub mod aabb;
pub mod color;
pub mod id;
pub mod math;
pub mod profiling;

pub use aabb::{Aabb, Ray};
pub use color::Color;
pub use id::Id;
pub use math::*;
