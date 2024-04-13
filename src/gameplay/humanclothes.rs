use comfy::{commands, vec2, ChooseRandom, Entity, Sprite, Transform};

use crate::{lazy_load_texture, utils::fileutils::list_assets_subdirectory, RES_I32};

pub type ClothReference = String;

#[derive(Debug)]
pub struct Look {
    pub hair: ClothReference,
    pub body: Option<ClothReference>,
    pub eyes: ClothReference,
}

fn assets_sub_with_prefix(sub: &str, prefix: &str) -> Vec<String> {
    let all_files = list_assets_subdirectory(sub).unwrap();
    let filtered_files: Vec<String> = all_files
        .into_iter()
        .filter(|p| p.starts_with(&(prefix.to_owned() + "_")))
        .map(|str| format!("{}/{}", sub, str))
        .collect();
    if filtered_files.len() < 1 {
        panic!("Assets with prefix '{}' not found", prefix);
    }
    return filtered_files;
}

fn pick_random_asset_with_prefix(prefix: String) -> ClothReference {
    let eyes = assets_sub_with_prefix("human", &prefix);
    let chosen: ClothReference = eyes.choose().unwrap().to_owned();
    chosen
}

impl Look {
    pub fn new() -> Self {
        Self {
            hair: Self::pick_hair(),
            body: Some(Self::pick_clothes()),
            eyes: Self::pick_eyes()
        }
    }

    pub fn pick_eyes() -> ClothReference {
        pick_random_asset_with_prefix("eyes".into())
    }

    pub fn pick_clothes() -> ClothReference {
        pick_random_asset_with_prefix("clothes".into())
    }

    pub fn pick_hair() -> ClothReference {
        pick_random_asset_with_prefix("hair".into())
    }

    pub fn lazy_load_sprites(&self) {
        lazy_load_texture(self.eyes.clone());
        lazy_load_texture(self.hair.clone());
        if let Some(item) = &self.body {
            lazy_load_texture(item.clone());
        }
    }

    pub fn spawn_for_entity(&self, entity: Entity) {
        let eyes = EyesLookPart { 
            value: self.eyes.clone(),
            ent: entity
        };
        eyes.spawn_comfy();
        let hair = HairLookPart {
            value: self.hair.clone(),
            ent: entity
        };
        hair.spawn_comfy();
        if let Some(item) = &self.body {
            let body = BodyClothesLookPart {
                value: item.clone(),
                ent: entity
            };
            body.spawn_comfy()
        }
    }
}

pub struct EyesLookPart {
    pub ent: Entity,
    value: ClothReference
}

impl EyesLookPart {
    pub fn spawn_comfy(self) {
        let sprite = Sprite::new(self.value.clone(), vec2(1 as f32, 1 as f32), 12, comfy::WHITE).with_rect(
            0,
            0,
            RES_I32,
            RES_I32,
        );
        commands().spawn((
            sprite,
            Transform::position(
                vec2(1.0, 1.0),
            ),
            self,
        ));
    }
}

pub struct BodyClothesLookPart {
    pub ent: Entity,
    value: ClothReference
}

impl BodyClothesLookPart {
    pub fn spawn_comfy(self) {
        let sprite = Sprite::new(self.value.clone(), vec2(1 as f32, 1 as f32), 13, comfy::WHITE).with_rect(
            0,
            0,
            RES_I32,
            RES_I32,
        );
        commands().spawn((
            sprite,
            Transform::position(
                vec2(1.0, 1.0),
            ),
            self,
        ));
    }
}

pub struct HairLookPart {
    pub ent: Entity,
    value: ClothReference
}

impl HairLookPart {
    pub fn spawn_comfy(self) {
        let sprite = Sprite::new(self.value.clone(), vec2(1 as f32, 1 as f32), 14, comfy::WHITE).with_rect(
            0,
            0,
            RES_I32,
            RES_I32,
        );
        commands().spawn((
            sprite,
            Transform::position(
                vec2(1.0, 1.0),
            ),
            self,
        ));
    }
}
