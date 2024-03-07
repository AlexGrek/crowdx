use comfy::Entity;

use crate::{behavior::{mental::IntentionCompleted, routine::gotoroutine::GoToRoutine, sanity::{Routine, Sanity, SelfAware}}, core::position::Ps, state::Reality};

#[derive(Debug, Clone)]
struct OfficeWorkerRoutine {
    walker_routine: GoToRoutine
}

#[derive(Debug)]
pub struct OfficeWorker {
    pub sa: SelfAware,
    pub initialized: bool,
    pub name: String
}

impl Routine for OfficeWorkerRoutine {
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
                
            }
            IntentionCompleted::None => (),
        }
    }
}

impl OfficeWorker {
    pub fn new(name: String, speed: f32, pos: Ps) -> Self {
        Self {
            name,
            initialized: false,
            sa: SelfAware::new(
                speed,
                pos,
                Box::new(OfficeWorkerRoutine {
                    walker_routine: GoToRoutine { target: pos },
                }),
            ),
        }
    }
}
