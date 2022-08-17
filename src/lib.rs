mod character;
mod chat;
mod constants;
mod io;
pub mod map;
pub mod network;
mod persistence;
pub mod world;

use crate::map::MapType;
use async_std::net::Ipv4Addr;
use clap::Parser;
use num_enum::TryFromPrimitive;

#[repr(u16)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, TryFromPrimitive)]
pub enum Protocol {
    // Clients 6.0 and 6.1 can be found on the internet, but are incomplete (no data files)
    Tibia103 = 103,
    Tibia300 = 300,
    Tibia310 = 310,
    Tibia400 = 400,
    Tibia501 = 501,
    // Tibia600 = 600,
    // Tibia610 = 610,
    Tibia620 = 620,
    Tibia630 = 630,
    Tibia640 = 640,
    Tibia650 = 650,
}

#[derive(Parser)]
#[clap(version = "0.1.0", author = "Ricardo")]
pub struct Opts {
    #[clap(
        short,
        long,
        default_value = "0.0.0.0",
        help = "Server IP Address (v4)"
    )]
    pub ip: Ipv4Addr,
    #[clap(short, long, default_value = "7171", help = "Server port")]
    pub port: u16,
    #[clap(
        short,
        long,
        parse(from_occurrences),
        help = "Verbosity level (-v or -vv)"
    )]
    pub verbose: u32,
    #[clap(long, help = "Disable debug chat commands")]
    pub nodebug: bool,

    //Game world options
    #[clap(
        short,
        long,
        default_value = "Checkerboard",
        help = "Type of map (Checkerboard, FixedTile, RookgaardTemple or File"
    )]
    pub map: MapType,
    #[clap(long, help = "Tile if FixedTile map, file if File map")]
    pub map_arg: Option<String>,
    #[clap(long, help = "Enable day/night cycle")]
    pub day_night_cycle_enabled: bool,
}
