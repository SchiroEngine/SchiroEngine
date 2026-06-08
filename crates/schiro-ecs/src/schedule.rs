use bevy_ecs::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreSet {
    First,
    PreUpdate,
    Update,
    PostUpdate,
    Last,
}

#[derive(Resource, Default)]
pub struct SystemOrder;
