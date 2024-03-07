use crate::behavior::creatures::{Direction, PsOffsetProvider};
use crate::behavior::dog;
use crate::behavior::routing::PathfindRouter;
use crate::core::anycellmap::AnyCellmap;
use crate::core::Initializable;
use crate::gameplay::ent::bed::Bed;
use crate::gameplay::ent::conputer::Conputer;
use crate::gameplay::ent::officeworker::OfficeWorker;
use crate::initializers::create_bones;
use crate::state::WorldState;
use comfy::hecs::Component;
use comfy::{
    commands, draw_rect_outline, is_key_pressed, splat, vec2, IntoParallelIterator, Lazy, Mutex,
    ParallelIterator, Sprite, TextParams, BLUE, GREEN, ORANGE_RED, WHITE,
};
use comfy::{
    is_key_down, is_mouse_button_pressed, main_camera_mut, num_traits::ToPrimitive, world,
    EngineContext, KeyCode, MouseButton, Transform, Vec2,
};

pub static GLOBAL_HEATMAP: Lazy<Mutex<AnyCellmap<f64>>> =
    Lazy::new(|| Mutex::new(AnyCellmap::new(&0.0, 0, 0)));

use crate::{Bone, Player, Selection, TrashCan, RES_I32};

pub fn update_camera(state: &mut WorldState, _c: &mut EngineContext, dt: f32) {
    for (_, (_, transform)) in world().query::<(&Player, &mut Transform)>().iter() {
        // Handle movement and animation
        let mut moved = false;
        let speed = 30.0;
        let mut move_dir = Vec2::ZERO;

        if is_key_down(KeyCode::W) {
            move_dir.y += 1.0;
            moved = true;
        }
        if is_key_down(KeyCode::S) {
            move_dir.y -= 1.0;
            moved = true;
        }
        if is_key_down(KeyCode::A) {
            move_dir.x -= 1.0;
            moved = true;
        }
        if is_key_down(KeyCode::D) {
            move_dir.x += 1.0;
            moved = true;
        }

        if moved {
            transform.position += move_dir.normalize_or_zero() * speed * dt;
        }

        if is_key_pressed(KeyCode::F) {
            state.paused = !state.paused;
            println!("Paused: {}", state.paused)
        }

        if is_mouse_button_pressed(MouseButton::Right) {
            let mousepad = comfy::mouse_world();
            let x = (mousepad.x / 1.0).round().to_i32().unwrap();
            let y = (mousepad.y / 1.0).round().to_i32().unwrap();
            println!("Clicked right: x: {}   y: {}", x, y);
            if state.reality.cellmap.within_bounds(x, y) {
                state.dog_order = Some((x, y).into())
            }
        }

        main_camera_mut().center = transform.position;
    }
}

pub fn update_bones(state: &mut WorldState, _c: &mut EngineContext, _dt: f32) {
    for (entity, (obj, transform)) in world().query::<(&mut Bone, &mut Transform)>().iter() {
        let mut handles = state.reality.carriables.lock();
        let handle = handles.get(&entity).unwrap();
        transform.position = handle.get_exact_pos();
        if handle.consumed {
            commands().despawn(entity);
            handles.remove(&entity);
            create_bones(1, &state.reality.cellmap)
        }
    }
}

pub fn update_init<T: Initializable + Component>(state: &mut WorldState, _c: &mut EngineContext, _dt: f32) {
    for (entity, (obj, transform)) in world().query::<(&mut T, &mut Transform)>().iter() {
        if !obj.is_initialized() {
            obj.initialize(&entity, transform, &mut state.reality)
        }
    }
}

pub fn update_initializable_all(state: &mut WorldState, c: &mut EngineContext, dt: f32) {
    update_init::<TrashCan>(state, c, dt);
    // println!(" ---> TrashCans initialized");
    update_init::<Bone>(state, c, dt);
    // println!(" ---> Bones initialized");
    update_init::<Conputer>(state, c, dt);
    // println!(" ---> Conputers initialized");
    update_init::<Bed>(state, c, dt);
    // println!(" ---> Beds initialized");
    update_init::<dog::Dog>(state, c, dt);
    // println!(" ---> Dogs initialized");
    update_init::<OfficeWorker>(state, c, dt);
    // println!(" ---> Workers initialized");
}

