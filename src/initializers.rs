use crate::{
    behavior::{
        carriable::carriableitem::CarriableItemHandle,
        dog,
        item_types::{BONE, TRASHCAN},
        messaging::communication::Communicator,
    },
    core::{animation::AdditionalAnimationDescr, position::Ps},
    gameplay::ent::{officeworker::OfficeWorker, MapEntityObject},
    state::WorldState,
    ui::statusbar::{self, Statusbar},
    worldmap::{Cellmap, TileReference},
    Bone, TrashCan, RES_I32,
};
use comfy::{num_traits::ToPrimitive, *};

pub fn spawn_object_sprite<T, F>(
    x: i32,
    y: i32,
    tile: &TileReference,
    mut size: Vec2,
    name: String,
    spawner: F,
    animations: Vec<AdditionalAnimationDescr>,
) where
    F: Fn() -> T,
    T: MapEntityObject + 'static,
{
    match tile.animated {
        Some(anim) => {
            size.x = size.x / anim.steps as f32;
            let mut builder = AnimatedSpriteBuilder::new().add_animation(
                "base",
                anim.delay,
                true,
                AnimationSource::Atlas {
                    name: name.to_owned().into(),
                    offset: ivec2(0, 0),
                    step: ivec2(RES_I32, 0),
                    size: isplat(RES_I32),
                    frames: anim.steps.clone(),
                },
            );
            for other_anim in animations.iter() {
                builder = builder.add_animation(
                    &other_anim.animation_name,
                    other_anim.delay,
                    true,
                    AnimationSource::Atlas {
                        name: other_anim.atlas_name.to_owned().into(),
                        offset: ivec2(0, 0),
                        step: ivec2(RES_I32, 0),
                        size: isplat(RES_I32),
                        frames: other_anim.steps,
                    },
                );
            }
            builder.z_index = 1;
            let mut animated = builder.build();
            animated.play("base");
            println!(
                "Animated sprite (x: {}, y: {}) size: {:?}",
                x,
                y,
                vec2(x as f32, y as f32) + ((size - vec2(1.0, 1.0)) / vec2(2.0, 2.0))
            );
            commands().spawn((
                animated,
                Transform::position(
                    vec2(x as f32, y as f32) + ((size - vec2(1.0, 1.0)) / vec2(2.0, 2.0)),
                ),
                spawner(),
            ));
        }
        None => {
            commands().spawn((
                Sprite::new(name.to_owned(), size, 1, WHITE).with_rect(
                    0,
                    0,
                    tile.size.x,
                    tile.size.y,
                ),
                Transform::position(
                    vec2(x as f32, y as f32) + ((size - vec2(1.0, 1.0)) / vec2(2.0, 2.0)),
                ),
                spawner(),
            ));
        }
    }
}

pub fn initialize_bones(state: &mut WorldState, _c: &mut EngineContext) {
    println!("Initializing bones...");
    let mut carriables = state.reality.carriables.lock();
    for (entity, (_bone, transform)) in world().query::<(&mut Bone, &mut Transform)>().iter() {
        let handle = CarriableItemHandle::new(BONE, entity, transform.position.into());
        println!("Bone {:?}: {:?}", entity, handle);
        carriables.insert(entity, handle);
    }
}

pub fn create_bones(count: usize, map: &Cellmap) {
    for _ in 0..count {
        let ps = map.pick_random_passable_ps();
        commands().spawn((
            Sprite::new("bone.png", vec2(1.0, 1.0), 10, WHITE).with_rect(0, 0, RES_I32, RES_I32),
            Transform::position(ps.into()),
            Bone { initialized: false },
        ));
    }
}

pub fn create_trashcans(count: usize, map: &Cellmap) {
    for _ in 0..count {
        let ps = map.pick_random_passable_ps();
        commands().spawn((
            Sprite::new("trash_can48.png", vec2(1.0, 1.0), 10, WHITE)
                .with_rect(0, 0, RES_I32, RES_I32),
            Transform::position(ps.into()),
            TrashCan { initialized: false },
        ));
    }
}

