//! Built-in systems and shared resources.
//!
//! The functions in this module are regular Bevy systems. The host
//! application is responsible for adding them to a [`bevy_ecs::schedule::Schedule`]
//! in the order it wants them to run.

use bevy_ecs::prelude::*;
use glam::Quat;

use crate::components::{GlobalTransform, Rotator, Transform};

/// Recomputes the [`GlobalTransform`] of every entity whose [`Transform`]
/// was changed since the last run.
pub fn propagate_transforms(
    mut query: Query<(&Transform, &mut GlobalTransform), Changed<Transform>>,
) {
    for (transform, mut global) in query.iter_mut() {
        global.0 = transform.compute_matrix();
    }
}

/// Advances the rotation of every entity carrying a [`Rotator`]
/// component, scaled by the elapsed time.
pub fn rotate_entities(time: Res<Time>, mut query: Query<(&mut Transform, &Rotator)>) {
    for (mut transform, rotator) in query.iter_mut() {
        let delta = time.delta_seconds();
        transform.rotation *= Quat::from_rotation_x(rotator.speed.x * delta);
        transform.rotation *= Quat::from_rotation_y(rotator.speed.y * delta);
        transform.rotation *= Quat::from_rotation_z(rotator.speed.z * delta);
    }
}

/// Frame timing resource used by systems that need a delta time.
#[derive(Resource, Default)]
pub struct Time {
    /// Time elapsed since the previous frame, in seconds.
    pub delta: f32,
    /// Total time elapsed since the application started, in seconds.
    pub total: f32,
}

impl Time {
    /// Returns the time elapsed since the previous frame, in seconds.
    pub fn delta_seconds(&self) -> f32 {
        self.delta
    }

    /// Updates both the delta and total time fields.
    pub fn update(&mut self, dt: f32) {
        self.delta = dt;
        self.total += dt;
    }
}