pub fn update_selection(state: &mut WorldState, _c: &mut EngineContext, _dt: f32) {
    for (entity, (_, transform)) in world().query::<(&Selection, &mut Transform)>().iter() {
        if !state.selected {
            commands().despawn(entity)
        }
        if state.selected
            && state
                .reality
                .cellmap
                .within_bounds(state.selected_cell.x, state.selected_cell.y)
        {
            transform.position = state.selected_cell.into();
        }
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        if !state.selected {
            commands().spawn((
                Sprite::new("selectionhd.png", vec2(1.0, 1.0), 10, WHITE)
                    .with_rect(0, 0, RES_I32, RES_I32),
                Transform::position(vec2(0.0, 0.0)),
                Selection,
            ));
        }
        let mousepad = comfy::mouse_world();
        let x = (mousepad.x / 1.0).round().to_i32().unwrap();
        let y = (mousepad.y / 1.0).round().to_i32().unwrap();
        println!("Clicked: x: {}   y: {}", x, y);
        if !state.reality.cellmap.within_bounds(x, y) {
            state.deselect_cell()
        } else {
            if state.reality.cellmap.pos_within_bounds(state.selected_cell) {
                state.reality.cellmap.deoccupy_ps(&state.selected_cell);
            }
            state.select_or_deselect_cell((x, y).into());
            if state
                .reality
                .cellmap
                .within_bounds(state.selected_cell.x, state.selected_cell.y)
            {
                let cell = state
                    .reality
                    .cellmap
                    .get_xy(state.selected_cell.x, state.selected_cell.y);
                println!("Cell info: {:?}", cell);
                // make this point occupied
                if state.selected {
                    state.reality.cellmap.occupy_xy(x, y)
                }
            }
        }
    }

    if is_key_down(KeyCode::Q) {
        main_camera_mut().zoom += 1.0;
    }

    if is_key_down(KeyCode::E) {
        main_camera_mut().zoom -= 1.0;
    }
}

pub fn update_dogs(state: &mut WorldState, _c: &mut EngineContext, dt: f32) {
    let wrld = world();
    let mut queried = wrld.query::<(&mut dog::Dog, &mut Transform)>();
    let items = queried.iter().collect::<Vec<_>>();

    if !state.paused {
        items.into_par_iter().for_each(|data| {
            let (entity, (dog, _)) = data;
            if !dog.initialized {
                return;
            }
            dog.sa.sanity.lock().move_direction_if_can(dt);
            if let Some(order) = state.dog_order {
                dog.sa.sanity.lock().intend_go_to(order);
            }
            let intention_result = dog.sa.sanity.lock().think_intention_level_if_not_moving(
                entity,
                &state.reality,
            );
            dog.sa.think_routine_level(
                intention_result,
                &state.reality,
                entity,
                dt,
            );
        });
    }

    let mut items_again = queried.iter().collect::<Vec<_>>();

    comfy::ChooseRandom::shuffle(&mut items_again);

    for (_entity, (dog, transform)) in items_again.into_iter() {
        transform.position = dog.get_exact_pos();

        if state.paused {
            if dog.sa.get_ps() == state.selected_cell && state.selected {
                let mut text_params = TextParams::default();
                text_params.color = WHITE;
                text_params.font.size = 8.0;
                comfy::draw_text_ex(
                    &format!("dog: {:?}", dog),
                    transform.position,
                    comfy::TextAlign::TopLeft,
                    text_params,
                );

                let tgt = dog.sa.sanity.lock().mv.current_move_path.target.clone();

                if let Some(target) = tgt {
                    draw_rect_outline(
                        vec2(target.x.to_f32().unwrap(), target.y.to_f32().unwrap()),
                        splat(0.9),
                        0.6,
                        comfy::RED,
                        1,
                    );
                }

                for step in dog
                    .sa
                    .sanity
                    .lock()
                    .mv
                    .current_move_path
                    .calculated_steps
                    .iter()
                {
                    draw_rect_outline(
                        vec2(step.x.to_f32().unwrap(), step.y.to_f32().unwrap()),
                        splat(0.7),
                        0.1,
                        ORANGE_RED,
                        1,
                    );
                }
            }
        }

        if is_key_pressed(KeyCode::Up) {
            let dir = Direction::Up;
            dog.sa
                .sanity
                .lock()
                .start_move_direction(dir, &state.reality.cellmap);
            println!("Dog redirected: {:?}", dog);
        }
        if is_key_pressed(KeyCode::Down) {
            let dir = Direction::Down;
            dog.sa
                .sanity
                .lock()
                .start_move_direction(dir, &state.reality.cellmap);
            println!("Dog redirected: {:?}", dog);
        }
        if is_key_pressed(KeyCode::Left) {
            let dir = Direction::Left;
            dog.sa
                .sanity
                .lock()
                .start_move_direction(dir, &state.reality.cellmap);
            println!("Dog redirected: {:?}", dog);
        }
        if is_key_pressed(KeyCode::Right) {
            let dir = Direction::Right;
            dog.sa
                .sanity
                .lock()
                .start_move_direction(dir, &state.reality.cellmap);
            println!("Dog redirected: {:?}", dog);
        }

        if is_key_pressed(KeyCode::P) {
            dog.sa
                .sanity
                .lock()
                .move_to_ps(&state.reality.cellmap, (12, 8).into());
            println!("Dog ordered to move: {:?}", dog);
        }

        if is_key_pressed(KeyCode::I) {
            println!("Dog status: {:?}", dog);
        }

        let mut sanity = dog.sa.sanity.lock();

        if sanity.carrier.has_anything() {
            let ps_offset = sanity.mv.as_ps_offset_container();
            sanity
                .carrier
                .update_positions(&state.reality.carriables, &ps_offset)
        }

        if !state.paused {
            sanity.think_movement_level_if_not_moving(&mut state.reality.cellmap);
        }
    }
    // print!("  ");
    state.dog_order = None;
}

