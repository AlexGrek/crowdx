use std::collections::LinkedList;

use crate::core::position::{Ps, PsProvider, PsSigned};
use comfy::RandomRange;
use comfy::{num_traits::ToPrimitive, Vec2};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct MovementIntention {
    pub target: Option<Ps>,
    pub calculated_steps: LinkedList<Ps>,
    pub stuck: bool,
}

impl MovementIntention {
    pub fn exists(&self) -> bool {
        return self.target.is_some();
    }

    pub fn stuck(&mut self) {
        self.stuck = true
    }

    pub fn unstuck(&mut self) {
        self.stuck = false
    }

    fn remove_front(&mut self, remove_all_including: usize) {
        // println!("remove_front ({})\nPath: {:?}", remove_all_including, self.calculated_steps);
        for _ in 0..remove_all_including {
            self.calculated_steps.pop_front();
            // println!("Pop next loc (front), was: {:?}, left: {}", popd, self.calculated_steps.len());
        }
        // println!("Result: {:?}", self.calculated_steps);
    }

    fn prepend(&mut self, path: &LinkedList<Ps>) {
        for item in path.into_iter().rev() {
            self.calculated_steps.push_front(item.to_owned())
        }
    }

    fn replace_front(&mut self, path: &LinkedList<Ps>, remove_all_including: usize) {
        // println!("replace_front ({}:{:?}, {})\nPath: {:?}", path.len(), path, remove_all_including, self.calculated_steps);
        self.remove_front(remove_all_including);
        self.prepend(path);
        // println!("Result: {:?}", self.calculated_steps);
    }
}

#[derive(Debug)]
pub struct MovingObjectData {
    pub pos: Ps,
    pub offset: Vec2,
    pub direction: Option<Direction>,
}

impl crate::core::position::PsProvider for MovingObjectData {
    fn get_current_ps(&self) -> Ps {
        self.pos
    }
}

impl MovingObjectData {
    pub fn get_exact_pos(&self) -> Vec2 {
        let global_pos: Vec2 = self.pos.into();
        self.offset + global_pos
    }

    fn check_movement_stop(&mut self, prop: f32, negative: bool) -> Option<Ps> {
        let compare = if !negative { -0.01 } else { 0.01 };
        let mut reset = false;
        if negative && prop < compare {
            // very interesting things happen here, as we don't need to move anymore
            reset = true;
        }
        if !negative && prop > compare {
            // very interesting things happen here, as we don't need to move anymore
            reset = true;
        }
        if reset {
            self.direction = None;
            self.offset = Vec2::new(0.0, 0.0);
            return Some(self.pos.clone());
        } else {
            return None;
        }
    }

    pub fn follow_direction_until_stop(&mut self, ds: f32) -> Option<Ps> {
        match self.direction {
            Some(Direction::Up) => {
                self.offset.y += ds;
                self.check_movement_stop(self.offset.y, false)
            }
            Some(Direction::Down) => {
                self.offset.y -= ds;
                self.check_movement_stop(self.offset.y, true)
            }
            Some(Direction::Left) => {
                self.offset.x -= ds;
                self.check_movement_stop(self.offset.x, true)
            }
            Some(Direction::Right) => {
                self.offset.x += ds;
                self.check_movement_stop(self.offset.x, false)
            }
            None => None,
        }
    }

    pub fn start_move_direction(&mut self, dir: Direction) {
        match dir {
            Direction::Down => self.pos.y -= 1,
            Direction::Up => self.pos.y += 1,
            Direction::Left => self.pos.x -= 1,
            Direction::Right => self.pos.x += 1,
        }
        match dir {
            Direction::Down => self.offset.y = 1.0,
            Direction::Up => self.offset.y = -1.0,
            Direction::Left => self.offset.x = 1.0,
            Direction::Right => self.offset.x = -1.0,
        }
        self.direction = Some(dir)
    }
}

#[derive(Debug)]
pub struct SelfMovingThingData {
    pub loc: MovingObjectData,
    pub speed: f32,
}

impl crate::core::position::PsProvider for SelfMovingThingData {
    fn get_current_ps(&self) -> Ps {
        return self.loc.get_current_ps();
    }
}

impl SelfMovingThingData {
    pub fn get_exact_pos(&self) -> Vec2 {
        self.loc.get_exact_pos()
    }

    pub fn follow_direction_until_stop(&mut self, dt: f32) -> Option<Ps> {
        let ds = self.speed * dt;
        self.loc.follow_direction_until_stop(ds)
    }

