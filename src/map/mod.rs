use crate::{config::Map as MapConfig, constants::Fluid};
use anyhow::{anyhow, Result};
use once_cell::sync::OnceCell;
use position::Position;
use serde_derive::Deserialize;
use std::collections::BTreeMap;

pub mod position;

const MAP_WIDTH: u16 = 100;
const MAP_HEIGHT: u16 = 100;
#[allow(dead_code)]
const MAP_LAYERS: u8 = 16;
const RESPAWN_LOCATION: Position = Position::new(50, 50, 7);

pub static MAP: OnceCell<Map> = OnceCell::new();

#[derive(Deserialize, Debug)]
pub enum MapType {
    FixedTile,
    Checkerboard,
    RookgaardTemple,
    File,
}

pub fn init_map(config: &MapConfig) -> Result<()> {
    let map = match &config.map_type {
        MapType::FixedTile => {
            let tile = config.tile.expect("No map tile specified");
            Map::fixed_tile(tile, MAP_WIDTH, MAP_HEIGHT, 0, 0, RESPAWN_LOCATION)
        }
        MapType::Checkerboard => {
            Map::checkerboard_pattern(MAP_WIDTH, MAP_HEIGHT, 0, 0, RESPAWN_LOCATION)
        }
        MapType::RookgaardTemple => {
            Map::rookgaard_temple(MAP_WIDTH, MAP_HEIGHT, 0, 0, RESPAWN_LOCATION)
        }
        MapType::File => {
            let _file = config.file.as_ref().expect("No map file specified");
            return Err(anyhow!("Map from file is not yet supported."));
        }
    };
    MAP.set(map).unwrap();
    Ok(())
}

#[derive(Debug)]
pub struct Map {
    pub(crate) metadata: MapMetadata,
    tiles: BTreeMap<Position, Tile>,
}

#[derive(Debug)]
pub struct MapMetadata {
    width: u16,
    height: u16,
    offset_x: u16,
    offset_y: u16,
    pub(crate) respawn_location: Position,
}

#[derive(Debug)]
struct Tile(Vec<TileObject>);

#[derive(Debug, Clone, Copy)]
pub enum TileObject {
    Other(u16),
    FluidContainer(u16, Fluid),
    LightSource(u16, u8),
    Stackable(u16, u8),
}

impl Map {
    fn fixed_tile(
        tile_id: u16,
        width: u16,
        height: u16,
        offset_x: u16,
        offset_y: u16,
        respawn_location: Position,
    ) -> Map {
        let mut map: BTreeMap<Position, Tile> = BTreeMap::new();
        for x in 0..width {
            for y in 0..height {
                let x = x + offset_x;
                let y = y + offset_y;

                let position = Position::new(x, y, 7);
                if let Some(tile) = map.get_mut(&position) {
                    tile.push(TileObject::Other(tile_id));
                } else {
                    let tile = Tile::with_object(TileObject::Other(tile_id));
                    map.insert(position, tile);
                }
            }
        }

        Map {
            metadata: MapMetadata::new(width, height, offset_x, offset_y, respawn_location),
            tiles: map,
        }
    }

    fn checkerboard_pattern(
        width: u16,
        height: u16,
        offset_x: u16,
        offset_y: u16,
        respawn_location: Position,
    ) -> Map {
        let mut map: BTreeMap<Position, Tile> = BTreeMap::new();
        for x in 0..width {
            for y in 0..height {
                let x = x + offset_x;
                let y = y + offset_y;

                let tile_id = if (x + y) % 2 == 0 { 0x010c } else { 0x0113 };

                let position = Position::new(x, y, 7);
                if let Some(tile) = map.get_mut(&position) {
                    tile.push(TileObject::Other(tile_id));
                } else {
                    let tile = Tile::with_object(TileObject::Other(tile_id));
                    map.insert(position, tile);
                }
            }
        }

        Map {
            metadata: MapMetadata::new(width, height, offset_x, offset_y, respawn_location),
            tiles: map,
        }
    }

