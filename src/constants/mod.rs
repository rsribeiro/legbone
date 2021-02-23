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
enum Fluid {
    None = 0,
    Water = 1,
    Blood = 2,
    Beer = 3,
    Slime = 4,
    Lemonade = 5,
    Milk = 6,
    WineManaFluid = 7
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum ObjectUpdateType {
    Remove = 0,
    Add = 1,
    Update = 2
}
