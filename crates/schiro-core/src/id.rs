use slotmap::{new_key_type, SlotMap};

new_key_type! {
    pub struct Id;
}

pub type IdMap<T> = SlotMap<Id, T>;

pub fn new_id_map<T>() -> IdMap<T> {
    SlotMap::with_key()
}
