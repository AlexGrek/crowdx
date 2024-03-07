use comfy::{Entity, HashSet};
use super::{carriable::carriableitem::{CarriableItemHandle, CarriableItems}, creatures::PsOffsetProvider};

#[derive(Debug)]
pub struct Carrier {
    items_carried: HashSet<Entity>
}

impl Carrier {
    pub fn new() -> Self {
        Self { items_carried: HashSet::new() }
    }

    pub fn pick_up(&mut self, myself: Entity, handle: &mut CarriableItemHandle) {
        println!("Taking item {:?} by {:?}", handle, myself);
        handle.take(myself);
        self.items_carried.insert(handle.item_id);
    }

    pub fn has_anything(&self) -> bool {
        return !self.items_carried.is_empty()
    }

    pub fn has_item(&self, entity: &Entity) -> bool {
        return self.items_carried.contains(entity);
    }

    pub fn drop(&mut self, handle: &mut CarriableItemHandle) {
        handle.release();
        self.items_carried.remove(&handle.item_id);
    }

    pub fn try_drop(&mut self, handle: &mut CarriableItemHandle) -> bool {
        if !self.has_item(&handle.item_id) {
            false
        } else {
            self.drop(handle);
            true
        }
    }

    pub fn try_consume(&mut self, handle: &mut CarriableItemHandle) -> bool {
        if !self.has_item(&handle.item_id) {
            false
        } else {
            self.drop(handle);
            handle.consume();
            true
        }
    }

    pub fn update_positions(&mut self, items_mutex: &CarriableItems, me: &dyn PsOffsetProvider) {
        let mut to_remove: HashSet<Entity> = HashSet::new();
        let mut items = items_mutex.lock();
        for item in self.items_carried.iter() {
            match items.get_mut(item) {
                Some(handle) => {
                    handle.copy_position(me)
                },
                None => { to_remove.insert(item.to_owned()); }
            }
        }
        for dead_item in to_remove.iter() {
            self.items_carried.remove(dead_item);
        }
    }
}