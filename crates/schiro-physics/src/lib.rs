#![deny(unsafe_code)]

pub mod collider;
pub mod joint;
pub mod query;
pub mod rigid_body;
pub mod world;

pub use world::PhysicsWorld;

use tracing::info;

pub fn init() {
    info!("initializing physics engine");
}
