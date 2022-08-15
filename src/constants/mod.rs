use num_enum::TryFromPrimitive;

#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
pub enum MagicEffect {
    //Since v3.0
    DrawBlood = 0,
    LoseEnergy = 1,
    Puff = 2,
    BlockHit = 3,

    //Since v4.0
    Explosion1= 4,
    Explosion2 = 5,
    Explosiom3 = 6,
    YellowRing = 7,
    GreenRing = 8,
    HitArea = 9,
    Teleport = 10,
    EnergyDamage = 11,
    BlueSparkles = 12,
    RedSparkles = 13,
    GreenSparkles = 14,
    Burn = 15,

    //Since v5.01
    SplashPoison = 16,

    //Since v6.2
    DarkDamage = 17
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum Fluid {
    None = 0,
    Blue = 1,
    Red = 2,
    Brown = 3,
    Green = 4,
    Yellow = 5,
    White = 6,
    Purple = 7
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum ObjectUpdateType {
    Remove = 0,
    Add = 1,
    Update = 2
}

impl ObjectUpdateType {
    pub fn to_protocol_103_type(self) -> Self {
        // self
        match self {
            Self::Remove => Self::Add,
            Self::Add => Self::Remove,
            Self::Update => Self::Update,
        }
    }
}
