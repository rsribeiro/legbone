use crate::map::MapType;
use anyhow::{Result, anyhow};
use std::net::Ipv4Addr;
use std::{
    sync::OnceLock,
    path::Path
};
use serde_derive::Deserialize;

pub static CONFIG: OnceLock<Config> = OnceLock::new();

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

pub fn init(config: &Path) -> Result<()> {
    if config.exists() {
        let config = toml::from_str::<Config>(&std::fs::read_to_string(config)?)?;
        CONFIG.set(config).unwrap();
        Ok(())
    } else {
        Err(anyhow!("File {config:?} does not exist"))
    }
}
