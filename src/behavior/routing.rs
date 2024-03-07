use std::collections::LinkedList;

use comfy::ChooseRandom;
use pathfinding::directed::astar::astar;

use crate::{
    core::position::{Ps, PsProvider, PsSigned},
    worldmap::Cellmap,
};

pub trait PathfindRouter: PsProvider {
    fn move_to_ps_or_around_1(&mut self, cellmap: &Cellmap, target: Ps) -> bool {
        let first_try = self.move_to_ps(cellmap, target);
        match first_try {
            false => self.move_around_ps(cellmap, target),
            true => true,
        }
    }

    fn move_to_ps(&mut self, cellmap: &Cellmap, target: Ps) -> bool {
        let path = try_find_route_from_to(cellmap, true, self.get_current_ps(), target);
        match path {
            Some(p) => {
                self.follow_steps(target, p);
                true
            }
            None => false,
        }
    }

    fn follow_steps(&mut self, target: Ps, steps: LinkedList<Ps>);

    fn move_around_ps(&mut self, cellmap: &Cellmap, target: Ps) -> bool {
        // println!("Finding path around {:?}", target);
        let around = target.successors(cellmap, true);
        if around.is_empty() {
            false
        } else {
            let mut routes: Vec<LinkedList<Ps>> = around
                .into_iter()
                .filter_map(|tgt| try_find_route_from_to(cellmap, true, self.get_current_ps(), tgt))
                .collect();
            if let Some(found_path) = routes.choose_mut() {
                self.follow_steps(
                    found_path.back().unwrap_or(&self.get_current_ps()).clone(),
                    std::mem::take(found_path),
                );
                return true;
            }
            false
        }
    }
}

pub trait PathfindPoint {
    fn successors(&self, cellmap: &Cellmap, skip_ocuppied: bool) -> Vec<Ps>;
    fn successors_weighted(&self, cellmap: &Cellmap, skip_ocuppied: bool) -> Vec<(Ps, i32)>;
}

pub const SUCCESSORS: [(isize, isize); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

impl PathfindPoint for Ps {
    fn successors(&self, cellmap: &Cellmap, skip_ocuppied: bool) -> Vec<Ps> {
        let mut successors: Vec<Ps> = Vec::with_capacity(4);
        for (x, y) in SUCCESSORS {
            let possible_place = PsSigned {
                x: x + self.x as isize,
                y: y + self.y as isize,
            };
            if cellmap.xy_within_bounds(&possible_place) {
                let possible_cell = cellmap.get_xy(possible_place.x, possible_place.y);
                if possible_cell.is_passable(skip_ocuppied) {
                    successors.push(possible_cell.position)
                }
            }
        }
        successors.shuffle();
        successors
    }

    fn successors_weighted(&self, cellmap: &Cellmap, skip_ocuppied: bool) -> Vec<(Ps, i32)> {
        let successors = self.successors(cellmap, skip_ocuppied);
        successors.into_iter().map(|p| (p, 1)).collect()
    }
}

pub fn try_find_route_from_to(
    cellmap: &Cellmap,
    skip_occupied: bool,
    start: Ps,
    target: Ps,
) -> Option<LinkedList<Ps>> {
    // let dist = start.manhattan_distance(&target);
    // if dist > 10 {
    //     print!("[{}]", dist);
    // }
    let result = astar(
        &start,
        |p| p.successors_weighted(cellmap, skip_occupied),
        |p| p.manhattan_distance(&target),
        |p| *p == target,
    );
    result.map(|vc| {
        let mut list = LinkedList::from_iter(vc.0.into_iter());
        list.pop_front(); // first step we never need, it's our current position
        list
    })
}