pub fn update_heatmap(state: &mut WorldState, _c: &mut EngineContext, _dt: f32) {
    let mut heatmap = GLOBAL_HEATMAP.lock();
    for (index, item) in state.reality.cellmap.map.iter().enumerate() {
        let pos = item.position;

        if state.paused && item.status.occupied {
            draw_rect_outline(
                vec2(pos.x.to_f32().unwrap(), pos.y.to_f32().unwrap()),
                splat(1.0),
                0.3,
                BLUE,
                1,
            );
        } else {
            if item.status.occupied {
                heatmap.map[index] += 0.002;
            }
        }

        let value = heatmap.map[index] / 128.0;

        if value > 0.005 {
            draw_rect_outline(
                vec2(pos.x.to_f32().unwrap(), pos.y.to_f32().unwrap()),
                splat(1.0),
                0.9,
                GREEN.alpha(value as f32),
                1,
            );
        }
    }
}

pub fn update_time(state: &mut WorldState, _c: &mut EngineContext, dt: f32) {
    if !state.paused {
        state.reality.time.tick(dt);
    }

    comfy::draw_text(
        &format!("time: {}", state.reality.time),
        Vec2::ZERO,
        WHITE,
        comfy::TextAlign::Center,
    );
}

pub fn update_sane_objects(state: &mut WorldState, _c: &mut EngineContext, dt: f32) {
    let wrld = world();
    let mut queried = wrld.query::<(&mut OfficeWorker, &mut Transform)>();
    let items = queried.iter().collect::<Vec<_>>();

    if !state.paused {
        items.into_par_iter().for_each(|data| {
            let (entity, (dog, _)) = data;
            if !dog.initialized {
                return;
            }
            dog.sa.sanity.lock().move_direction_if_can(dt);
            if let Some(order) = state.dog_order {
                dog.sa.sanity.lock().intend_go_to(order);
            }
            let intention_result = dog.sa.sanity.lock().think_intention_level_if_not_moving(
                entity,
                &state.reality,
            );
            dog.sa.think_routine_level(
                intention_result,
                &state.reality,
                entity,
                dt,
            );
        });
    }

    let mut items_again = queried.iter().collect::<Vec<_>>();

    comfy::ChooseRandom::shuffle(&mut items_again);

    for (_entity, (dog, transform)) in items_again.into_iter() {
        transform.position = dog.sa.get_exact_pos();

        if state.paused {
            if dog.sa.get_ps() == state.selected_cell && state.selected {
                let mut text_params = TextParams::default();
                text_params.color = WHITE;
                text_params.font.size = 8.0;
                comfy::draw_text_ex(
                    &format!("dog: {:?}", dog),
                    transform.position,
                    comfy::TextAlign::TopLeft,
                    text_params,
                );

                let tgt = dog.sa.sanity.lock().mv.current_move_path.target.clone();

                if let Some(target) = tgt {
                    draw_rect_outline(
                        vec2(target.x.to_f32().unwrap(), target.y.to_f32().unwrap()),
                        splat(0.9),
                        0.6,
                        comfy::RED,
                        1,
                    );
                }

                for step in dog
                    .sa
                    .sanity
                    .lock()
                    .mv
                    .current_move_path
                    .calculated_steps
                    .iter()
                {
                    draw_rect_outline(
                        vec2(step.x.to_f32().unwrap(), step.y.to_f32().unwrap()),
                        splat(0.7),
                        0.1,
                        ORANGE_RED,
                        1,
                    );
                }
            }
        }

        let mut sanity = dog.sa.sanity.lock();

        if sanity.carrier.has_anything() {
            let ps_offset = sanity.mv.as_ps_offset_container();
            sanity
                .carrier
                .update_positions(&state.reality.carriables, &ps_offset)
        }

        if !state.paused {
            sanity.think_movement_level_if_not_moving(&mut state.reality.cellmap);
        }
    }
    // print!("  ");
    state.dog_order = None;
}
