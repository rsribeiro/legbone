use crate::character::Direction;
use num_enum::TryFromPrimitive;

#[repr(u16)]
#[derive(Debug, TryFromPrimitive)]
pub enum HeaderSend {
    //103 = 00 00 01 00 02 00 03 00 04 00 05 00 0A 00 0B 00 0C 00 0D 00 0E       00 12 00 13 00 14 00 15       00 19                                                             00 64 00 65 00 66 00 67 00 68 00 C8 00
    //300 =       01 00 02 00 03 00 04 00 05 00 0A 00 0B 00 0C 00 0D 00 0E 00 0F 00 12 00 13 00 14 00 15 00 16 00 19 00 1A       00 23       00 28 00 32 00 33       00 3C       00 64 00 65 00 66 00 67 00 68 00 C8 00
    //310 =       01 00 02 00 03 00 04 00 05 00 0A 00 0B 00 0C 00 0D 00 0E 00 0F 00 12 00 13 00 14 00 15 00 16 00 19 00 1A 00 1B 00 23       00 28 00 32 00 33       00 3C       00 64 00 65 00 66 00 67 00 68 00 C8 00
    //400 =       01 00 02 00 03 00 04 00 05 00 0A 00 0B 00 0C 00 0D 00 0E 00 0F 00 12 00 13 00 14 00 15 00 16 00 19 00 1A 00 1B 00 23       00 28 00 32 00 33 00 34 00 3C 00 3D 00 64 00 65 00 66 00 67 00 68 00 C8 00
    //501 =       01 00 02 00 03 00 04 00 05 00 0A 00 0B 00 0C 00 0D 00 0E 00 0F 00 12 00 13 00 14 00 15 00 16 00 19 00 1A 00 1B 00 23       00 28 00 32 00 33 00 34 00 3C 00 3D 00 64 00 65 00 66 00 67 00 68 00 C8 00
    //620 =       01 00 02 00 03 00 04 00 05 00 0A 00 0B 00 0C 00 0D 00 0E 00 0F 00 12 00 13 00 14 00 15 00 16 00 19 00 1A 00 1B 00 23       00 28 00 32 00 33       00 3C 00 3D 00 64 00 65 00 66 00 67 00 68 00 C8 00
    //630 =       01 00 02 00 03 00 04 00 05 00 0A 00 0B 00 0C 00 0D 00 0E 00 0F 00 12 00 13 00 14 00 15 00 16 00 19 00 1A 00 1B 00 23       00 28 00 32 00 33       00 3C 00 3D 00 64 00 65 00 66 00 67 00 68 00 C8 00
    //640 =       01 00 02 00 03 00 04 00 05 00 0A 00 0B 00 0C 00 0D 00 0E 00 0F 00 12 00 13 00 14 00 15 00 16 00 19 00 1A 00 1B 00 23 00 24 00 28 00 32 00 33       00 3C 00 3D 00 64 00 65 00 66 00 67 00 68 00 C8 00
    //650 =       01 00 02 00 03 00 04 00 05 00 0A 00 0B 00 0C 00 0D 00 0E 00 0F 00 12 00 13 00 14 00 15 00 16 00 19 00 1A 00 1B 00 23 00 24 00 28 00 32 00 33       00 3C 00 3D 00 64 00 65 00 66 00 67 00 68 00 C8 00

    //Exclusive to protocol Tibia103
    Unknown0x0000 = 0x0000,

    //Since Tibia103
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
    CloseContainer = 0x0012,
    OpenContainer = 0x0013,
    EquippedItem = 0x0014,
    RemoveEquippedItem = 0x0015,
    UpdateObject = 0x0019,
    GreenChat = 0x0064,
    Chat = 0x0065,
    UserList = 0x0066,
    UserInfo = 0x0067,
    StatusMessage = 0x0068,
    Echo = 0x00c8,

    //Since Tibia300
    Unknown0x000f = 0x000f,
    UpdateInventoryItem = 0x0016,
    MagicEffect = 0x001a,
    Text = 0x0023,
    WorldLight = 0x0028,
    UpdateCharacter = 0x0032,
    Unknown0x0033 = 0x0033,
    Stats = 0x003c,

    //Since Tibia310
    ProjectileEffect = 0x001b,

    //Since Tibia 400
    Skills = 0x003d,

    //Exclusive to protocols Tibia400 and Tibia501
    Unknown0x0034 = 0x0034,

    //Since Tibia640
    HouseText = 0x0024,
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
    Logout = 0x00ff,
}
