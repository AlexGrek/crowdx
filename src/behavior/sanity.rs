use comfy::{Entity, Mutex, RandomRange};
use core::fmt::Debug;
use std::{borrow::BorrowMut, collections::LinkedList};

use crate::{
    core::position::{Ps, PsProvider}, gameplay::gametime::{Time, TimeSpan}, persistence::{self, Persistence}, state::Reality, worldmap::Cellmap
};

use super::{
    carriable::carriableitem::CarriableItems,
    carrier::Carrier,
    creatures::{validate_path, Direction, PsOffsetProvider, SelfRoutingData},
    mental::{Brains, Intention, IntentionClass, IntentionCompleted},
    routing::{try_find_route_from_to, PathfindRouter},
};

const DETOURS: [usize; 4] = [6, 8, 16, 32];

pub trait SaneMovingUnit {
    fn get_mind_mut(&mut self) -> &mut Brains;
    fn get_mv_mut(&mut self) -> &mut SelfRoutingData;
}

#[derive(Debug)]
pub struct SelfAware<T: Routine> {
    pub routine: Box<T>,
    pub sanity: Mutex<Sanity>,
}

impl<T: Routine> SelfAware<T> {
    pub fn new(speed: f32, pos: Ps, routine: Box<T>) -> Self {
        let sanity = Sanity::new(speed, pos);
        Self {
            routine,
            sanity: Mutex::new(sanity),
        }
    }

    pub fn think_routine_level(
        &mut self,
        result: IntentionCompleted,
        map: &Reality,
        entity: Entity,
        dt: f32,
    ) {
        // println!("Thinking routine level after {:?}", result);
        self.routine
            .get_processing_fn(self.sanity.lock().borrow_mut(), result, map, entity, dt)
    }
}

impl<T: Routine> PsOffsetProvider for SelfAware<T> {
    fn get_ps(&self) -> Ps {
        self.sanity.lock().get_current_ps()
    }

    fn get_offset(&self) -> comfy::Vec2 {
        self.sanity.lock().mv.movement.loc.offset
    }
}

#[derive(Debug)]
pub struct Sanity {
    pub mind: Brains,
    pub mv: SelfRoutingData,
    pub carrier: Carrier,
    // routine: Arc<Mutex<Box<dyn Routine<T>>>>,
}

pub trait Routine: Debug + Send + Sync {
    fn get_processing_fn(
        &mut self,
        sane: &mut Sanity,
        result: IntentionCompleted,
        map: &Reality,
        entity: Entity,
        dt: f32,
    );
}

#[derive(Debug)]
pub struct NoRoutine;

impl Routine for NoRoutine {
    fn get_processing_fn(
        &mut self,
        _: &mut Sanity,
        _result: IntentionCompleted,
        _map: &Reality,
        _entity: Entity,
        _dt: f32,
    ) {
        return ();
    }
}

#[derive(Debug)]
pub struct GoToRandomFreePsRoutine;

impl Sanity {
    pub fn new(speed: f32, pos: Ps) -> Self {
        Self {
            mv: SelfRoutingData::new(speed, pos),
            mind: Brains::new(),
            carrier: Carrier::new(),
            // routine: Arc::new(Mutex::new(routine)),
        }
    }

    pub fn finish_current_intention(&mut self, success: bool) -> bool {
        if let Some(current) = self.mind.intentions.get_current() {
            self.mind.remember_intention_result(current.value, success)
        }
        self.mind.intentions.finish_current()
    }

    pub fn finish_current_intention_with(
        &mut self,
        result: IntentionCompleted,
    ) -> IntentionCompleted {
        if result == IntentionCompleted::Success {
            if let Some(current) = self.mind.intentions.get_current() {
                self.mind.remember_intention_result(current.value, true)
            }
        }
        if result == IntentionCompleted::Failure {
            if let Some(current) = self.mind.intentions.get_current() {
                self.mind.remember_intention_result(current.value, false)
            }
        }

        self.mind.intentions.finish_current();
        result
    }

    pub fn no_intentions_left(&self) -> bool {
        return self.mind.intentions.intentions_left() == 0;
    }

    pub fn reset_intentions(&mut self) {
        self.mind.intentions.clear_all()
    }

