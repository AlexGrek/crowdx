use crate::{
    behavior::{
        item_types::*, mental::{IntentionClass, IntentionCompleted, PRIORITY_BASE}, messaging::communication::Communicator, routine::{gotoroutine::GoToRoutine, randomwalk::RandomStepRoutine}, sanity::{Routine, Sanity, SelfAware}
    },
    core::{
        position::{Ps, PsProvider},
        Initializable,
    },
    gameplay::{gametime::Time, humanclothes::Look},
    state::Reality,
};
use comfy::ChooseRandom;
use comfy::Entity;
use comfy::RandomRange;

#[derive(Debug, Clone)]
pub struct OfficeWorkerRoutine {
    walker_routine: GoToRoutine,
    pub assigned_bed: Option<Ps>,
    pub assigned_office: Option<Ps>,
    pub visible_entities: Vec<Entity> 
}

#[derive(Debug)]
pub struct OfficeWorker {
    pub sa: SelfAware<OfficeWorkerRoutine>,
    pub initialized: bool,
    pub name: String,
    pub look: Look,
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
        entity: &Entity,
        _transform: &mut comfy::Transform,
        reality: &mut Reality,
    ) {
        let mut locked = reality.interactive.lock();
        let possible: Vec<Ps> = locked
            .iter()
            .filter(|handle| handle.1.item_type == CONPUTER)
            .filter(|handle| handle.1.available())
            .map(|x| x.0.clone())
            .collect();
        if possible.len() < 1 {
            println!("Not enough conputers for Office Worker {:?}", entity);
        }
        let handle = locked.get_mut(possible.choose().unwrap()).unwrap();
        self.assign_office(handle.get_interactive_ps());
        handle.assign();

        let possible_bed: Vec<Ps> = locked
            .iter()
            .filter(|handle| handle.1.item_type == BED)
            .filter(|handle| handle.1.available())
            .map(|x| x.0.clone())
            .collect();
        if possible_bed.len() < 1 {
            println!("Not enough beds for Office Worker {:?}", entity);
        }
        let handle_bed = locked.get_mut(possible_bed.choose().unwrap()).unwrap();
        self.assign_bed(handle_bed.get_interactive_ps());
        handle_bed.assign();

        // init body parts
        self.look.spawn_for_entity(entity.clone());

        self.initialized = true;

        println!("{:?}", self);
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

fn do_chunk_of_work(sanity: &mut Sanity, chunk_size_minutes: isize, now: &Time) {
    sanity.intend_with_priority_timed(
        PRIORITY_BASE,
        IntentionClass::UseInteractiveMinutes(chunk_size_minutes),
        now,
    )
}

fn do_chunk_of_sleep(sanity: &mut Sanity, chunk_size_hours: isize, now: &Time) {
    sanity.intend_with_priority_timed(
        PRIORITY_BASE,
        IntentionClass::UseInteractiveMinutes(chunk_size_hours * 60),
        now,
    )
}

impl OfficeWorkerRoutine {
    fn sleep(
        &mut self,
        sanity: &mut Sanity,
        map: &Reality,
        entity: Entity,
        result: IntentionCompleted,
        communication: &Communicator,
        dt: f32,
    ) {
        match self.assigned_bed {
            Some(bed) => {
                let interative = map.interactive.lock();
                let target = interative.get(&bed).unwrap();
                self.walker_routine.target = target.get_interactive_ps();
                sanity.reset_intentions();
                if sanity.no_intentions_left()
                    && sanity.get_current_ps() == self.walker_routine.target
                {
                    do_chunk_of_sleep(sanity, usize::gen_range(1, 2) as isize, &map.time)
                }
                self.walker_routine
                    .get_processing_fn(sanity, result, map, entity, communication, dt);
            }
            None => println!("I want sleep, but no fucking bed!"),
        }
    }

    fn work(
        &mut self,
        sanity: &mut Sanity,
        map: &Reality,
        entity: Entity,
        result: IntentionCompleted,
        communication: &Communicator,
        dt: f32,
    ) {
        match self.assigned_office {
            Some(office) => {
                let interative = map.interactive.lock();
                // println!("Assigned: {:?}; all: {:?}", self.assigned_office, interative.keys());
                let target = interative.get(&office).unwrap();
                self.walker_routine.target = target.get_interactive_ps();
                sanity.reset_intentions();
                if sanity.no_intentions_left()
                    && sanity.get_current_ps() == self.walker_routine.target
                {
                    do_chunk_of_work(sanity, usize::gen_range(5, 15) as isize, &map.time)
                }
                self.walker_routine
                    .get_processing_fn(sanity, result, map, entity, communication, dt);
            }
            None => println!("I want work, but no fucking office!"),
        }
    }
}

impl Routine for OfficeWorkerRoutine {
    fn get_processing_fn(
        &mut self,
        sanity: &mut Sanity,
        result: IntentionCompleted,
        map: &Reality,
        entity: Entity,
        communication: &Communicator,
        dt: f32,
    ) {
        self.visible_entities = communication.find_visible_entities(&map);
        match result {
            IntentionCompleted::Success
            | IntentionCompleted::Failure
            | IntentionCompleted::Undefined => {
                if is_time_to_sleep(&map.time) {
                    // println!("Sleep {:?} (was {:?})", entity, sanity.mind.mem);
                    self.sleep(sanity, map, entity, result, communication, dt);
                    return;
                }
                if is_time_to_work(&map.time) {
                    // println!("Work {:?} (was {:?})", entity, sanity.mind.mem);
                    self.work(sanity, map, entity, result, communication, dt);
                    return;
                }
                RandomStepRoutine.get_processing_fn(sanity, result, map, entity, communication, dt);
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
                    visible_entities: Vec::new()
                }),
            ),
            look: Look::new(),
        }
    }

    pub fn assign_bed(&mut self, bed: Ps) {
        self.sa.routine.assigned_bed = Some(bed)
    }

    pub fn assign_office(&mut self, office: Ps) {
        self.sa.routine.assigned_office = Some(office)
    }
}
