use crate::map::MapType;
use anyhow::Result;
use async_std::net::Ipv4Addr;
use once_cell::sync::OnceCell;
use serde_derive::Deserialize;
use std::fs;

pub static CONFIG: OnceCell<Config> = OnceCell::new();

#[derive(Deserialize, Debug)]
pub struct Config {
    pub server: Server,
    pub world: World,
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub ip: Ipv4Addr,
    pub port: u16,
    pub debug_commands: bool,
}

#[derive(Deserialize, Debug)]
pub struct World {
    pub map: Map,
    pub day_night_cycle: bool,
}

#[derive(Deserialize, Debug)]
pub struct Map {
    pub map_type: MapType,
    pub file: Option<String>,
    pub tile: Option<u16>,
}

pub fn init(config: &str) -> Result<()> {
    let config: Config = toml::from_slice(&fs::read(config)?)?;
    CONFIG.set(config).unwrap();
    Ok(())
}
