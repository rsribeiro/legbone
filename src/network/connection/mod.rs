mod debug;
mod receive;
mod send;

use crate::{
    character::player::Player,
    io::ReadExt,
    persistence,
    world::message::{PlayerToWorldMessage, WorldToPlayerMessage},
    Protocol,
};
use anyhow::{anyhow, Result};
use crossbeam_queue::SegQueue;
use tokio::{
    time::timeout,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    io::{
        AsyncRead,
        AsyncWrite,
        AsyncReadExt,
        AsyncWriteExt
    }
};
use std::{
    convert::TryInto,
    time::Duration,
    net::SocketAddr
};

pub struct Connection<R: AsyncRead + Unpin, W: AsyncWrite + Unpin> {
    read_half: R,
    write_half: W,
    player: Player, //TODO remove
    player_id: u32,
    protocol: Protocol,
    message_queue: SegQueue<Vec<u8>>,
    sender: UnboundedSender<PlayerToWorldMessage>,
    receiver: UnboundedReceiver<WorldToPlayerMessage>,
    peer_addr: SocketAddr,
}

impl<R: AsyncRead + Unpin, W: AsyncWrite + Unpin> Connection<R, W> {
    fn new(
        read_half: R,
        write_half: W,
        protocol: Protocol,
        player: Player,
        sender: UnboundedSender<PlayerToWorldMessage>,
        receiver: UnboundedReceiver<WorldToPlayerMessage>,
        peer_addr: SocketAddr
    ) -> Self {
        let player_id = player.id;
        Self {
            read_half,
            write_half,
            player,
            player_id,
            protocol,
            message_queue: SegQueue::new(),
            sender,
            receiver,
            peer_addr
        }
    }

    pub async fn handle_login(
        mut read_half: R,
        mut write_half: W,
        sender: UnboundedSender<PlayerToWorldMessage>,
        local_addr: SocketAddr,
        peer_addr: SocketAddr
    ) -> Result<Option<Connection<R, W>>> {
        let length = read_half.read_u16_le().await?;
        log::trace!("handle_login: length={length}");

        let (player, protocol) = match length {
            67 => player_login(&mut read_half, &mut write_half).await?,
            221 | 223 | 723 => create_new_player(&mut read_half, &mut write_half).await?,
            _ => account_login(&mut read_half, &mut write_half, length, local_addr).await?,
        };

        if let Some(player) = player {
            let (game_sender, receiver) = unbounded_channel();

            sender.send(PlayerToWorldMessage::LoadPlayer(player.id, game_sender))?;

            log::info!(
                "Player logged in: protocol={:?}, id={}, name={}, ",
                protocol,
                player.id,
                player.name
            );
            let mut client = Connection::new(read_half, write_half, protocol, player, sender, receiver, peer_addr);
            client.queue_login_info().await?;
            client.flush_message_queue().await?;

            Ok(Some(client))
        } else {
            Ok(None)
        }
    }
}

async fn player_login<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(read_half: &mut R, _write_half: &mut W) -> Result<(Option<Player>, Protocol)> {
    //TODO validate message using initial bytes
    //103+ = 00, 00, 01, 01, 00
    //650  = N/A
    read_half.skip(5).await?;

    let protocol: Protocol = read_half.read_u16_le().await?.try_into()?;

    let mut name = String::new();
    read_half.read_string(&mut name, 30).await?;

    let mut password = String::new();
    read_half.read_string(&mut password, 30).await?;

    log::trace!(
        "Journey Onward! Name={name}, password={password}, protocol={protocol:?}"
    );

    Ok((persistence::load_player_by_name(&name), protocol))
}

async fn create_new_player<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(read_half: &mut R, _write_half: &mut W) -> Result<(Option<Player>, Protocol)> {
    //TODO validate message using initial bytes
    //103+ = 00, 00, 00, 01, 00
    //640+ = N/A
    read_half.skip(5).await?;

    let protocol: Protocol = read_half.read_u16_le().await?.try_into()?;

    let mut name = String::new();
    read_half.read_string(&mut name, 30).await?;

    let mut password = String::new();
    read_half.read_string(&mut password, 30).await?;

    let gender = read_half.read_gender(protocol).await?;

    //TODO find out what those bytes mean
    //103+ = 01, 01
    read_half.skip(2).await?;

    let outfit_colors = read_half.read_outfit_colors().await?;

    let mut real_name = String::new();
    read_half.read_string(&mut real_name, 50).await?;

    let mut location = String::new();
    let location_size = if protocol == Protocol::Tibia103 {
        48
    } else {
        50
    };
    read_half.read_string(&mut location, location_size).await?;

    let mut email = String::new();
    read_half.read_string(&mut email, 50).await?;

    let mut comment = String::new();
    if protocol >= Protocol::Tibia400 && protocol <= Protocol::Tibia501 {
        read_half.read_string(&mut comment, 500).await?;
    }

    log::trace!("New Game! Name={name}, password={password}, real name={real_name}, location={location}, e-mail={email}, comment={comment}, protocol={protocol:?}, outfit={outfit_colors:?}, gender={gender:?}");

    let mut player = persistence::create_player(&name);
    player.outfit = outfit_colors;
    player.gender = gender;
    Ok((Some(player), protocol))
}

async fn account_login<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    read_half: &mut R,
    write_half: &mut W,
    message_length: u16,
    local_addr: SocketAddr
) -> Result<(Option<Player>, Protocol)> {
    log::trace!("Account login attempt. length={message_length}");

    //TODO validate message using initial bytes
    //640- = NA
    //650  = 01, 01, 00
    read_half.skip(3).await?;

    let protocol: Protocol = read_half.read_u16_le().await?.try_into()?;

    if protocol >= Protocol::Tibia650 {
        let account_number = read_half.read_u32_le().await?;
        let password_length = read_half.read_u16_le().await?;

        let mut password = String::new();
        read_half.read_string(&mut password, password_length).await?;

        log::trace!("Journey Onward! Account number={account_number}, password={password}, protocol={protocol:?}");

        let msg = send::prepare_character_list(local_addr).await?;
        write_half.write_u16_le(msg.len() as u16).await?;
        write_half.write_all(&msg).await?;
        write_half.flush().await?;

        //Awaits connection be terminated by client, which will connect again using the chosen character
        loop {
            match timeout(Duration::from_secs(1), read_half.read_u8()).await {
                Ok(Ok(_)) => { /* Do nothing */ },
                Ok(Err(err)) if err.kind() == std::io::ErrorKind::UnexpectedEof || err.kind() == std::io::ErrorKind::ConnectionReset => {
                    log::info!("Client disconnected after receiving character list ({:?}).", err.kind());
                    break;
                },
                Ok(Err(err)) if err.kind() == std::io::ErrorKind::TimedOut => { /* Do nothing */ },
                Ok(Err(err)) => {
                    log::error!("Connection ended ({:?})", err.kind());
                    return Err(err.into())
                },
                Err(_elapsed) => { /* Do nothing */ }
            }
        }

        Ok((None, protocol))
    } else {
        log::error!(
            "Unrecognized login message. Protocol={protocol:?}, length={message_length}"
        );
        Err(anyhow!(
            "Unrecognized login message. Protocol={:?}, length={}",
            protocol,
            message_length
        ))
    }
}

impl<R: AsyncRead + Unpin, W: AsyncWrite + Unpin> Drop for Connection<R, W> {
    fn drop(&mut self) {
        log::info!("Connection with {} finished.", self.peer_addr);
    }
}
