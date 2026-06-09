//! Type safe identifiers backed by a slot map.
//!
//! The [`Id`] newtype wraps a [`slotmap`] key so that the type system can
//! distinguish between different identifier flavors (entity, asset, light
//! and so on) without using sentinel values.

use slotmap::{new_key_type, SlotMap};

new_key_type! {
    /// Opaque, stable handle used to refer to engine resources.
    pub struct Id;
}

/// Slot map keyed by [`Id`] values.
pub type IdMap<T> = SlotMap<Id, T>;

/// Builds a new empty [`IdMap`] with the appropriate key type.
pub fn new_id_map<T>() -> IdMap<T> {
    SlotMap::with_key()
}
