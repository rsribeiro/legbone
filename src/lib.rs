pub mod world;
pub mod network;
pub mod map;
mod character;
mod constants;
mod chat;
mod io;
mod persistence;

use num_enum::TryFromPrimitive;
use clap::Clap;
use async_std::net::Ipv4Addr;
use crate::map::MapType;

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

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Ricardo")]
pub struct Opts {
    #[clap(short, long, default_value="127.0.0.1", about="Server IP Address (v4)")]
    pub ip: Ipv4Addr,
    #[clap(short, long, default_value="7171", about="Server port")]
    pub port: u16,
    #[clap(short, long, default_value="Checkerboard", about="Type of map (Checkerboard, FixedTile, RookgaardTemple or File")]
    pub map: MapType,
    #[clap(long, about="Tile if FixedTile map, file if File map")]
    pub map_arg: Option<String>,
    #[clap(short, long, parse(from_occurrences), about="Verbosity level (-v or -vv)")]
    pub verbose: u32,
    #[clap(long, about="Disable debug chat commands")]
    pub nodebug: bool
}
