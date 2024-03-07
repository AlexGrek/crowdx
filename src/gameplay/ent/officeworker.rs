use crate::{
    behavior::{
        item_types::*,
        mental::{IntentionClass, IntentionCompleted, PRIORITY_BASE},
        routine::gotoroutine::GoToRoutine,
        sanity::{Routine, Sanity, SelfAware},
    },
    core::{position::{Ps, PsProvider}, Initializable},
    gameplay::gametime::Time,
    state::Reality,
};
use comfy::ChooseRandom;
use comfy::RandomRange;
use comfy::Entity;

#[derive(Debug, Clone)]
pub struct OfficeWorkerRoutine {
    walker_routine: GoToRoutine,
    pub assigned_bed: Option<Entity>,
    pub assigned_office: Option<Entity>,
}

#[derive(Debug)]
pub struct OfficeWorker {
    pub sa: SelfAware<OfficeWorkerRoutine>,
    pub initialized: bool,
    pub name: String,
}

fn is_time_to_work(time: &Time) -> bool {
    time.hours > 8 && time.hours < 18
}

fn is_time_to_sleep(time: &Time) -> bool {
    time.hours > 22 || time.hours < 7
}

impl Initializable for OfficeWorker {
    fn initialize(
        &mut self,
        _entity: &Entity,
        _transform: &mut comfy::Transform,
        reality: &mut Reality,
    ) {
        let mut locked = reality.interactive.lock();
        let possible: Vec<Entity> = locked
            .iter()
            .filter(|handle| handle.1.item_type == CONPUTER)
            .filter(|handle| handle.1.available())
            .map(|x| x.0.clone())
            .collect();
        if possible.len() < 1 {
            println!("Not enough conputers for Office Worker {:?}", _entity);
        }
        let handle = locked.get_mut(possible.choose().unwrap()).unwrap();
        self.assign_office(handle.item_id);
        handle.assign();

        let possible_bed: Vec<Entity> = locked
            .iter()
            .filter(|handle| handle.1.item_type == BED)
            .filter(|handle| handle.1.available())
            .map(|x| x.0.clone())
            .collect();
        if possible_bed.len() < 1 {
            println!("Not enough beds for Office Worker {:?}", _entity);
        }
        let handle_bed = locked.get_mut(possible_bed.choose().unwrap()).unwrap();
        self.assign_bed(handle_bed.item_id);
        handle_bed.assign();

        self.initialized = true;

        println!("{:?}", self);
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

fn do_chunk_of_work(sanity: &mut Sanity, chunk_size_minutes: isize, now: &Time) {
    sanity.intend_with_priority_timed(PRIORITY_BASE, IntentionClass::UseInteractiveMinutes(chunk_size_minutes), now)
}

fn do_chunk_of_sleep(sanity: &mut Sanity, chunk_size_hours: isize, now: &Time) {
    sanity.intend_with_priority_timed(PRIORITY_BASE, IntentionClass::UseInteractiveMinutes(chunk_size_hours * 60), now)
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
                if is_time_to_sleep(&map.time) {
                    println!("Sleep {:?}", entity);
                    match self.assigned_bed {
                        Some(bed) => {
                            let interative = map.interactive.lock();
                            let target = interative.get(&bed).unwrap();
                            self.walker_routine.target = target.get_interactive_ps();
                            sanity.reset_intentions();
                            if sanity.no_intentions_left() && sanity.get_current_ps() == self.walker_routine.target {
                                do_chunk_of_sleep(sanity, usize::gen_range(1, 2) as isize, &map.time)
                            }
                            self.walker_routine
                                .get_processing_fn(sanity, result, map, entity, dt);
                        }
                        None => println!("I want sleep, but no fucking bed!"),
                    }
                } else if is_time_to_work(&map.time) {
                    println!("Work {:?}", entity);
                    match self.assigned_office {
                        Some(office) => {
                            let interative = map.interactive.lock();
                            let target = interative.get(&office).unwrap();
                            self.walker_routine.target = target.get_interactive_ps();
                            sanity.reset_intentions();
                            if sanity.no_intentions_left() && sanity.get_current_ps() == self.walker_routine.target {
                                do_chunk_of_work(sanity, usize::gen_range(5, 15) as isize, &map.time)
                            }
                            self.walker_routine
                                .get_processing_fn(sanity, result, map, entity, dt);
                        }
                        None => println!("I want work, but no fucking office!"),
                    }
                }
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
                    assigned_bed: None,
                    assigned_office: None,
                }),
            ),
        }
    }

    pub fn assign_bed(&mut self, bed: Entity) {
        self.sa.routine.assigned_bed = Some(bed)
    }

    pub fn assign_office(&mut self, office: Entity) {
        self.sa.routine.assigned_office = Some(office)
    }
}
