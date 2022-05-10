use num_enum::TryFromPrimitive;
use crate::character::Direction;

#[repr(u16)]
#[derive(Debug, TryFromPrimitive)]
pub enum HeaderSend {
    Unknown0x0000 = 0x0000,
    Login = 0x0001,
    Error = 0x0002,
    DataWindow = 0x0003,
    Info = 0x0004,
    MessageOfTheDay = 0x0005,
    Map = 0x000a,
    MoveOneTileNorth = 0x000b,
    MoveOneTileEast = 0x000c,
    MoveOneTileSouth = 0x000d,
    MoveOneTileWest = 0x000e,
    Unknown0x000f = 0x000f,
    CloseContainer = 0x0012,
    OpenContainer = 0x0013,
    EquippedItem = 0x0014,
    RemoveEquippedItem = 0x0015,
    UpdateInventoryItem = 0x0016,
    UpdateObject = 0x0019,
    MagicEffect = 0x001a,
    ProjectileEffect = 0x001b,
    Text = 0x0023,
    HouseText = 0x0024,
    WorldLight = 0x0028,
    UpdateCharacter = 0x0032,
    Unknown0x0033 = 0x0033,
    Stats = 0x003c,
    Skills = 0x003d,
    Unknown0x0064 = 0x0064,
    Chat = 0x0065,
    UserList = 0x0066,
    UserInfo = 0x0067,
    StatusMessage = 0x0068,
    Echo = 0x00c8,
}

impl From<Direction> for HeaderSend {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::North => Self::MoveOneTileNorth,
            Direction::West => Self::MoveOneTileWest,
            Direction::South => Self::MoveOneTileSouth,
            Direction::East => Self::MoveOneTileEast,
        }
    }
}

#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
pub enum AuxiliaryHeaderSend {
    ChangeDirection = 0xfa,
    Character = 0xfb,
}

#[repr(u16)]
#[derive(Debug, TryFromPrimitive)]
pub enum HeaderReceive {
    UserList = 0x0003,
    PlayerInfo = 0x0004,
    Walk = 0x0005,
    AutoWalk = 0x0006,
    LookAt = 0x0007,
    Chat = 0x0009,
    ChangeDirection = 0x000a,
    Comment = 0x000b,
    Push = 0x0014,
    UseItem = 0x001e,
    CloseContainer = 0x001f,
    RequestChangeData = 0x0020,
    SetData = 0x0021,
    SetText = 0x0023,
    HouseText = 0x0024,
    ChangeMode = 0x0032,
    ExitBattle = 0x0033,
    SetTarget = 0x0034,
    Echo = 0x00C8,
    Logout = 0x00ff
}
