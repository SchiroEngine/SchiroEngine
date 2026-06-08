use bevy_ecs::prelude::*;

use crate::components::{GlobalTransform, Transform};

pub fn propagate_transforms(
    mut query: Query<(&Transform, &mut GlobalTransform), Changed<Transform>>,
) {
    for (transform, mut global) in query.iter_mut() {
        global.0 = transform.compute_matrix();
    }
}
