use comfy::Entity;

use crate::{behavior::{mental::IntentionCompleted, sanity::{Routine, Sanity}}, core::position::Ps, state::Reality};

use super::randomwalk::RandomStepRoutine;

#[derive(Debug, Clone)]
pub struct GoToRoutine {
    pub target: Ps
}


impl Routine for GoToRoutine {
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
            | IntentionCompleted::Undefined => {
                if sanity.no_intentions_left() {
                    sanity.intend_go_to(self.target)
                }
            }
            IntentionCompleted::None => (),
            IntentionCompleted::Failure => {
                let mut mov = RandomStepRoutine;
                mov.get_processing_fn(sanity, result, map, entity, dt);
            },
        }
    }
}