    pub fn start_move_direction(&mut self, dir: Direction) {
        self.loc.start_move_direction(dir)
    }
}

#[derive(Debug)]
pub struct SelfRoutingData {
    pub movement: SelfMovingThingData,
    pub current_move_path: MovementIntention,
    pub reliable_steps_left: isize,
    reached_cell: bool,
}

impl crate::core::position::PsProvider for SelfRoutingData {
    fn get_current_ps(&self) -> Ps {
        return self.movement.get_current_ps();
    }
}

impl PsOffsetProvider for SelfRoutingData {
    fn get_ps(&self) -> Ps {
        self.get_current_ps()
    }

    fn get_offset(&self) -> comfy::Vec2 {
        self.movement.loc.offset
    }
}

pub fn try_get_direction_from_to(from: Ps, to: Ps) -> Option<Direction> {
    match to - from {
        PsSigned { x: 0, y } => {
            if y > 0 {
                Some(Direction::Up)
            } else {
                Some(Direction::Down)
            }
        }
        PsSigned { y: 0, x } => {
            if x > 0 {
                Some(Direction::Right)
            } else {
                Some(Direction::Left)
            }
        }
        PsSigned { x, y } => {
            println!(
                "ERROR: Direction difference is {:?}, {:?}, sad but true [ from:{:?} to:{:?} ]",
                x, y, from, to
            );
            panic!("Direction difference");
            // None
        }
    }
}

pub fn get_direction_from_to(from: Ps, to: Ps) -> Direction {
    try_get_direction_from_to(from, to).unwrap()
}

pub fn validate_path(path: &LinkedList<Ps>) -> Option<(Ps, Ps, usize)> {
    for i in 1..path.len() {
        let prev = path.iter().nth(i - 1).unwrap();
        let current = path.iter().nth(i).unwrap();
        if prev.manhattan_distance(current) != 1 {
            return Some((prev.to_owned(), current.to_owned(), i));
        }
    }
    None
}

impl SelfRoutingData {
    pub fn as_ps_offset_container(&self) -> PsOffsetContainer {
        PsOffsetContainer {
            pos: self.get_current_ps(),
            offset: self.get_offset(),
        }
    }

    pub fn get_next_direction(&self, to: Ps) -> Direction {
        get_direction_from_to(self.movement.loc.pos, to)
    }

    pub fn try_get_next_direction(&self, to: Ps) -> Option<Direction> {
        try_get_direction_from_to(self.movement.loc.pos, to)
    }

    pub fn peek_next_loc(&self) -> Option<&Ps> {
        if self.current_move_path.exists() {
            self.current_move_path.calculated_steps.front()
        } else {
            None
        }
    }

    pub fn stop_moving(&mut self) {
        self.current_move_path.target = None;
        self.current_move_path.calculated_steps = LinkedList::new();
        self.reached_cell = false
    }

    pub fn stuck(&mut self) {
        self.current_move_path.stuck()
    }

    pub fn unstuck(&mut self) {
        self.current_move_path.unstuck()
    }

    pub fn is_stuck_flag(&mut self) -> bool {
        self.current_move_path.stuck
    }

    pub fn consume_next_loc(&mut self) -> Option<Ps> {
        if self.current_move_path.exists() {
            self.reached_cell = false;
            self.unstuck();
            let popd = self.current_move_path.calculated_steps.pop_front();
            // println!("Pop next loc, left: {}", self.current_move_path.calculated_steps.len());
            popd
        } else {
            None
        }
    }

    pub fn step_next_direction(&mut self) -> Option<Direction> {
        match self.consume_next_loc() {
            None => None,
            Some(loc) => {
                if let Some(direction) = self.try_get_next_direction(loc) {
                    // we have next direction to move
                    self.unstuck();
                    self.reliable_steps_left -= 1;
                    self.start_move_direction(direction);
                    self.movement.loc.pos = loc;
                    Some(direction)
                } else {
                    println!(
                        "Cannot find direction from {:?} to {:?}",
                        self.movement.loc.pos, loc
                    );
                    None
                }
            }
        }
    }

    fn is_destination_cell_reached(&self) -> bool {
        match self.current_move_path.target {
            None => true, // always reach destination if we have no destination
            Some(dest) => self.movement.loc.pos == dest,
        }
    }

