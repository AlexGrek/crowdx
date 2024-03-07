use crate::{behavior::creatures::PsOffsetProvider, core::position::Ps};

use super::carriableitem::{CarriableItemHandle, CarriableItems};

pub fn find_all_by<F>(carriables: &CarriableItems, predicate: F) -> Vec<CarriableItemHandle> where F: Fn (&CarriableItemHandle) -> bool {
    let items = carriables.lock();
    let mut found: Vec<CarriableItemHandle> = vec![];
    for (_item_entity, item) in items.iter() {
        if predicate(item) {
            found.push(item.clone());
        }
    }
    return found;
}

pub fn find_one_by<F>(carriables: &CarriableItems, predicate: F) -> Option<CarriableItemHandle> where F: Fn (&CarriableItemHandle) -> bool {
    let items = carriables.lock();
    for (_item_entity, item) in items.iter() {
        if predicate(item) {
            return Some(item.clone())
        }
    }
    None
}

pub fn find_all_with_type(carriables: &CarriableItems, item_type: Option<&'static str>) -> Vec<CarriableItemHandle> {
    let predicate = |item: &CarriableItemHandle| {
        item_type.is_none() || item_type.is_some_and(|tp| item.item_type == tp)
    };
    find_all_by(carriables, predicate)
}

pub fn find_all_available_with_type(carriables: &CarriableItems, item_type: Option<&'static str>) -> Vec<CarriableItemHandle> {
    // println!("Called find_all: {:?}", item_type);
    let predicate = |item: &CarriableItemHandle| {
        item.available() && (item_type.is_none() || item_type.is_some_and(|tp| item.item_type == tp))
    };
    find_all_by(carriables, predicate)
}

pub fn find_closest_available_with_type(carriables: &CarriableItems, item_type: Option<&'static str>, close_to: Ps) -> Option<CarriableItemHandle> {
    let mut all = find_all_available_with_type(carriables, item_type);
    all.sort_unstable_by_key(|val| close_to.manhattan_distance(&val.get_ps()));
    // println!("Found: {:?}", all);
    all.first().map(|handle| handle.to_owned())
}