pub fn initialize_trashcan(
    state: &mut WorldState,
    o: &mut crate::TrashCan,
    transform: &Transform,
    entity: Entity,
) {
    let mut carriables = state.reality.carriables.lock();
    let handle = CarriableItemHandle::new(TRASHCAN, entity, transform.position.into());
    println!("Initialized Trashcan {:?}: {:?}", entity, handle);
    carriables.insert(entity, handle);
    o.initialized = true;
}

pub fn spawn_dogs(count: isize, cellmap: &Cellmap, x_limit: usize, y_limit: usize) {
    // println!("Spawning {} dogs...", count);
    for i in 1..=count {
        let mut regenerate = true;
        while regenerate {
            let x = usize::gen_range(0, x_limit);
            let y = usize::gen_range(0, y_limit);
            if cellmap.get_xy(x, y).is_passable(true) {
                regenerate = false;
                spawn_dog(format!("Dog {}", i), x, y);
            }
        }
    }
}

pub fn spawn_workers(count: isize, cellmap: &Cellmap, x_limit: usize, y_limit: usize) {
    println!("Spawning {} workers...", count);
    for i in 1..=count {
        let mut regenerate = true;
        while regenerate {
            let x = usize::gen_range(0, x_limit);
            let y = usize::gen_range(0, y_limit);
            if cellmap.get_xy(x, y).is_passable(true) {
                regenerate = false;
                spawn_worker(format!("Worker {}", i), x, y);
            }
        }
    }
}

pub fn spawn_dog(name: String, x: usize, y: usize) {
    println!("++ {:?} (x: {}, y: {})", name, x, y);
    crate::lazy_load_texture("dog48_idle.png".into());
    crate::lazy_load_texture("dog48_idle_reversed.png".into());
    commands().spawn((
        AnimatedSpriteBuilder::new()
            .z_index(1)
            .add_animation(
                "idle",
                0.1,
                true,
                AnimationSource::Atlas {
                    name: "dog48_idle.png".into(),
                    offset: ivec2(0, 0),
                    step: ivec2(RES_I32, 0),
                    size: isplat(RES_I32),
                    frames: 4,
                },
            )
            .add_animation(
                "idle_left",
                0.1,
                true,
                AnimationSource::Atlas {
                    name: "dog48_idle_reversed.png".into(),
                    offset: ivec2(0, 0),
                    step: ivec2(RES_I32, 0),
                    size: isplat(RES_I32),
                    frames: 4,
                },
            )
            .build(),
        Transform::position(vec2(x as f32, y as f32)),
        dog::Dog::new(name.to_string(), f32::gen_range(3.0, 10.0), (x, y).into()),
        Communicator::new(Ps { x, y }),
    ));
    // commands().spawn((
    //     Sprite::new("dog48.png", vec2(1.0, 1.0), 1, WHITE).with_rect(0, 0, RES_I32, RES_I32),
    //     Transform::position(vec2(x.to_f32().unwrap(), y.to_f32().unwrap())),
    //     dog::Dog::new(name.to_string(), f32::gen_range(3.0, 10.0), (x, y).into()),
    // ));
}

pub fn spawn_worker(name: String, x: usize, y: usize) {
    println!("WORKER {:?} (x: {}, y: {})", name, x, y);

    crate::lazy_load_texture("human/human_base.png".into());
    let sprite = Sprite::new("human/human_base.png", vec2(1.0, 1.0), 11, WHITE)
        .with_rect(0, 0, RES_I32, RES_I32);
    let worker = OfficeWorker::new(name.to_string(), f32::gen_range(3.0, 10.0), (x, y).into());
    worker.look.lazy_load_sprites();
    let statusbar = Statusbar::new();
    let communicator = Communicator::new(Ps { x, y });
    commands().spawn((
        sprite,
        Transform::position(vec2(x.to_f32().unwrap(), y.to_f32().unwrap())),
        worker,
        statusbar,
        communicator,
    ));
}