    pub fn reset_intentions_lower_than(&mut self, priority: i32) {
        self.mind.intentions.clear_lower_than(priority)
    }

    pub fn intend_with_priority(&mut self, priority: i32, intention: IntentionClass) {
        self.mind
            .intentions
            .intend(Intention::new(priority, intention))
    }

    pub fn intend_with_priority_timed(
        &mut self,
        priority: i32,
        intention: IntentionClass,
        time: &Time,
    ) {
        self.mind.save_time(time);
        self.intend_with_priority(priority, intention);
    }

    pub fn intend(&mut self, intention: IntentionClass) {
        self.intend_with_priority(0, intention)
    }

    pub fn intend_go_to_around(&mut self, target: Ps) {
        self.intend(IntentionClass::MoveToDestination(target))
    }

    pub fn intend_go_to(&mut self, target: Ps) {
        self.intend(IntentionClass::MoveToPs(target))
    }

    fn follow_direction_until_next_cell(&mut self, dt: f32) -> Option<Ps> {
        self.mv.follow_direction_until_stop(dt)
    }

    pub fn start_move_direction(&mut self, dir: Direction, cellmap: &Cellmap) {
        if cellmap.xy_within_bounds(&self.mv.predict_next_pos(dir)) {
            self.mv.start_move_direction(dir)
        }
    }

    fn try_pick_item(
        &mut self,
        entity: Entity,
        carriables: &CarriableItems,
        item_type: Option<&'static str>,
    ) -> IntentionCompleted {
        let position = self.get_current_ps();
        let mut items = carriables.lock();
        for (_, item) in items.iter_mut() {
            if item.get_ps() == position && item.available() {
                if item_type.is_none() || item_type.is_some_and(|tp| item.item_type == tp) {
                    self.carrier.pick_up(entity, item);
                    // println!("Item {:?} taken by {:?}", item_entity, entity);
                    return IntentionCompleted::Success;
                }
            }
        }
        // println!("{:?} failed to take an item", entity);
        IntentionCompleted::Failure
    }

    fn try_consume_item(
        &mut self,
        entity: Entity,
        carriables: &CarriableItems,
        item_type: Option<&'static str>,
    ) -> IntentionCompleted {
        let position = self.get_current_ps();
        let mut items = carriables.lock();
        // println!("Trying to consume item at {:?}", position);
        for (_, item) in items.iter_mut() {
            if item.get_ps() == position && item.available() {
                if item_type.is_none() || item_type.is_some_and(|tp| item.item_type == tp) {
                    item.consume();
                    // println!("Item {:?} consumed by {:?}", item_entity, entity);
                    return IntentionCompleted::Success;
                }
            }
        }
        println!("{:?} failed to take an item", entity);
        IntentionCompleted::Failure
    }

    fn try_drop_item(
        &mut self,
        entity: Entity,
        carriables: &CarriableItems,
        item_type: Option<&'static str>,
    ) -> IntentionCompleted {
        // let position = self.get_current_ps();
        let mut items = carriables.lock();
        // println!("Trying to drop item at {:?}", position);
        for (_, item) in items.iter_mut() {
            if item_type.is_none() || item_type.is_some_and(|tp| item.item_type == tp) {
                let dropped = self.carrier.try_drop(item);
                if dropped {
                    return IntentionCompleted::Success;
                }
            }
        }
        println!("{:?} failed to drop an item", entity);
        IntentionCompleted::Failure
    }

    fn try_consume_carried_item(
        &mut self,
        entity: Entity,
        carriables: &CarriableItems,
        item_type: Option<&'static str>,
    ) -> IntentionCompleted {
        // let position = self.get_current_ps();
        let mut items = carriables.lock();
        // println!("Trying to drop item at {:?}", position);
        for (_, item) in items.iter_mut() {
            if item_type.is_none() || item_type.is_some_and(|tp| item.item_type == tp) {
                let done = self.carrier.try_consume(item);
                if done {
                    return IntentionCompleted::Success;
                }
            }
        }
        println!("{:?} failed to drop an item", entity);
        IntentionCompleted::Failure
    }

