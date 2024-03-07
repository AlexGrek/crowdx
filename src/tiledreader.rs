use comfy::{ivec2, num_traits::ToPrimitive};
use std::io::Cursor;
use tiled::{Loader, PropertyValue, Tileset};

use crate::{
    core::anycellmap::AnyCellmap,
    utils::basic::get_file_name,
    worldmap::{Cell, Cellmap, TileReference},
};

const IMPASSABLE_TILES: [&'static str; 3] = ["block", "wall", "conputer"];

pub struct MyTiledReader;

impl tiled::ResourceReader for MyTiledReader {
    type Resource = Cursor<&'static [u8]>;
    type Error = std::io::Error;

    // really dumb example implementation that just keeps resources in memory
    fn read_from(
        &mut self,
        path: &std::path::Path,
    ) -> std::result::Result<Self::Resource, Self::Error> {
        if path == std::path::Path::new("my_map.tmx") {
            Ok(Cursor::new(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/unnamed.tmx"
            ))))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            ))
        }
    }
}

pub fn read_tilemap_default() -> tiled::Map {
    let mut loader = Loader::new();
    let map = loader
        .load_tmx_map(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/level0.tmx"))
        .unwrap();
    return map;
}

#[derive(Debug, Clone)]
pub struct DecorTile {
    pub bg: Option<String>,
    pub top: Option<String>,
    pub animated_bg: Option<crate::core::animation::BasicTileAnimation>,
    pub animated_top: Option<crate::core::animation::BasicTileAnimation>,
}

pub fn create_decorations_map(
    map: &tiled::Map,
    index_bg: usize,
    index_top: usize,
) -> AnyCellmap<DecorTile> {
    let layer_bg = map.get_layer(index_bg).unwrap().as_tile_layer().unwrap();
    let layer_top = map.get_layer(index_top).unwrap().as_tile_layer().unwrap();
    let base_tileset: &Tileset = map.tilesets().last().unwrap();

    let max_x: i32 = layer_bg.height().unwrap().try_into().unwrap();
    let max_y: i32 = layer_bg.width().unwrap().try_into().unwrap();
    let default_decor_tile = DecorTile {
        bg: None,
        top: None,
        animated_bg: None,
        animated_top: None,
    };
    let mut map = AnyCellmap::new(&default_decor_tile, max_x, max_y);
    for y in 0..max_y {
        for x in 0..max_x {
            if let Some(tile) = layer_bg.get_tile(x, max_y - y - 1) {
                let index = tile.id();
                let tdata = base_tileset.get_tile(index).unwrap();
                let image = &tdata.image.as_ref();
                if let Some(img) = image {
                    map.get_xy_mut(x, y).bg =
                        img.source.to_str().map(|f| get_file_name(f)).flatten();
                    if tdata.properties.contains_key("animated") {
                        // ohohoho, we have an aniimated tile here!
                        if let PropertyValue::IntValue(frames) =
                            tdata.properties.get("frames").unwrap()
                        {
                            map.get_xy_mut(x, y).animated_bg = Some(
                                crate::core::animation::BasicTileAnimation::new(*frames, 0.1),
                            );
                        }
                    }
                }
            }
            if let Some(tile) = layer_top.get_tile(x, max_y - y - 1) {
                let index = tile.id();
                let tdata = base_tileset.get_tile(index).unwrap();
                let image = &tdata.image.as_ref();
                if let Some(img) = image {
                    map.get_xy_mut(x, y).top =
                        img.source.to_str().map(|f| get_file_name(f)).flatten();
                    if tdata.properties.contains_key("animated") {
                        // ohohoho, we have an aniimated tile here!
                        if let PropertyValue::IntValue(frames) =
                            tdata.properties.get("frames").unwrap()
                        {
                            map.get_xy_mut(x, y).animated_top = Some(
                                crate::core::animation::BasicTileAnimation::new(*frames, 0.1),
                            );
                        }
                    }
                }
            }
        }
    }
    map
}

pub fn create_cellmap(map: tiled::Map, index: usize) -> Cellmap {
    let layer0 = map.get_layer(index).unwrap().as_tile_layer().unwrap();

    let base_tileset: &Tileset = map.tilesets().last().unwrap();
    for tileset in map.tilesets() {
        println!("{:?}", tileset);
        // base_tileset = tileset;
    }
    let max_x: i32 = layer0.height().unwrap().try_into().unwrap();
    let max_y: i32 = layer0.width().unwrap().try_into().unwrap();
    let total = (max_x * max_y).to_usize().unwrap();
    let mut vec = crate::utils::create_vec::<Cell>(total);
    for y in 0..max_y {
        for x in 0..max_x {
            let position = (x, y);
            let tile_opt = layer0.get_tile(x, max_y - y - 1);
            let (tileref, passable): (Option<TileReference>, bool) = match tile_opt {
                Some(tile) => {
                    let index = tile.id();
                    let tdata = base_tileset.get_tile(index).unwrap();
                    let image = &tdata.image.as_ref();
                    let h = image.map(|f| f.height).unwrap_or(48);
                    let w = image.map(|f| f.width).unwrap_or(48);
                    let data = tile.get_tile().unwrap().properties.clone();
                    let klass = tile
                        .get_tile()
                        .unwrap()
                        .user_type
                        .clone()
                        .unwrap_or("".to_string());
                    let passable = !crate::utils::is_string_in_array(&klass, &IMPASSABLE_TILES);
                    let tr = TileReference {
                        animated: if tdata.properties.contains_key("animated") {
                            // ohohoho, we have an aniimated tile here!
                            if let PropertyValue::IntValue(frames) =
                                tdata.properties.get("frames").unwrap()
                            {
                                Some(crate::core::animation::BasicTileAnimation::new(
                                    *frames, 0.1,
                                ))
                            } else {
                                None
                            }
                        } else {
                            None
                        },
                        size: ivec2(w, h),
                        klass,
                        tile_image: get_file_name(
                            &image
                                .map(|f| f.source.to_str())
                                .flatten()
                                .unwrap_or("wtf.png")
                                .to_owned(),
                        )
                        .unwrap(),
                        tile_index: index,
                        tile_name: image
                            .map(|f| f.source.to_str())
                            .flatten()
                            .unwrap_or("wtf.png")
                            .to_owned(),
                        props: data,
                    };
                    (Some(tr), passable)
                }
                None => (None, true),
            };
            let cell_index = y * max_x + x;
            let cell = Cell::new(position, passable, tileref);
            vec[cell_index.to_usize().unwrap()] = Some(cell);
        }
    }
    return Cellmap::new(vec, max_x, max_y);
}
