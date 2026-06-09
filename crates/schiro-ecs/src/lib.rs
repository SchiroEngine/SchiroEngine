//! Entity Component System built on top of [`bevy_ecs`].
//!
//! This crate re-exports the `bevy_ecs` types under the `engine` module
//! and provides the SchiroEngine-specific [`components`], [`systems`] and
//! [`schedule`] helpers that the runtime and editor rely on.
//!
//! # Quick start
//!
//! ```
//! use schiro_ecs::{init, World};
//!
//! let mut world = World::new();
//! init(&mut world);
//! ```

#![deny(unsafe_code)]

pub mod components;
pub mod schedule;
pub mod systems;

pub use bevy_ecs::system::Resource;
pub use bevy_ecs::world::World;
pub use bevy_ecs::{self as engine};
pub use components::*;
pub use systems::Time;
use tracing::info;

/// Initializes the default SchiroEngine resources into the supplied world.
///
/// Currently registers [`schedule::SystemOrder`] and [`systems::Time`].
pub fn init(world: &mut World) {
    info!("initializing ECS world");
    world.init_resource::<schedule::SystemOrder>();
    world.init_resource::<Time>();
}