    pub fn stop_if_destination_cell_reached(&mut self) -> bool {
        if self.is_destination_cell_reached() {
            // println!("Destination cell reached");
            self.stop_moving();
            self.reached_cell = true;
            self.reliable_steps_left = 0;
            return true;
        }
        return false;
    }

    pub fn new(speed: f32, initial_position: Ps) -> Self {
        SelfRoutingData {
            movement: SelfMovingThingData {
                loc: MovingObjectData {
                    pos: initial_position,
                    offset: (0.0, 0.5).into(),
                    direction: None,
                },
                speed: speed,
            },
            current_move_path: MovementIntention {
                target: None,
                calculated_steps: LinkedList::new(),
                stuck: false,
            },
            reliable_steps_left: 0,
            reached_cell: true,
        }
    }

    fn calculate_reliable_steps(&self, max: usize) -> isize {
        if max < 4 {
            max.to_isize().unwrap()
        } else {
            isize::min(
                (((max as f64) / 2.0) * f64::gen_range(1.0, 2.0))
                    .floor()
                    .to_isize()
                    .unwrap_or(max.to_isize().unwrap()),
                i32::gen_range(20, 80).to_isize().unwrap(),
            )
        }
    }

    pub fn peek_close_step_or_final_step(&self, min: usize) -> Option<(Ps, usize)> {
        // println!("peek_close_step_or_final_step:\n len:{:?}, back:{:?}", self.current_move_path.calculated_steps.len(), self.current_move_path.calculated_steps.back());
        if self.current_move_path.calculated_steps.len() > min {
            // println!("self.current_move_path.calculated_steps.len() > min-1");
            self.current_move_path
                .calculated_steps
                .iter()
                .nth(min - 1)
                .map(|stp| (stp.clone(), min))
        } else {
            // println!("self.current_move_path.calculated_steps.len() <= min-1");
            self.current_move_path
                .calculated_steps
                .back()
                .map(|stp| (stp.clone(), self.current_move_path.calculated_steps.len()))
        }
    }

    pub fn set_movement_path(&mut self, path: LinkedList<Ps>, target: Ps) {
        self.reliable_steps_left = self.calculate_reliable_steps(path.len());
        self.reached_cell = false;
        self.current_move_path = MovementIntention {
            calculated_steps: path,
            target: Some(target),
            stuck: false,
        }
    }

    pub fn change_near_movement_path(
        &mut self,
        path: &LinkedList<Ps>,
        remove_all_including: usize,
    ) {
        self.reached_cell = false;
        self.current_move_path
            .replace_front(path, remove_all_including);
        self.reliable_steps_left =
            self.calculate_reliable_steps(self.current_move_path.calculated_steps.len());
    }

    pub fn is_no_reliabler_steps_left(&self) -> bool {
        if self.current_move_path.calculated_steps.len() < 3 {
            // if 2 or less steps left - don't care about reliable steps
            return true;
        }
        if self.reliable_steps_left <= 0 {
            // println!("No reliable steps left");
            return true;
        }
        // still have some
        false
    }

    pub fn get_exact_pos(&self) -> Vec2 {
        self.movement.get_exact_pos()
    }

    pub fn follow_direction_until_stop(&mut self, dt: f32) -> Option<Ps> {
        self.movement.follow_direction_until_stop(dt)
    }

    pub fn start_move_direction(&mut self, dir: Direction) {
        self.unstuck();
        self.movement.start_move_direction(dir)
    }

    pub fn predict_next_pos(&self, dir: Direction) -> PsSigned {
        let mut pos: PsSigned = self.movement.loc.pos.into();
        match dir {
            Direction::Down => pos.y -= 1,
            Direction::Up => pos.y += 1,
            Direction::Left => pos.x -= 1,
            Direction::Right => pos.x += 1,
        }
        pos
    }

    pub fn is_reached_cell_flag_set(&self) -> bool {
        self.reached_cell
    }
}

pub trait PsOffsetProvider {
    fn get_ps(&self) -> Ps;
    fn get_offset(&self) -> comfy::Vec2;

    fn get_exact_pos(&self) -> Vec2 {
        let global_pos: Vec2 = self.get_ps().into();
        self.get_offset() + global_pos
    }
}

#[derive(Clone, Debug)]
pub struct PsOffsetContainer {
    pub pos: Ps,
    pub offset: Vec2,
}

impl PsOffsetProvider for PsOffsetContainer {
    fn get_ps(&self) -> Ps {
        self.pos.clone()
    }

    fn get_offset(&self) -> comfy::Vec2 {
        self.offset.clone()
    }
}
