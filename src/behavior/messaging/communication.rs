use std::cmp;

use comfy::{world, Entity, HashSet};

use crate::{
    core::{anycellmap::AnyCellmap, position::Ps},
    state::Reality,
    worldmap::Cellmap,
};

#[derive(Copy, Clone, PartialEq, Eq, comfy::Hash)]
pub struct Communicator {
    pub ps: Ps,
    pub vision_limit: i32,
}

impl Communicator {
    pub fn new(ps: Ps) -> Self {
        return Self {
            ps,
            vision_limit: 10,
        };
    }

    pub fn mark_position_on_map(&self, entity: Entity, map: &mut AnyCellmap<HashSet<Entity>>) {
        map.get_xy_mut(self.ps.x, self.ps.y).insert(entity);
    }

    pub fn find_visible_entities(&self, map: &Reality) -> Vec<Entity> {
        let mut found: Vec<Entity> = Vec::new();
        let lock = &map.comm_map.lock();
        for (ps, entity) in query_all_max_distance(self.vision_limit, self, lock).into_iter() {
            if self.is_ps_visible_from(&ps, &map.cellmap) {
                found.push(entity)
            }
        }
        return found;
    }

    pub fn is_ps_visible_from(&self, other: &Ps, map: &Cellmap) -> bool {
        return calculate_vision_ray(&self.ps, other, map);
    }
}

fn query_all_max_distance(
    distance: i32,
    from: &Communicator,
    map: &AnyCellmap<HashSet<Entity>>,
) -> Vec<(Ps, Entity)> {
    let from_x = cmp::max(from.ps.x as i32 - distance as i32, 0);
    let from_y = cmp::max(from.ps.x as i32 - distance as i32, 0);
    let to_x = cmp::min(from.ps.x as i32 + distance, map.wh_usize().0 as i32);
    let to_y = cmp::min(from.ps.y as i32 + distance, map.wh_usize().1 as i32);

    let mut found: Vec<(Ps, Entity)> = Vec::new();

    for i in from_x..to_x {
        for j in from_y..to_y {
            let pos = Ps::from((i, j));
            if pos.manhattan_distance(&from.ps) > (distance as i32) {
                continue;
            }
            let cell = map.get_xy(i, j);
            for ent in cell.iter() {
                found.push((pos, ent.to_owned()))
            }
        }
    }
    return found;
}

fn calculate_vision_ray(from: &Ps, to: &Ps, map: &Cellmap) -> bool {
    // println!("Raycasting: {:?} ---> {:?}", from, to);
    let mut reached = false;
    let mut next: Ps = from.clone();
    if next.manhattan_distance(to) < 2 {
        return true; // it's our point or very close, no point in raycasting
    }
    while !reached {
        // print!("   -> {:?} ", next);
        let delta = next.distance_to_normalize(to);
        // print!("[ {:?} ] ", delta);
        next = next.with_normalized_change(delta, 0.5);
        // check if we are close to our target
        if next.manhattan_distance(&to) < 2 {
            reached = true;
            break; // done, return success
        }
        // check if it is in bounds and passable
        if map.pos_within_bounds(next) && map.get_pos(&next).is_passable(false) {
            continue; // not done yet
        } else {
            // we cannot see through
            break;
        }
    }
    // println!("Raycasting complete: {:?} ---> {:?} <-- {:?} == {}", from, to, next, reached);
    reached
}
