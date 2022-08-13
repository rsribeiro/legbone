use position::Position;
use std::{
    collections::BTreeMap,
    str::FromStr
};
use once_cell::sync::OnceCell;
use anyhow::{
    Result,
    anyhow
};

pub mod position;

pub const MAP_WIDTH: u16 = 50;
pub const MAP_HEIGHT: u16 = 50;
pub const MAP_LAYERS: u8 = 16;
pub const RESPAWN_LOCATION: Position = Position::new(8, 6, 7);

pub static MAP: OnceCell<Map> = OnceCell::new();

pub enum MapType {
    FixedTile,
    Checkerboard,
    RookgaardTemple,
    File
}

impl FromStr for MapType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "FixedTile" => Ok(MapType::FixedTile),
            "Checkerboard" => Ok(MapType::Checkerboard),
            "RookgaardTemple" => Ok(MapType::RookgaardTemple),
            "File" => Ok(MapType::File),
            _ => Err("no match"),
        }
    }
}

pub fn init_map(map_type: MapType, map_arg: Option<String>) -> Result<()> {
    let map = match map_type {
        MapType::FixedTile => {
            let tile = map_arg.ok_or_else(|| anyhow!("Send numeric tile argument on map_arg."))?.parse()?;
            Map::fixed_tile(tile, MAP_WIDTH, MAP_HEIGHT, 0, 0, RESPAWN_LOCATION)
        },
        MapType::Checkerboard => Map::checkerboard_pattern(MAP_WIDTH, MAP_HEIGHT, 0, 0, RESPAWN_LOCATION),
        MapType::RookgaardTemple => Map::rookgaard_temple(MAP_WIDTH, MAP_HEIGHT, 0, 0, RESPAWN_LOCATION),
        MapType::File => {
            let _file = map_arg.ok_or_else(|| anyhow!("Send file argument on map_arg."))?;
            return Err(anyhow!("Map from file is not yet supported."));
        }
    };
    MAP.set(map).unwrap();
    Ok(())
}

#[derive(Debug)]
pub struct Map {
    pub(crate) metadata: MapMetadata,
    tiles: BTreeMap<Position,MapTile>
}

#[derive(Debug)]
pub struct MapMetadata {
    width: u16,
    height: u16,
    offset_x: u16,
    offset_y: u16,
    pub(crate) respawn_location: Position
}

#[derive(Debug)]
struct MapTile(Vec<u16>);

impl MapMetadata {
    const fn new(width: u16, height: u16, offset_x: u16, offset_y: u16, respawn_location: Position) -> Self {
        Self { width, height, offset_x, offset_y, respawn_location }
    }
}

impl MapTile {
    const fn empty() -> Self {
        MapTile(Vec::new())
    }

    fn clear(&mut self) -> &mut Self {
        self.0.clear();
        self
    }

    fn add_tile(&mut self, tile: u16) -> &mut Self {
        self.0.push(tile);
        self
    }
}

impl Map {
    fn fixed_tile(tile: u16, width: u16, height: u16, offset_x: u16, offset_y: u16, respawn_location: Position) -> Self {
        let mut map: BTreeMap<Position,MapTile> = BTreeMap::new();
        for x in 0..width {
            for y in 0..height {
                let x = x + offset_x;
                let y = y + offset_y;

                let position = Position::new(x,y,7);
                if let Some(map_tile) = map.get_mut(&position) {
                    map_tile.0.push(tile);
                } else {
                    let mut map_tile = MapTile::empty();
                    map_tile.0.push(tile);
                    map.insert(position, map_tile);
                }
            }
        }

        Self {
            metadata: MapMetadata::new(width, height, offset_x, offset_y, respawn_location),
            tiles: map
        }
    }