    fn think_intentions_level(&mut self, entity: Entity, reality: &Reality) -> IntentionCompleted {
        // println!("Processing intentions:\n   {:?}\n   {:?}", self.mind.intentions, self.get_current_ps());
        if let Some(current_intention) = self.mind.intentions.get_current() {
            match current_intention.value {
                IntentionClass::MoveToDestination(dest) => {
                    return self.process_destination_intention(dest, &reality.cellmap, false);
                }
                IntentionClass::MoveToPs(dest) => {
                    return self.process_destination_intention(dest, &reality.cellmap, true);
                }
                IntentionClass::WaitCycles(_) => {
                    // print!(">");
                    if self.mind.count_cycles() {
                        self.finish_current_intention(true);
                        return IntentionCompleted::Success;
                    }
                }
                IntentionClass::PickItemOfType(item_type) => {
                    let result = self.try_pick_item(
                        entity,
                        &reality.carriables,
                        Some(item_type),
                    );
                    return self.finish_current_intention_with(result);
                }
                IntentionClass::PickAnyItem() => {
                    let result = self.try_pick_item(
                        entity,
                        &reality.carriables,
                        None,
                    );
                    return self.finish_current_intention_with(result);
                }
                IntentionClass::ConsumeItemOfType(item_type) => {
                    let result = self.try_consume_item(
                        entity,
                        &reality.carriables,
                        Some(item_type),
                    );
                    return self.finish_current_intention_with(result);
                }
                IntentionClass::DropCarriedItemOfType(item_type) => {
                    let result = self.try_drop_item(
                        entity,
                        &reality.carriables,
                        Some(item_type),
                    );
                    return self.finish_current_intention_with(result);
                }
                IntentionClass::ConsumeAnyItem() => {
                    let result =self.try_consume_item(
                        entity,
                        &reality.carriables,
                        None,
                    );
                    return self.finish_current_intention_with(result);
                }
                IntentionClass::DropAnyCarriedItem() => {
                    let result = self.try_drop_item(
                        entity,
                        &reality.carriables,
                        None,
                    );
                    return self.finish_current_intention_with(result);
                }
                IntentionClass::ConsumeCarriedItemOfType(item_type) => {
                    let result = self.try_consume_carried_item(
                        entity,
                        &reality.carriables,
                        Some(item_type),
                    );
                    return self.finish_current_intention_with(result);
                }
                IntentionClass::ConsumeAnyCarriedItem() => {
                    let result = self.try_consume_carried_item(
                        entity,
                        &reality.carriables,
                        None,
                    );
                    return self.finish_current_intention_with(result);
                }
                IntentionClass::UseInteractiveOnce() => todo!(),
                IntentionClass::UseInteractiveCycles(_) => todo!(),
                IntentionClass::UseInteractiveMinutes(minutes) => {
                    return self.use_interactive_minutes(entity, minutes, reality)
                }
            }
        }
        IntentionCompleted::Undefined
    }

    fn try_use_interactive(
        &mut self,
        entity: Entity,
        reality: &Reality,
    ) -> Option<IntentionCompleted> {
        let position = self.get_current_ps();
        let mut items = reality.interactive.lock();
        for (_, item) in items.iter_mut() {
            if item.get_interactive_ps() == position {
                if let Some(user) = item.used_by {
                    if user != entity {
                        println!(
                            "WARN: IItem {:?} already in use, cannot use by {:?}",
                            item, entity
                        );
                        return Some(IntentionCompleted::Failure);
                    }
                    return None;
                }
                item.use_by(entity);
                // println!("IItem {:?} is now used by {:?}", item, entity);
                return None;
            }
        }
        println!("ERROR: Not found any interactive object in position {:?}", position);
        return Some(IntentionCompleted::Failure);
    }

    fn try_release_interactive(&mut self, entity: Entity, reality: &Reality) -> IntentionCompleted {
        let position = self.get_current_ps();
        let mut items = reality.interactive.lock();
        for (_, item) in items.iter_mut() {
            if item.get_interactive_ps() == position {
                if let Some(user) = item.used_by {
                    if user != entity {
                        println!(
                            "WARN: IItem {:?} already in use BY SOMEONE ELSE, cannot release by {:?}",
                            item, entity
                        );
                        return IntentionCompleted::Failure;
                    } else {
                        item.release();
                        // println!("IItem {:?} is released by {:?}", item, entity);
                        return IntentionCompleted::Success;
                    }
                }
            }
        }
        println!("ERROR: Not found any interactive object to release in position {:?}", position);
        IntentionCompleted::Failure
    }

