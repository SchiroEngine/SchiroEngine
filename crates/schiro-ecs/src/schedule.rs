//! Scheduling primitives built on top of [`bevy_ecs`].
//!
//! Provides a [`CoreSet`] enum of system stages and a placeholder
//! [`SystemOrder`] resource intended for future configuration of the
//! default schedule.

use bevy_ecs::prelude::*;

/// High level scheduling stages used by the engine.
///
/// Stages are evaluated in declaration order, matching the
/// `bevy_ecs::schedule::Stage` convention.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreSet {
    /// Runs once at the start of a frame, before any user system.
    First,
    /// Runs just before the user [`CoreSet::Update`] stage.
    PreUpdate,
    /// User defined systems, runs once per frame.
    Update,
    /// Runs just after [`CoreSet::Update`].
    PostUpdate,
    /// Runs at the very end of the frame.
    Last,
}

/// Reserved resource for runtime-tweakable system ordering.
///
/// Currently unused: the resource exists so that future configuration
/// panels can mutate the schedule without changing the type signatures
/// of the systems that read it.
#[derive(Resource, Default)]
pub struct SystemOrder;
