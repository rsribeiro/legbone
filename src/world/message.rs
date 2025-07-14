use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Debug)]
pub enum PlayerToWorldMessage {
    LoadPlayer(u32, UnboundedSender<WorldToPlayerMessage>),
    UnloadPlayer(u32),
    Walk(u32),
}

#[derive(Clone, Copy, Debug)]
pub enum WorldToPlayerMessage {
    WorldLight(u8),
}