    fn use_interactive_minutes(
        &mut self,
        entity: Entity,
        minutes: isize,
        reality: &Reality,
    ) -> IntentionCompleted {
        if self.mind.time_reference.elapsed(&reality.time) >= TimeSpan::new(minutes) {
            let result = self.try_release_interactive(entity, reality);
            self.finish_current_intention_with(result)
        } else {
            self.try_use_interactive(entity, reality)
                .unwrap_or(IntentionCompleted::None)
        }
    }

    fn process_destination_intention(
        &mut self,
        dest: Ps,
        cellmap: &Cellmap,
        exact: bool,
    ) -> IntentionCompleted {
        let required_distance = if exact { 0 } else { 1 };
        let restart_intention = match self.mv.current_move_path.target {
            Some(tgt) => dest.manhattan_distance(&tgt) > required_distance,
            None => true,
        };
        if restart_intention {
            // print!("!");
            // println!("Intention procesing: move to {:?}", dest);
            let possible = if !exact {
                self.move_to_ps_or_around_1(cellmap, dest)
            } else {
                self.move_to_ps(cellmap, dest)
            };
            if !possible {
                // print!("X");
                // destination is unreachable
                // println!("Cannot move to destination");
                self.finish_current_intention(false);
                self.mv.stop_moving();
                return IntentionCompleted::Failure;
            }
            // println!(
            //     "FULL PTHFIND APPLIED from {:?} to {:?}: {:?}",
            //     self.get_current_ps(),
            //     dest,
            //     self.mv.current_move_path
            // );
        }
        let reached = dest.manhattan_distance(&self.get_current_ps()) <= required_distance
            || self.mv.is_reached_cell_flag_set();
        if reached {
            self.finish_current_intention(true);
            // print!("Y");
            return IntentionCompleted::Success;
        }
        // println!(
        //     "\n\n{:?} ({:?}) -> {:?}\n",
        //     self.is_next_step_valid(cellmap),
        //     self.mv.reliable_steps_left,
        //     self.mv.current_move_path
        // );
        if !self.is_next_step_valid(cellmap) {
            let detour_successful: bool;
            if self.mv.is_no_reliabler_steps_left() {
                // Do NOT do detours if just reliable steps ended
                // force full path recalculation
                detour_successful = self.force_recalculate_path_to_target(cellmap);
                // and immediately stop intention if failed
                if !detour_successful {
                    self.mv.stop_moving();
                    return IntentionCompleted::None;
                }
            } else {
                detour_successful = self.try_multiple_detours(cellmap);
            }
            // no detours or possible path
            if !detour_successful {
                self.mv.stop_moving();
                self.mind
                    .intend_cycles_count(i32::gen_range(10, 60) as isize, 100)
            }

            return IntentionCompleted::None;
        }
        return IntentionCompleted::None;
    }

    fn try_multiple_detours(&mut self, cellmap: &Cellmap) -> bool {
        for detour_len in DETOURS {
            if self.try_small_detour(cellmap, detour_len) {
                return true;
            }
        }
        false
    }

    fn force_recalculate_path_to_target(&mut self, cellmap: &Cellmap) -> bool {
        if self.mv.current_move_path.target.is_none() {
            return false;
        }
        match try_find_route_from_to(
            cellmap,
            true,
            self.get_current_ps(),
            self.mv.current_move_path.target.unwrap(),
        ) {
            Some(path) => {
                self.follow_steps(self.mv.current_move_path.target.unwrap(), path);
                true
            }
            None => false,
        }
    }

