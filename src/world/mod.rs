use async_std::{
    prelude::*,
    task,
    stream
};
use flume::{
    unbounded,
    Sender,
    Receiver
};
use std::{
    time::Duration,
    sync::{
        Arc,
        RwLock
    },
    collections::BTreeMap
};
use message::{
    PlayerToWorldMessage,
    WorldToPlayerMessage
};

pub mod message;

pub struct World {
    sender: Sender<PlayerToWorldMessage>,
    receiver: Receiver<PlayerToWorldMessage>
}

impl World {
    pub fn new() -> Arc<RwLock<World>> {
        let (sender, receiver) = unbounded();

        Arc::new(RwLock::new(World {
            sender,
            receiver
        }))
    }

    pub fn sender(&self) -> Sender<PlayerToWorldMessage> {
        self.sender.clone()
    }

    pub fn init_loop(world: &Arc<RwLock<World>>) {
        let senders = Arc::new(RwLock::new(BTreeMap::new()));
        task::spawn(Self::message_loop(world.clone(), senders.clone()));
        task::spawn(Self::world_loop(world.clone(), senders.clone()));
    }

    async fn message_loop(world: Arc<RwLock<World>>, senders: Arc<RwLock<BTreeMap<u32, Sender<WorldToPlayerMessage>>>>) {
        loop {
            match world.read().unwrap().receiver.recv() {
                Ok(message) => {
                    match message {
                        PlayerToWorldMessage::LoadPlayer(player_id, sender) => {
                            log::debug!("Load player {}", player_id);
                            senders.write().unwrap().insert(player_id, sender);
                        },
                        PlayerToWorldMessage::UnloadPlayer(player_id) => {
                            log::debug!("Unload player {}", player_id);
                            senders.write().unwrap().remove(&player_id);
                        }
                        PlayerToWorldMessage::Walk(player_id) => log::trace!("Received player {} walk", player_id)
                    }
                },
                Err(err) => log::error!("{}", err)
            }
        }
    }

    async fn world_loop(_world: Arc<RwLock<World>>, senders: Arc<RwLock<BTreeMap<u32, Sender<WorldToPlayerMessage>>>>) {
        let mut hour = 0;
        let mut interval = stream::interval(Duration::from_secs(3));
        while let Some(_) = interval.next().await {
            hour = if hour >= 23 {
                0
            } else {
                hour + 1
            };
            let light_level = Self::hour_to_light_level(hour);

            // log::trace!("Hour: {}, light_level: {}", hour, light_level);
            for (_player_id, sender) in senders.read().unwrap().iter() {
                let _ = sender.send(WorldToPlayerMessage::WorldLight(light_level));
            }
        }
    }

    const fn hour_to_light_level(hour: u8) -> u8 {
        match hour {
            0|1|2|3|4|22|23 => 1,
            5|21 => 2,
            6|20 => 3,
            7|19 => 4,
            8|18 => 5,
            9|10|11|12|13|14|15|16|17 => 6,
            _ => 6
        }
    }
}
