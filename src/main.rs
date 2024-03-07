pub mod behavior;
pub mod gameplay;
pub mod initializers;
pub mod state;
mod tiledreader;
mod updaters;
pub mod utils;
mod worldmap;

pub mod core;

use core::Initializable;
use std::{fs::File, io::Read};

use comfy::{num_traits::ToPrimitive, *};
use gameplay::gametime::Time;
use initializers::{create_bones, initialize_bones};
use state::{Reality, WorldState};
use tiledreader::*;
use updaters::GLOBAL_HEATMAP;
use worldmap::Cellmap;

use crate::{behavior::carriable::carriableitem::CarriableItemHandle, initializers::tile_object_for_tile};

const RES_I32: i32 = 48;

lazy_static::lazy_static! {
    static ref TEXTURES_TO_LOAD: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
}

pub fn lazy_load_texture(texture: String) {
    let mut set = TEXTURES_TO_LOAD.lock();
    set.insert(texture);
}

#[derive(Debug, Copy, Clone)]
struct Player;
#[derive(Debug)]
struct Bg;
#[derive(Debug)]
struct Fg;

#[derive(Debug)]
struct Selection;

#[derive(Debug)]
pub struct Bone {
    initialized: bool,
}

impl Initializable for Bone {
    fn initialize(&mut self, entity: &Entity, transform: &mut Transform, reality: &mut Reality) {
        let mut carriables = reality.carriables.lock();
        let handle = CarriableItemHandle::new(behavior::item_types::BONE, *entity, transform.position.into());
        println!("Initialized Bone {:?}: {:?}", entity, handle);
        carriables.insert(*entity, handle);
        self.initialized = true;
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[derive(Debug)]
pub struct TrashCan {
    initialized: bool,
}

impl Initializable for TrashCan {
    fn initialize(&mut self, entity: &Entity, transform: &mut Transform, reality: &mut Reality) {
        let mut carriables = reality.carriables.lock();
        let handle = behavior::carriable::carriableitem::CarriableItemHandle::new(
            behavior::item_types::TRASHCAN,
            *entity,
            transform.position.into(),
        );
        println!("Initialized Trashcan {:?}: {:?}", entity, handle);
        carriables.insert(*entity, handle);
        self.initialized = true;
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl GameLoop for WorldState {
    fn new(_c: &mut EngineState) -> Self {
        // begin

        let map = read_tilemap_default();
        set_y_sort(0, true);

        let decor = create_decorations_map(&map, 0, 2);
        let cellmap = create_cellmap(map, 1);
        let world = Self {
            reality: Reality::new(cellmap),
            x: 1,
            y: 1,
            initialized: false,
            selected_cell: (100500, 100500).into(),
            selected: false,
            dog_order: None,
            entities_initialized: false,
            paused: false,
        };

        let mut heatmap = GLOBAL_HEATMAP.lock();
        heatmap.reset_and_resize(
            0.0,
            world.reality.cellmap.wh_i32().0,
            world.reality.cellmap.wh_i32().1,
        );

        let (max_x, max_y) = world.reality.cellmap.wh_i32();

        for x in 0..max_x {
            for y in 0..max_y {
                let cell = world.reality.cellmap.get_xy(x, y);
                if let Some(tile) = &cell.reference {
                    let name = tile.tile_image.clone();
                    // println!("Tile name: {}", name);
                    let mut size = vec2(
                        tile.size.x as f32 / RES_I32 as f32,
                        tile.size.y as f32 / RES_I32 as f32,
                    );
                    let object = tile_object_for_tile(tile);
                    // render sprite
                    match tile.animated {
                        Some(anim) => {
                            size.x = size.x / anim.steps as f32;
                            println!("Animated tile: {:?} {:?}\n{:?}", anim, tile, size);
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
                            builder.z_index = 1;
                            let mut animated = builder.build();
                            animated.play("base");
                            commands().spawn((
                                animated,
                                Transform::position(
                                    vec2(x as f32, y as f32)
                                        + ((size - vec2(1.0, 1.0)) / vec2(2.0, 2.0)),
                                ),
                                object,
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
                                    vec2(x as f32, y as f32)
                                        + ((size - vec2(1.0, 1.0)) / vec2(2.0, 2.0)),
                                ),
                                object,
                            ));
                        }
                    }
                    // old code below
                    // commands().spawn((
                    //     Sprite::new(name, size, 1, WHITE).with_rect(0, 0, tile.size.x, tile.size.y),
                    //     Transform::position(
                    //         vec2(x as f32, y as f32) + ((size - vec2(1.0, 1.0)) / vec2(2.0, 2.0)),
                    //     ),
                    //     Grass,
                    // ));
                }

                let decor_cell = decor.get_xy(x, y);
                if let Some(bg) = &decor_cell.bg {
                    lazy_load_texture(bg.to_owned());
                    match decor_cell.animated_bg {
                        Some(anim) => {
                            let mut builder = AnimatedSpriteBuilder::new().add_animation(
                                "base",
                                anim.delay,
                                true,
                                AnimationSource::Atlas {
                                    name: bg.to_owned().into(),
                                    offset: ivec2(0, 0),
                                    step: ivec2(RES_I32, 0),
                                    size: isplat(RES_I32),
                                    frames: anim.steps.clone(),
                                },
                            );
                            builder.z_index = -1;
                            let mut animated = builder.build();
                            animated.play("base");
                            commands().spawn((
                                animated,
                                Transform::position(vec2(x as f32, y as f32)),
                                Bg,
                            ));
                        }
                        None => {
                            commands().spawn((
                                Sprite::new(bg.to_owned(), vec2(1.0, 1.0), -1, WHITE)
                                    .with_rect(0, 0, RES_I32, RES_I32),
                                Transform::position(vec2(x as f32, y as f32)),
                                Bg,
                            ));
                        }
                    }
                }
                if let Some(fg) = &decor_cell.top {
                    lazy_load_texture(fg.to_owned());
                    match decor_cell.animated_top {
                        Some(anim) => {
                            let mut builder = AnimatedSpriteBuilder::new().add_animation(
                                "base",
                                anim.delay,
                                true,
                                AnimationSource::Atlas {
                                    name: fg.to_owned().into(),
                                    offset: ivec2(0, 0),
                                    step: ivec2(RES_I32, 0),
                                    size: isplat(RES_I32),
                                    frames: anim.steps.clone(),
                                },
                            );
                            builder.z_index = 100;
                            let mut animated = builder.build();
                            animated.play("base");
                            commands().spawn((
                                animated,
                                Transform::position(vec2(x as f32, y as f32)),
                                Fg,
                            ));
                        }
                        None => {
                            commands().spawn((
                                Sprite::new(fg.to_owned(), vec2(1.0, 1.0), 100, WHITE)
                                    .with_rect(0, 0, RES_I32, RES_I32),
                                Transform::position(vec2(x as f32, y as f32)),
                                Fg,
                            ));
                        }
                    }
                }
            }
        }

        let (w, h) = world.reality.cellmap.wh_usize();
        let fw = w as f32;
        let fh = h as f32;
        commands().spawn((Transform::position(vec2(fw / 2.0, fh / 2.0)), Player));

        commands().spawn((
            Sprite::new("selectionhd.png", vec2(1.0, 1.0), 10, WHITE)
                .with_rect(0, 0, RES_I32, RES_I32),
            Transform::position(vec2(fw / 2.0, fh / 2.0)),
            Selection,
        ));

        create_bones(3, &world.reality.cellmap);
        initializers::create_trashcans(2, &world.reality.cellmap);

        initializers::spawn_dogs(
            100,
            &world.reality.cellmap,
            world.reality.cellmap.wh_usize().0,
            world.reality.cellmap.wh_usize().1,
        );
        // spawn_dog("Jumpy".to_string(), 6, 6);
        // spawn_dog("Lasy".to_string(), 8, 6);
        // spawn_dog("Kord".to_string(), 5, 4);
        // spawn_dog("Donald".to_string(), 32, 3);
        // spawn_dog("Jetty".to_string(), 6, 2);
        world
    }

    fn update(&mut self, c: &mut EngineContext) {
        if !self.initialized {
            setup(c, &self.reality.cellmap);
            initialize_bones(self, c);
            self.initialized = true;
            return;
        }

        let dt = c.delta;

        updaters::update_initializable_all(self, c, dt);
        updaters::update_bones(self, c, dt);
        updaters::update_dogs(self, c, dt);
        updaters::update_camera(self, c, dt);
        updaters::update_selection(self, c, dt);
        // updaters::update_heatmap(self, c, dt);
        updaters::update_time(self, c, dt);
    }
}

comfy_game!("Simulation", WorldState);

fn load_sprite(c: &mut EngineContext, sprite_name: &str) {
    let path = env!("CARGO_MANIFEST_DIR").to_owned() + &format!("/assets/{}", { sprite_name });
    load_texture(c, sprite_name, &path);
}

fn load_file(path: &str) -> Option<Vec<u8>> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file {}: {}", path, e);
            return None;
        }
    };

    let mut buffer = Vec::new();
    match file.read_to_end(&mut buffer) {
        Ok(_) => {
            return Some(buffer);
        }
        Err(e) => {
            eprintln!("Error reading file {}: {}", path, e);
            return None;
        }
    }
}

fn load_texture(c: &mut EngineContext, name: &str, path: &str) {
    let data = load_file(path).unwrap();
    c.load_texture_from_bytes(
        // Name of our sprite
        name, // &[u8] with the image data.
        &data,
    );
}

fn setup(c: &mut EngineContext, cellmap: &Cellmap) {
    const SPRITES: [&str; 5] = ["bone", "wat", "trash_can48", "selectionhd", "dog48"];

    let mut sprites: HashSet<String> = cellmap
        .map
        .iter()
        .filter_map(|f| f.get_tile_name())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    sprites.extend(SPRITES.into_iter().map(|f| f.to_string() + ".png"));
    sprites.extend(TEXTURES_TO_LOAD.lock().iter().map(|f| f.to_owned()));

    for s in sprites.iter() {
        println!("Loading sprite {s}");
        load_sprite(c, s);
    }
}