    fn try_small_detour(&mut self, cellmap: &Cellmap, detour_dist: usize) -> bool {
        if let Some(following_step) = self.mv.peek_close_step_or_final_step(detour_dist) {
            let (position_target, position_index) = following_step;
            if position_target == self.get_current_ps() {
                println!("WARN: trying to find path to same position {:?} in detour [dist = {}, asked dist = {}]...\n{:?}", position_target, position_index, detour_dist, self.mv.current_move_path);
                return false;
            }
            return match try_find_route_from_to(
                cellmap,
                true,
                self.get_current_ps(),
                position_target,
            ) {
                Some(path) => {
                    if path.len() == 0 {
                        println!(
                            "0 length path found from {:?} to {:?} [dist = {}, asked dist = {}]",
                            self.get_current_ps(),
                            position_target,
                            position_index,
                            detour_dist
                        );
                        panic!("Path is 0 length")
                    }
                    // print!("{}", position_index);
                    let target = path.back().unwrap();
                    if target != &position_target {
                        println!(
                            "target {:?} is not equal position_target {:?}",
                            target, position_target
                        );
                        panic!(
                            "target {:?} is not equal position_target {:?}",
                            target, position_target
                        );
                    }
                    if let Some(invalid) = validate_path(&path) {
                        println!("\n\nnvalid path part detected: {:?}", invalid);
                        println!("\nFull path: {:?}", &path);
                    }
                    // println!("--------> Calculated subpath: {:?}", path);
                    // println!("Full path was  : {:?}", self.mv.current_move_path.calculated_steps);
                    self.mv.change_near_movement_path(&path, position_index);
                    if let Some(_invalid) =
                        validate_path(&self.mv.current_move_path.calculated_steps)
                    {
                        // println!("\n\nnvalid path detected: {:?}", invalid);
                        println!(
                            "\nFull path: {:?}",
                            self.mv.current_move_path.calculated_steps
                        );
                    }
                    // println!("Full path after: {:?}", self.mv.current_move_path.calculated_steps);
                    true
                }
                None => {
                    // print!("-");
                    false
                }
            };
        } else {
            // no steps found
            // println!("ERRRR: {:?}", self.mv.current_move_path.calculated_steps);
            false
        }
    }

    fn is_next_step_valid(&self, cellmap: &Cellmap) -> bool {
        if self.mv.is_no_reliabler_steps_left() {
            return false;
        }
        let next_possible_step = self
            .mv
            .peek_next_loc()
            .map(|loc| cellmap.get_pos(loc).is_passable(true))
            .unwrap_or(true);
        return next_possible_step;
    }

    fn think_movement_level(&mut self, cellmap: &mut Cellmap) -> bool {
        let mut movement_dest_reached = false;
        // assume this is valid step now
        let prev_position = self.get_current_ps();
        let next = self.mv.step_next_direction();
        if let Some(_found) = next {
            movement_dest_reached = self.mv.stop_if_destination_cell_reached();
        }
        cellmap.move_occupy_ps(&prev_position, &self.get_current_ps());
        movement_dest_reached
    }

    pub fn move_direction_if_can(&mut self, dt: f32) -> bool {
        match self.mv.movement.loc.direction {
            Some(_direction) => {
                // Just stupid movement, cannot change anything while moving
                self.just_move_direction(dt);
                true
            }
            None => {
                // Not moving, so do nothing
                false
            }
        }
    }

    fn just_move_direction(&mut self, dt: f32) {
        self.follow_direction_until_next_cell(dt);
    }

    pub fn think_intention_level_if_not_moving(
        &mut self,
        entity: Entity,
        reality: &Reality,
    ) -> IntentionCompleted {
        match self.mv.movement.loc.direction {
            Some(_) => IntentionCompleted::None,
            None =>
            // Not moving, so we have a frame to think
            {
                self.think_intentions_level(entity, reality)
            }
        }
    }

    pub fn think_movement_level_if_not_moving(&mut self, cellmap: &mut Cellmap) -> bool {
        match self.mv.movement.loc.direction {
            Some(_) => false,
            None =>
            // Not moving, so we have a frame to think
            {
                self.think_movement_level(cellmap)
            }
        }
    }
}

impl PsProvider for Sanity {
    fn get_current_ps(&self) -> Ps {
        return self.mv.get_current_ps();
    }
}

impl PathfindRouter for Sanity {
    fn follow_steps(&mut self, target: Ps, steps: LinkedList<Ps>) {
        if let Some(invalid) = validate_path(&steps) {
            println!("\n\nInvalid FULL path part detected: {:?}", invalid);
            println!("\nFull path: {:?}", &steps);
        }
        self.mv.set_movement_path(steps, target);
    }
}
