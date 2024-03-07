use crate::{
    behavior::{
        carriable::carriableitem::CarriableItemHandle,
        dog::{self, Dog},
        item_types::{BONE, TRASHCAN},
        messaging::MessagingHost,
    },
    gameplay::ent::{conputer::Conputer, Grass},
    state::WorldState,
    worldmap::{Cellmap, TileReference},
    Bone, TrashCan, RES_I32,
};
use comfy::*;

pub fn tile_object_for_tile(
    tile: &TileReference,
) -> Box<dyn crate::gameplay::ent::MapEntityObject> {
    match tile.klass.as_str() {
        "conputer" => {
            let x = tile
                .props
                .get("x")
                .map(|c| TileReference::extract_int_value(c))
                .flatten()
                .unwrap_or(0);
            let y = tile
                .props
                .get("y")
                .map(|c| TileReference::extract_int_value(c))
                .flatten()
                .unwrap_or(0);
            let pc = Conputer::new(ivec2(x, y));
            println!("Cnputer created: {:?}", pc);
            Box::new(pc)
        }
        _x => Box::new(Grass {}),
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

pub fn spawn_dog(name: String, x: usize, y: usize) {
    println!("++ {:?} (x: {}, y: {})", name, x, y);
    crate::lazy_load_texture("dog48_idle.png".into());
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
            .build(),
        Transform::position(vec2(x as f32, y as f32)),
        dog::Dog::new(name.to_string(), f32::gen_range(3.0, 10.0), (x, y).into()),
    ));
    // commands().spawn((
    //     Sprite::new("dog48.png", vec2(1.0, 1.0), 1, WHITE).with_rect(0, 0, RES_I32, RES_I32),
    //     Transform::position(vec2(x.to_f32().unwrap(), y.to_f32().unwrap())),
    //     dog::Dog::new(name.to_string(), f32::gen_range(3.0, 10.0), (x, y).into()),
    // ));
}