    fn checkerboard_pattern(width: u16, height: u16, offset_x: u16, offset_y: u16, respawn_location: Position) -> Self {
        let mut map: BTreeMap<Position,MapTile> = BTreeMap::new();
        for x in 0..width {
            for y in 0..height {
                let x = x + offset_x;
                let y = y + offset_y;

                let tile = if (x + y) % 2  == 0 {
                    0x010c
                } else {
                    0x0113
                };

                let position = Position::new(x,y,7);
                if let Some(map_tile) = map.get_mut(&position) {
                    map_tile.0.push(tile);
                } else {
                    let mut map_tile = MapTile::empty();
                    map_tile.0.push(tile);
                    map.insert(position, map_tile);
                }
            }
        }

        Self {
            metadata: MapMetadata::new(width, height, offset_x, offset_y, respawn_location),
            tiles: map
        }
    }

    fn rookgaard_temple(width: u16, height: u16, offset_x: u16, offset_y: u16, respawn_location: Position) -> Self {
        let mut map = Self::checkerboard_pattern(width, height, offset_x, offset_y, respawn_location);

        let center = map.metadata.respawn_location;
        let x_1 = center.x - 8;
        let x_2 = center.x + 8;
        let y_1 = center.y - 6;
        let y_2 = center.y + 6;

        for x in x_1+2..=x_2-2 {
            for y in y_1+2..=y_2-2 {
                map.get_map_tile(Position::new(x, y, 7)).clear().add_tile(0x0a);
            }
        }

        //water
        map.get_map_tile(Position::new(x_1 + 2, y_1 + 10, 7)).clear().add_tile(0x5a0e);
        map.get_map_tile(Position::new(x_1 + 3, y_1 + 10, 7)).clear().add_tile(0x5f0e);
        map.get_map_tile(Position::new(x_1 + 13, y_1 + 10, 7)).clear().add_tile(0x5c0e);
        map.get_map_tile(Position::new(x_1 + 14, y_1 + 10, 7)).clear().add_tile(0x000e);
        map.get_map_tile(Position::new(x_1 + 13, y_1 + 9, 7)).clear().add_tile(0x5e0e);
        map.get_map_tile(Position::new(x_1 + 14, y_1 + 9, 7)).clear().add_tile(0x5a0e);

        //marble
        map.get_map_tile(Position::new(x_1 + 5, y_1 + 9, 7)).clear().add_tile(0x010c).add_tile(0x0327);
        map.get_map_tile(Position::new(x_1 + 6, y_1 + 9, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 7, y_1 + 9, 7)).clear().add_tile(0x010c).add_tile(0x0327);
        map.get_map_tile(Position::new(x_1 + 8, y_1 + 9, 7)).clear().add_tile(0x0113);
        map.get_map_tile(Position::new(x_1 + 9, y_1 + 9, 7)).clear().add_tile(0x010c).add_tile(0x0327);
        map.get_map_tile(Position::new(x_1 + 10, y_1 + 9, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 11, y_1 + 9, 7)).clear().add_tile(0x010c).add_tile(0x0327);

        map.get_map_tile(Position::new(x_1 + 5, y_1 + 8, 7)).clear().add_tile(0x010c).add_tile(0x0224);
        map.get_map_tile(Position::new(x_1 + 6, y_1 + 8, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 7, y_1 + 8, 7)).clear().add_tile(0x0113);
        map.get_map_tile(Position::new(x_1 + 8, y_1 + 8, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 9, y_1 + 8, 7)).clear().add_tile(0x0113);
        map.get_map_tile(Position::new(x_1 + 10, y_1 + 8, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 11, y_1 + 8, 7)).clear().add_tile(0x010c).add_tile(0x0224);

        map.get_map_tile(Position::new(x_1 + 5, y_1 + 7, 7)).clear().add_tile(0x010c).add_tile(0x0327);
        map.get_map_tile(Position::new(x_1 + 6, y_1 + 7, 7)).clear().add_tile(0x0113);
        map.get_map_tile(Position::new(x_1 + 7, y_1 + 7, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 8, y_1 + 7, 7)).clear().add_tile(0x0113);
        map.get_map_tile(Position::new(x_1 + 9, y_1 + 7, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 10, y_1 + 7, 7)).clear().add_tile(0x0113);
        map.get_map_tile(Position::new(x_1 + 11, y_1 + 7, 7)).clear().add_tile(0x010c).add_tile(0x0327);

        map.get_map_tile(Position::new(x_1 + 5, y_1 + 6, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 6, y_1 + 6, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 7, y_1 + 6, 7)).clear().add_tile(0x0113);
        map.get_map_tile(Position::new(x_1 + 8, y_1 + 6, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 9, y_1 + 6, 7)).clear().add_tile(0x0113);
        map.get_map_tile(Position::new(x_1 + 10, y_1 + 6, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 11, y_1 + 6, 7)).clear().add_tile(0x010c);

        map.get_map_tile(Position::new(x_1 + 5, y_1 + 5, 7)).clear().add_tile(0x010c).add_tile(0x0327);
        map.get_map_tile(Position::new(x_1 + 6, y_1 + 5, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 7, y_1 + 5, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 8, y_1 + 5, 7)).clear().add_tile(0x0113);
        map.get_map_tile(Position::new(x_1 + 9, y_1 + 5, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 10, y_1 + 5, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 11, y_1 + 5, 7)).clear().add_tile(0x010c).add_tile(0x0327);

        map.get_map_tile(Position::new(x_1 + 5, y_1 + 4, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 6, y_1 + 4, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 7, y_1 + 4, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 8, y_1 + 4, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 9, y_1 + 4, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 10, y_1 + 4, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 11, y_1 + 4, 7)).clear().add_tile(0x010c);

        map.get_map_tile(Position::new(x_1 + 5, y_1 + 3, 7)).clear().add_tile(0x010c).add_tile(0x0327);
        map.get_map_tile(Position::new(x_1 + 6, y_1 + 3, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 7, y_1 + 3, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 8, y_1 + 3, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 9, y_1 + 3, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 10, y_1 + 3, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 11, y_1 + 3, 7)).clear().add_tile(0x010c).add_tile(0x0327);

        map.get_map_tile(Position::new(x_1 + 5, y_1 + 2, 7)).clear().add_tile(0x010c).add_tile(0x0224);
        map.get_map_tile(Position::new(x_1 + 6, y_1 + 2, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 7, y_1 + 2, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 8, y_1 + 2, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 9, y_1 + 2, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 10, y_1 + 2, 7)).clear().add_tile(0x010c);
        map.get_map_tile(Position::new(x_1 + 11, y_1 + 2, 7)).clear().add_tile(0x010c).add_tile(0x0224);

        //stone on top
        map.get_map_tile(Position::new(x_1 + 2, y_1 + 2, 7)).clear().add_tile(0xb00a);
        map.get_map_tile(Position::new(x_1 + 2, y_1 + 3, 7)).clear().add_tile(0xac0a);
        map.get_map_tile(Position::new(x_1 + 3, y_1 + 2, 7)).clear().add_tile(0xac0a);

        //trees
        map.get_map_tile(Position::new(x_1 + 13, y_1 + 2, 7)).add_tile(0x01a3);
        map.get_map_tile(Position::new(x_1 + 13, y_1 + 4, 7)).add_tile(0x00a3);
        map.get_map_tile(Position::new(x_1 + 14, y_1 + 4, 7)).add_tile(0x01a3);
        map.get_map_tile(Position::new(x_1 + 15, y_1 + 4, 7)).add_tile(0x00a3);
        map.get_map_tile(Position::new(x_1 + 15, y_1 + 3, 7)).add_tile(0x00a0);

        // TESTE
        // map.get_map_tile(Position::new(x_1 + 8, y_1 + 2, 7)).add_tile(0x0547);

        map
    }

    fn get_map_tile(&mut self, position: Position) -> &mut MapTile {
        self.tiles.entry(position).or_insert_with(MapTile::empty);
        self.tiles.get_mut(&position).unwrap()
    }

    pub fn get_tile(&self, position: Position) -> Option<&[u16]> {
        if position.x >= self.metadata.offset_x
            && position.x < self.metadata.offset_x + self.metadata.width
            && position.y >= self.metadata.offset_y
            && position.y < self.metadata.offset_y + self.metadata.height {
            self.tiles.get(&position).map(|t|t.0.as_slice())
        } else if position.z == 7 {
            Some(&[0x000e; 1]) //water
        } else {
            None
        }
    }
}
