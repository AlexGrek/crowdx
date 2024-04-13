use comfy::Entity;

use crate::{behavior::{mental::IntentionCompleted, messaging::communication::Communicator, sanity::{Routine, Sanity}}, core::position::Ps, state::Reality};

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
        communication: &Communicator,
        dt: f32,
    ) {
        // println!("GoToRoutine: {:?} {:?}", entity, result);
        match result {
            IntentionCompleted::Success
            | IntentionCompleted::Undefined => {
                if sanity.no_intentions_left() {
                    // println!("{:?} Intends go to: {:?}", entity, self.target);
                    sanity.intend_go_to(self.target)
                }
            }
            IntentionCompleted::None => (),
            IntentionCompleted::Failure => {
                let mut mov = RandomStepRoutine;
                // println!("Random step: {:?} {:?}", entity, result);
                mov.get_processing_fn(sanity, result, map, entity, communication, dt);
            },
        }
    }
}
