mod character;
mod chat;
pub mod config;
mod constants;
mod io;
pub mod map;
pub mod network;
mod persistence;
pub mod world;

use clap::{ArgAction, Parser};
use num_enum::TryFromPrimitive;

#[repr(u16)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, TryFromPrimitive)]
pub enum Protocol {
    Tibia103 = 103,
    Tibia300 = 300,
    Tibia310 = 310,
    Tibia400 = 400,
    Tibia412 = 412,
    Tibia501 = 501,
    Tibia510 = 510,
    // Some files from clients 5.11, 6.0 and 6.1 can be found on the internet, but they are incomplete
    // Tibia511 = 511,
    // Tibia600 = 600,
    // Tibia610 = 610,
    Tibia620 = 620,
    Tibia630 = 630,
    Tibia640 = 640,
    Tibia650 = 650,
    Tibia661 = 661,
    Tibia694 = 694,
}

#[derive(Parser)]
#[clap(version = "0.1.0", author = "Ricardo")]
pub struct Opts {
    #[clap(
        short,
        long,
        action = ArgAction::Count,
        help = "Verbosity level (-v or -vv)"
    )]
    pub verbose: u8,
}
