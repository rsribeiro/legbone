use super::{Gender, Outfit};
use crate::map::position::Position;
use num_enum::TryFromPrimitive;

#[derive(Clone)]
pub struct Player {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) position: Position,
    pub(crate) skills: Skills,
    pub(crate) stats: Stats,
    pub(crate) outfit: Outfit,
    pub(crate) gender: Gender,
}

#[derive(Copy, Clone, Debug)]
pub struct Skills {
    pub(crate) sword: u8,
    pub(crate) club: u8,
    pub(crate) axe: u8,
    pub(crate) distance: u8,
    pub(crate) shield: u8,
    pub(crate) fist: u8,
    pub(crate) fishing: u8,
    pub(crate) gauche: u8,
    pub(crate) missile: u8,
}

#[derive(Copy, Clone, Debug)]
pub struct Stats {
    pub(crate) health_points: u16,
    pub(crate) capacity: u16,
    pub(crate) intelligence: u8,
    pub(crate) strength: u8,
    pub(crate) dexterity: u8,
    pub(crate) experience_points: u32,
    pub(crate) experience_level: u8,
    pub(crate) mana_points: u16,
    pub(crate) magic_level: u8,
    pub(crate) ammunition: u16,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
pub enum InventorySlot {
    Helmet = 1,
    Necklace = 2,
    Bag = 3,
    Armor = 4,
    RightHand = 5,
    LeftHand = 6,
    Legs = 7,
    Boots = 8,
}
