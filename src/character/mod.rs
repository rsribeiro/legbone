use num_enum::TryFromPrimitive;

pub mod player;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Gender {
    Female,
    Male,
}

#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
pub enum FightMode {
    Offensive = 1,
    Normal = 2,
    Defensive = 3,
}

#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
pub enum FightStance {
    StandStill = 0,
    Chase = 1,

    //Since v5
    KeepDistance = 2,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, TryFromPrimitive)]
pub enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
    // None = 0xff
}

#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
pub enum HealthStatus {
    Dead = 0,
    NearlyDead = 1,
    Critical = 2,
    HeavilyWounded = 3,
    LightlyWounded = 4,
    BarelyWounded = 5,
    Healthy = 6,
    // Total = 7
}

#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
pub enum CharacterUpdateType {
    HealthStatus = 1,
    LightLevel = 2,
    Outfit = 3,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Outfit {
    pub(crate) head: u8,
    pub(crate) body: u8,
    pub(crate) legs: u8,
    pub(crate) shoes: u8,

    //Changing this value had no identifiable effect, but it has to be sent to and received from client
    //Race??? (1=human)
    //Gender???
    pub(crate) unknown_byte: u8,
}

impl Outfit {
    pub const fn new(head: u8, body: u8, legs: u8, shoes: u8) -> Self {
        Self {
            head,
            body,
            legs,
            shoes,
            unknown_byte: 0,
        }
    }

    pub const fn new_with_unknown_byte(
        head: u8,
        body: u8,
        legs: u8,
        shoes: u8,
        unknown_byte: u8,
    ) -> Self {
        Self {
            head,
            body,
            legs,
            shoes,
            unknown_byte,
        }
    }
}

#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
pub enum CharacterOutfit {
    //Since v3.0
    Human = 1,
    Orc = 5,
    Troll = 15,
    Rat = 21,
    Cyclops = 22,
    Snake = 28,
    Spider = 30,

    //Since v3.1
    Minotaur = 25,
    Rotworm = 26,
    Wolf = 27,

    //Since v4.0
    Bear = 16,
    Beholder = 17,
    Ghoul = 18,
    Deer = 31,
    Dog = 32,
    Skeleton = 33,
    Dragon = 34,
    Demon = 35,
    PoisonSpider = 36,

    //Since v5.01
    DemonSkeleton = 37,
    GiantSpider = 38,

    //Since v6.2
    OrcShaman = 6,
    OrcWarrior = 7,
    OrcBerserker = 8,
    Necromancer = 9,
    Warlock = 10,
    Hunter = 11,
    SantaClaus = 12,
    BlackSheep = 13,
    Sheep = 14,
    Slime = 19,
    MinotaurMage = 23,
    MinotaurArcher = 24,
    MinotaurGuard = 29,
    DragonLord = 39,
    FireDevil = 40,
    Lion = 41,
    PolarBear = 42,
    Scorpion = 43,
    Wasp = 44,
    Bug = 45,
    BlackKnight = 46,
    WildWarrior = 47,
    Ghost = 48,
    FireElemental = 49,
    OrcSpearman = 50,
    OrcLeader = 51,
    WinterWolf = 52,
    FrostTroll = 53,
    Witch = 54,
    Behemoth = 55,
    PirateCaptain = 56,
    Monk = 57,
    Priestess = 58,
    OrcWarlord = 59,
    Pig = 60,
    Goblin = 61,
    Elf = 62,
    ElfArcanist = 63,
    ElfScout = 64,
    Mummy = 65,
    Pirate = 66,
    StoneGolem = 67,
    Vampire = 68,
    Dwarf = 69,
    DwarfGuard = 70,
    DwarfSoldier = 71,
    Hero = 73,
    MinotaurMage2 = 74,

    //Since v6.5
    GameMaster = 75,
}
