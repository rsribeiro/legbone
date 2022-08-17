use flume::Sender;

pub enum PlayerToWorldMessage {
    LoadPlayer(u32, Sender<WorldToPlayerMessage>),
    UnloadPlayer(u32),
    Walk(u32),
}

pub enum WorldToPlayerMessage {
    WorldLight(u8),
}
