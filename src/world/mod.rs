use tokio::{
    task,
    time::interval,
    sync::{
        RwLock,
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}
    }
};
use tokio_stream::{
    StreamExt,
    wrappers::IntervalStream
};
use message::{PlayerToWorldMessage, WorldToPlayerMessage};
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::Duration,
};

pub mod message;

pub struct World {
    sender: UnboundedSender<PlayerToWorldMessage>,
    receiver: UnboundedReceiver<PlayerToWorldMessage>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct WorldOptions {
    pub day_night_cycle_enabled: bool,
}

impl World {
    pub fn new() -> Arc<RwLock<World>> {
        let (sender, receiver) = unbounded_channel();

        Arc::new(RwLock::new(World { sender, receiver }))
    }

    pub fn sender(&self) -> UnboundedSender<PlayerToWorldMessage> {
        self.sender.clone()
    }

    pub fn init_loop(world: &Arc<RwLock<World>>, world_options: WorldOptions) {
        let senders = Arc::new(RwLock::new(BTreeMap::new()));
        task::spawn(Self::message_loop(world.clone(), senders.clone()));
        task::spawn(Self::world_loop(world.clone(), world_options, senders));
    }

    async fn message_loop(
        world: Arc<RwLock<World>>,
        senders: Arc<RwLock<BTreeMap<u32, UnboundedSender<WorldToPlayerMessage>>>>,
    ) {
        loop {
            let receiver = &mut world.write().await.receiver;
            if let Some(message) = receiver.recv().await {
                match message {
                    PlayerToWorldMessage::LoadPlayer(player_id, sender) => {
                        log::debug!("Load player {player_id}");
                        senders.write().await.insert(player_id, sender);
                    }
                    PlayerToWorldMessage::UnloadPlayer(player_id) => {
                        log::debug!("Unload player {player_id}");
                        senders.write().await.remove(&player_id);
                    }
                    PlayerToWorldMessage::Walk(player_id) => {
                        log::trace!("Received player {player_id} walk")
                    }
                }
            }
        }
    }

    async fn world_loop(
        _world: Arc<RwLock<World>>,
        world_options: WorldOptions,
        senders: Arc<RwLock<BTreeMap<u32, UnboundedSender<WorldToPlayerMessage>>>>,
    ) {
        let mut hour = 0;
        let mut interval = IntervalStream::new(interval(Duration::from_secs(3)));
        while let Some(_instant) =  interval.next().await {
            hour = if hour >= 23 { 0 } else { hour + 1 };
            let light_level = Self::hour_to_light_level(hour);

            // log::trace!("Hour: {}, light_level: {}", hour, light_level);
            for (_player_id, sender) in senders.read().await.iter() {
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