    fn rookgaard_temple(
        width: u16,
        height: u16,
        offset_x: u16,
        offset_y: u16,
        respawn_location: Position,
    ) -> Map {
        let mut map =
            Map::checkerboard_pattern(width, height, offset_x, offset_y, respawn_location);

        let center = map.metadata.respawn_location;
        let x_1 = center.x - 8;
        let x_2 = center.x + 8;
        let y_1 = center.y - 6;
        let y_2 = center.y + 6;

        for x in x_1 + 2..=x_2 - 2 {
            for y in y_1 + 2..=y_2 - 2 {
                map.get_tile(Position::new(x, y, 7))
                    .clear()
                    .push(TileObject::Other(0x0a));
            }
        }

        //water
        map.get_tile(Position::new(x_1 + 2, y_1 + 10, 7))
            .push(TileObject::Other(0x5a0e));
        map.get_tile(Position::new(x_1 + 3, y_1 + 10, 7))
            .push(TileObject::Other(0x5f0e));
        map.get_tile(Position::new(x_1 + 13, y_1 + 10, 7))
            .push(TileObject::Other(0x5c0e));
        map.get_tile(Position::new(x_1 + 14, y_1 + 10, 7))
            .push(TileObject::Other(0x000e));
        map.get_tile(Position::new(x_1 + 13, y_1 + 9, 7))
            .push(TileObject::Other(0x5e0e));
        map.get_tile(Position::new(x_1 + 14, y_1 + 9, 7))
            .push(TileObject::Other(0x5a0e));

        //marble
        map.get_tile(Position::new(x_1 + 5, y_1 + 9, 7))
            .push(TileObject::Other(0x010c))
            .push(TileObject::Other(0x0327));
        map.get_tile(Position::new(x_1 + 6, y_1 + 9, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 7, y_1 + 9, 7))
            .push(TileObject::Other(0x010c))
            .push(TileObject::Other(0x0327));
        map.get_tile(Position::new(x_1 + 8, y_1 + 9, 7))
            .push(TileObject::Other(0x0113));
        map.get_tile(Position::new(x_1 + 9, y_1 + 9, 7))
            .push(TileObject::Other(0x010c))
            .push(TileObject::Other(0x0327));
        map.get_tile(Position::new(x_1 + 10, y_1 + 9, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 11, y_1 + 9, 7))
            .push(TileObject::Other(0x010c))
            .push(TileObject::Other(0x0327));

        map.get_tile(Position::new(x_1 + 5, y_1 + 8, 7))
            .push(TileObject::Other(0x010c))
            .push(TileObject::LightSource(0x0072, 6));
        map.get_tile(Position::new(x_1 + 6, y_1 + 8, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 7, y_1 + 8, 7))
            .push(TileObject::Other(0x0113));
        map.get_tile(Position::new(x_1 + 8, y_1 + 8, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 9, y_1 + 8, 7))
            .push(TileObject::Other(0x0113));
        map.get_tile(Position::new(x_1 + 10, y_1 + 8, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 11, y_1 + 8, 7))
            .push(TileObject::Other(0x010c))
            .push(TileObject::LightSource(0x0072, 6));

        map.get_tile(Position::new(x_1 + 5, y_1 + 7, 7))
            .push(TileObject::Other(0x010c))
            .push(TileObject::Other(0x0327));
        map.get_tile(Position::new(x_1 + 6, y_1 + 7, 7))
            .push(TileObject::Other(0x0113));
        map.get_tile(Position::new(x_1 + 7, y_1 + 7, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 8, y_1 + 7, 7))
            .push(TileObject::Other(0x0113));
        map.get_tile(Position::new(x_1 + 9, y_1 + 7, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 10, y_1 + 7, 7))
            .push(TileObject::Other(0x0113));
        map.get_tile(Position::new(x_1 + 11, y_1 + 7, 7))
            .push(TileObject::Other(0x010c))
            .push(TileObject::Other(0x0327));

        map.get_tile(Position::new(x_1 + 5, y_1 + 6, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 6, y_1 + 6, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 7, y_1 + 6, 7))
            .push(TileObject::Other(0x0113));
        map.get_tile(Position::new(x_1 + 8, y_1 + 6, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 9, y_1 + 6, 7))
            .push(TileObject::Other(0x0113));
        map.get_tile(Position::new(x_1 + 10, y_1 + 6, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 11, y_1 + 6, 7))
            .push(TileObject::Other(0x010c));

        map.get_tile(Position::new(x_1 + 5, y_1 + 5, 7))
            .push(TileObject::Other(0x010c))
            .push(TileObject::Other(0x0327));
        map.get_tile(Position::new(x_1 + 6, y_1 + 5, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 7, y_1 + 5, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 8, y_1 + 5, 7))
            .push(TileObject::Other(0x0113));
        map.get_tile(Position::new(x_1 + 9, y_1 + 5, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 10, y_1 + 5, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 11, y_1 + 5, 7))
            .push(TileObject::Other(0x010c))
            .push(TileObject::Other(0x0327));

        map.get_tile(Position::new(x_1 + 5, y_1 + 4, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 6, y_1 + 4, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 7, y_1 + 4, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 8, y_1 + 4, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 9, y_1 + 4, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 10, y_1 + 4, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 11, y_1 + 4, 7))
            .push(TileObject::Other(0x010c));

        map.get_tile(Position::new(x_1 + 5, y_1 + 3, 7))
            .push(TileObject::Other(0x010c))
            .push(TileObject::Other(0x0327));
        map.get_tile(Position::new(x_1 + 6, y_1 + 3, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 7, y_1 + 3, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 8, y_1 + 3, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 9, y_1 + 3, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 10, y_1 + 3, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 11, y_1 + 3, 7))
            .push(TileObject::Other(0x010c))
            .push(TileObject::Other(0x0327));

        map.get_tile(Position::new(x_1 + 5, y_1 + 2, 7))
            .push(TileObject::Other(0x010c))
            .push(TileObject::LightSource(0x0072, 6));
        map.get_tile(Position::new(x_1 + 6, y_1 + 2, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 7, y_1 + 2, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 8, y_1 + 2, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 9, y_1 + 2, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 10, y_1 + 2, 7))
            .push(TileObject::Other(0x010c));
        map.get_tile(Position::new(x_1 + 11, y_1 + 2, 7))
            .push(TileObject::Other(0x010c))
            .push(TileObject::LightSource(0x0072, 6));

        //stone on top
        map.get_tile(Position::new(x_1 + 2, y_1 + 2, 7))
            .push(TileObject::Other(0xb00a));
        map.get_tile(Position::new(x_1 + 2, y_1 + 3, 7))
            .push(TileObject::Other(0xac0a));
        map.get_tile(Position::new(x_1 + 3, y_1 + 2, 7))
            .push(TileObject::Other(0xac0a));

        //trees
        map.get_tile(Position::new(x_1 + 13, y_1 + 2, 7))
            .push(TileObject::Other(0x01a3));
        map.get_tile(Position::new(x_1 + 13, y_1 + 4, 7))
            .push(TileObject::Other(0x00a3));
        map.get_tile(Position::new(x_1 + 14, y_1 + 4, 7))
            .push(TileObject::Other(0x01a3));
        map.get_tile(Position::new(x_1 + 15, y_1 + 4, 7))
            .push(TileObject::Other(0x00a3));
        map.get_tile(Position::new(x_1 + 15, y_1 + 3, 7))
            .push(TileObject::Other(0x00a0));

        map
    }

