#[derive(Debug, Clone)]
pub struct EntityTypeHunter {
    item_type: &'static str,
    pick: bool,
}

impl EntityTypeHunter {
    pub fn new(item_type: &'static str, pick: bool) -> Self {
        Self { item_type, pick }
    }
}

use comfy::Entity;

use crate::{
    behavior::{
        carriable::
            carriablesearch::find_closest_available_with_type, mental::{IntentionClass, IntentionCompleted}, messaging::communication::Communicator, sanity::*
    }, core::position::PsProvider, state::Reality
};

use super::randomwalk::RandomWalkRoutine;

impl Routine for EntityTypeHunter {
    fn get_processing_fn(
        &mut self,
        sanity: &mut Sanity,
        result: IntentionCompleted,
        map: &Reality,
        entity: Entity,
        communication: &Communicator,
        dt: f32,
    ) {
        match result {
            IntentionCompleted::Success
            | IntentionCompleted::Failure
            | IntentionCompleted::Undefined => {
                let target_opt = find_closest_available_with_type(
                    &map.carriables,
                    Some(self.item_type),
                    sanity.get_current_ps(),
                );
                match target_opt {
                    Some(target) => {
                        if sanity.no_intentions_left() {
                            // go and take
                            // println!("New intention: take item {:?} at {:?}", self.item_type, target.position);
                            sanity
                                .intend_with_priority(2, IntentionClass::MoveToPs(target.position));
                            sanity.intend_with_priority(
                                1,
                                if self.pick {
                                    IntentionClass::PickItemOfType(self.item_type)
                                } else {
                                    IntentionClass::ConsumeItemOfType(self.item_type)
                                },
                            )
                        }
                    }
                    None => {
                        // just move randomly
                        let mut rand = RandomWalkRoutine;
                        rand.get_processing_fn(sanity, result, map, entity, communication, dt);
                    }
                }
            }
            IntentionCompleted::None => (),
        }
    }
}
