use comfy::Entity;

use crate::{
    behavior::messaging::MessagingHost, core::{
        position::{Ps, PsProvider},
        Initializable,
    }, state::Reality
};

use super::{
    carriable::carriablesearch::find_closest_available_with_type,
    creatures::PsOffsetProvider,
    item_types::{BONE, TRASHCAN},
    mental::{IntentionClass, IntentionCompleted},
    routine::entitytypehunter::EntityTypeHunter,
    sanity::{Sanity, SelfAware},
};

#[derive(Debug, Clone)]
pub struct DogRoutine {
    hunter: EntityTypeHunter,
}

impl super::sanity::Routine for DogRoutine {
    fn get_processing_fn(
        &mut self,
        sanity: &mut Sanity,
        result: IntentionCompleted,
        map: &Reality,
        entity: Entity,
        dt: f32,
    ) {
        match result {
            IntentionCompleted::Success
            | IntentionCompleted::Failure
            | IntentionCompleted::Undefined => {
                if sanity.carrier.has_anything() && sanity.no_intentions_left() {
                    // drop that to the nearest trash can!
                    let target_opt = find_closest_available_with_type(
                        &map.carriables,
                        Some(TRASHCAN),
                        sanity.get_current_ps(),
                    );
                    match target_opt {
                        Some(target) => {
                            sanity.intend_go_to(target.position);
                            sanity
                                .intend_with_priority(-1, IntentionClass::ConsumeAnyCarriedItem());
                            let cell = map.cellmap.pick_random_passable_ps();
                            // go away
                            sanity.intend_with_priority(-2, IntentionClass::MoveToDestination(cell))
                        }
                        None => sanity.intend(IntentionClass::WaitCycles(100)),
                    }
                } else if sanity.no_intentions_left() {
                    self.hunter
                        .get_processing_fn(sanity, result, map, entity, dt)
                }
            }
            IntentionCompleted::None => (),
        }
    }
}

#[derive(Debug)]
pub struct Dog {
    pub name: String,
    pub initialized: bool,
    pub sa: SelfAware<DogRoutine>,
}

impl Dog {
    pub fn new(name: String, speed: f32, pos: Ps) -> Self {
        Self {
            name,
            initialized: false,
            sa: SelfAware::new(
                speed,
                pos,
                Box::new(DogRoutine {
                    hunter: EntityTypeHunter::new(BONE, true),
                }),
            ),
        }
    }
}

impl Initializable for Dog {
    fn initialize(
        &mut self,
        entity: &Entity,
        _transform: &mut comfy::Transform,
        reality: &mut Reality,
    ) {
        let mut msg = reality.messaging.lock();
        let handle = MessagingHost::new();
        println!("Initialized doge {:?}: {:?}", entity, handle);
        msg.insert(*entity, handle);
        self.initialized = true;
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl PsOffsetProvider for Dog {
    fn get_ps(&self) -> Ps {
        self.sa.get_ps()
    }

    fn get_offset(&self) -> comfy::Vec2 {
        self.sa.get_offset()
    }
}
