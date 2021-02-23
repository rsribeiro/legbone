use std::{
    fmt::Display,
    ops::{
        Add,
        Sub
    }, 
    convert::TryInto
};
use crate::{
    character::{
        player::InventorySlot,
        Direction
    }
};
use anyhow::Result;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Position {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) z: u8
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self.x, self.y, self.z)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PositionQualifier {
    None,
    Container(u16,u8),
    Inventory(InventorySlot)
}

impl Position {
    pub const fn new(x: u16, y:u16, z: u8) -> Self {
        Self{x,y,z}
    }

    pub fn get_qualifier(&self) -> Result<PositionQualifier> {
        if self.x == 0xffff {
            if self.y & 0x40 == 0 {
                Ok(PositionQualifier::Inventory((self.y as u8).try_into()?))
            } else {
                Ok(PositionQualifier::Container(self.y - 0x40, self.z))
            }
        } else {
            Ok(PositionQualifier::None)
        }
    }
}

impl Add<(i16,i16,i8)> for Position {
    type Output = Self;

    fn add(self, rhs: (i16,i16,i8)) -> Self::Output {
        Self {
            x: (self.x as i16 + rhs.0) as u16,
            y: (self.y as i16 + rhs.1) as u16,
            z: (self.z as i8 + rhs.2) as u8,
        }
    }
}

impl Add<Direction> for Position {
    type Output = Self;

    fn add(self, rhs: Direction) -> Self::Output {
        let rhs = match rhs {
            Direction::North => (0,-1,0),
            Direction::East => (1,0,0),
            Direction::South => (0,1,0),
            Direction::West => (-1,0,0),
        };
        self + rhs
    }
}

impl Sub<(i16,i16,i8)> for Position {
    type Output = Self;

    fn sub(self, rhs: (i16,i16,i8)) -> Self::Output {
        Self {
            x: (self.x as i16 - rhs.0) as u16,
            y: (self.y as i16 - rhs.1) as u16,
            z: (self.z as i8 - rhs.2) as u8,
        }
    }
}
