use async_std::{prelude::*, stream, task};
use flume::{unbounded, Receiver, Sender};
use message::{PlayerToWorldMessage, WorldToPlayerMessage};
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
    time::Duration,
};

pub mod message;

pub struct World {
    sender: Sender<PlayerToWorldMessage>,
    receiver: Receiver<PlayerToWorldMessage>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct WorldOptions {
    pub day_night_cycle_enabled: bool,
}

impl World {
    pub fn new() -> Arc<RwLock<World>> {
        let (sender, receiver) = unbounded();

        Arc::new(RwLock::new(World { sender, receiver }))
    }

    pub fn sender(&self) -> Sender<PlayerToWorldMessage> {
        self.sender.clone()
    }

    pub fn init_loop(world: &Arc<RwLock<World>>, world_options: WorldOptions) {
        let senders = Arc::new(RwLock::new(BTreeMap::new()));
        task::spawn(Self::message_loop(world.clone(), senders.clone()));
        task::spawn(Self::world_loop(world.clone(), world_options, senders));
    }

    async fn message_loop(
        world: Arc<RwLock<World>>,
        senders: Arc<RwLock<BTreeMap<u32, Sender<WorldToPlayerMessage>>>>,
    ) {
        loop {
            match world.read().unwrap().receiver.recv() {
                Ok(message) => match message {
                    PlayerToWorldMessage::LoadPlayer(player_id, sender) => {
                        log::debug!("Load player {}", player_id);
                        senders.write().unwrap().insert(player_id, sender);
                    }
                    PlayerToWorldMessage::UnloadPlayer(player_id) => {
                        log::debug!("Unload player {}", player_id);
                        senders.write().unwrap().remove(&player_id);
                    }
                    PlayerToWorldMessage::Walk(player_id) => {
                        log::trace!("Received player {} walk", player_id)
                    }
                },
                Err(err) => log::error!("{}", err),
            }
        }
    }

    async fn world_loop(
        _world: Arc<RwLock<World>>,
        world_options: WorldOptions,
        senders: Arc<RwLock<BTreeMap<u32, Sender<WorldToPlayerMessage>>>>,
    ) {
        let mut hour = 0;
        let mut interval = stream::interval(Duration::from_secs(3));
        while interval.next().await.is_some() {
            hour = if hour >= 23 { 0 } else { hour + 1 };
            let light_level = Self::hour_to_light_level(hour);

            // log::trace!("Hour: {}, light_level: {}", hour, light_level);
            for (_player_id, sender) in senders.read().unwrap().iter() {
                if world_options.day_night_cycle_enabled {
                    let _ = sender.send(WorldToPlayerMessage::WorldLight(light_level));
                }
            }
        }
    }

    const fn hour_to_light_level(hour: u8) -> u8 {
        match hour {
            0..=4 | 22..=23 => 1,
            5 | 21 => 2,
            6 | 20 => 3,
            7 | 19 => 4,
            8 | 18 => 5,
            9..=17 => 6,
            _ => 6,
        }
    }
}
