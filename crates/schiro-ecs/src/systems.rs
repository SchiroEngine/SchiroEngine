use bevy_ecs::prelude::*;
use glam::Quat;

use crate::components::{GlobalTransform, Rotator, Transform};

pub fn propagate_transforms(
    mut query: Query<(&Transform, &mut GlobalTransform), Changed<Transform>>,
) {
    for (transform, mut global) in query.iter_mut() {
        global.0 = transform.compute_matrix();
    }
}

pub fn rotate_entities(time: Res<Time>, mut query: Query<(&mut Transform, &Rotator)>) {
    for (mut transform, rotator) in query.iter_mut() {
        let delta = time.delta_seconds();
        transform.rotation *= Quat::from_rotation_x(rotator.speed.x * delta);
        transform.rotation *= Quat::from_rotation_y(rotator.speed.y * delta);
        transform.rotation *= Quat::from_rotation_z(rotator.speed.z * delta);
    }
}

#[derive(Resource, Default)]
pub struct Time {
    pub delta: f32,
    pub total: f32,
}

impl Time {
    pub fn delta_seconds(&self) -> f32 {
        self.delta
    }

    pub fn update(&mut self, dt: f32) {
        self.delta = dt;
        self.total += dt;
    }
}
