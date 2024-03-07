use comfy::Entity;

use crate::{
    behavior::{mental::{IntentionCompleted, PRIORITY_BASE}, routing::SUCCESSORS, sanity::*}, core::position::{PsProvider, PsSigned}, state::Reality
};


#[derive(Debug, Clone)]
pub struct RandomWalkRoutine;


impl Routine for RandomWalkRoutine {
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
                    let cell = map.cellmap.pick_random_passable_ps();
                    // println!("New intention: move randomly at {:?}", cell);
                    sanity.intend_go_to(cell)
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

#[derive(Debug, Clone)]
pub struct RandomStepRoutine;

impl Routine for RandomStepRoutine {
    fn get_processing_fn(
        &mut self,
        sanity: &mut Sanity,
        result: IntentionCompleted,
        map: &Reality,
        _entity: Entity,
        _dt: f32,
    ) {
        match result {
            IntentionCompleted::Success
            | IntentionCompleted::Failure
            | IntentionCompleted::Undefined => {
                if sanity.no_intentions_left() {
                    let succ: PsSigned = comfy::ChooseRandom::choose(&SUCCESSORS.to_vec()).unwrap().to_owned().into();
                    let cell = sanity.get_current_ps() + succ;
                    if map.cellmap.xy_within_bounds(&cell) && map.cellmap.get_pos(&cell).is_passable(true) {
                        sanity.intend_go_to(cell.into());
                    } else {
                        sanity.mind.intend_cycles_count(10, PRIORITY_BASE);
                    }
                    
                }
            }
            IntentionCompleted::None => (),
        }
    }
}
