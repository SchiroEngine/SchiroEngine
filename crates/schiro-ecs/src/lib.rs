#![deny(unsafe_code)]

pub mod components;
pub mod schedule;
pub mod systems;

pub use bevy_ecs::{
    self as engine,
    system::Resource,
    world::World,
};
pub use components::*;
pub use systems::Time;

use tracing::info;

pub fn init(world: &mut World) {
    info!("initializing ECS world");
    world.init_resource::<schedule::SystemOrder>();
    world.init_resource::<Time>();
}