    fn get_tile(&mut self, position: Position) -> &mut Tile {
        self.tiles.entry(position).or_insert_with(Tile::empty);
        self.tiles.get_mut(&position).unwrap()
    }

    pub fn get_tile_objects(&self, position: Position) -> Option<&[TileObject]> {
        if position.x >= self.metadata.offset_x
            && position.x < self.metadata.offset_x + self.metadata.width
            && position.y >= self.metadata.offset_y
            && position.y < self.metadata.offset_y + self.metadata.height
        {
            self.tiles.get(&position).map(|t| t.0.as_slice())
        } else if position.z == 7 {
            Some(&[TileObject::Other(0x000e)]) //water
        } else {
            None
        }
    }
}

impl MapMetadata {
    const fn new(
        width: u16,
        height: u16,
        offset_x: u16,
        offset_y: u16,
        respawn_location: Position,
    ) -> MapMetadata {
        Self {
            width,
            height,
            offset_x,
            offset_y,
            respawn_location,
        }
    }
}

impl Tile {
    const fn empty() -> Tile {
        Tile(Vec::new())
    }

    fn with_object(tile_object: TileObject) -> Tile {
        Tile(vec![tile_object])
    }

    fn clear(&mut self) -> &mut Tile {
        self.0.clear();
        self
    }

    fn push(&mut self, object: TileObject) -> &mut Tile {
        self.0.push(object);
        self
    }
}
