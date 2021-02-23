mod send;
mod receive;
mod debug;

use crossbeam_queue::SegQueue;
use std::convert::TryInto;
use async_std::{
    prelude::*,
    net::TcpStream,
};
use crate::{
    io::ReadExt,
    Protocol,
    character::player::Player,
    world::message::{
        PlayerToWorldMessage,
        WorldToPlayerMessage
    },
    persistence
};
use byteorder_async::{
    LE,
    AsyncReadByteOrder,
    AsyncWriteByteOrder
};
use anyhow::{
    Result,
    anyhow
};
use flume::{
    Sender,
    Receiver,
    unbounded
};

pub struct Connection {
    stream: TcpStream,
    player: Player,//TODO remove
    player_id: u32,
    protocol: Protocol,
    message_queue: SegQueue<Vec<u8>>,
    sender: Sender<PlayerToWorldMessage>,
    receiver: Receiver<WorldToPlayerMessage>
}

impl Connection {
    fn new(stream: TcpStream, protocol: Protocol, player: Player, sender: Sender<PlayerToWorldMessage>, receiver: Receiver<WorldToPlayerMessage>) -> Self {
        let player_id = player.id;
        Self {
            stream,
            player,
            player_id,
            protocol,
            message_queue: SegQueue::new(),
            sender,
            receiver
        }
    }
    
    pub async fn handle_login(mut stream: TcpStream, sender: Sender<PlayerToWorldMessage>) -> Result<Option<Connection>> {
        let length = stream.read_u16::<LE>().await?;
        log::trace!("handle_login: length={}", length);

        let (player, protocol) = match length {
            67 => player_login(&mut stream).await?,
            221|223|723 => create_new_player(&mut stream).await?,
            _ => account_login(&mut stream, length).await?
        };
    
        if let Some(player) = player {
            let (game_sender, receiver) = unbounded();

            sender.send_async(PlayerToWorldMessage::LoadPlayer(player.id, game_sender)).await.unwrap();

            log::info!("Player logged in: protocol={:?}, id={}, name={}, ", protocol, player.id, player.name);
            let mut client = Connection::new(stream, protocol, player, sender, receiver);
            if protocol == Protocol::Tibia103 {
                client.queue_login_info_v103().await?;
            } else {
                client.queue_login_info().await?;
            }
            client.flush_message_queue().await?;
        
            Ok(Some(client))
        } else {
            Ok(None)
        }
    }
}

async fn player_login(stream: &mut TcpStream) -> Result<(Option<Player>,Protocol)> {
    //TODO validate message using initial bytes
    //103+ = 00, 00, 01, 01, 00
    //650  = N/A
    stream.skip(5).await?;

    let protocol: Protocol = stream.read_u16::<LE>().await?.try_into()?;

    let mut name = String::new();
    stream.read_string(&mut name, 30).await?;

    let mut password = String::new();
    stream.read_string(&mut password, 30).await?;

    log::trace!("Journey Onward! Name={}, password={}, protocol={:?}", name, password, protocol);

    Ok((persistence::load_player_by_name(&name), protocol))
}

async fn create_new_player(stream: &mut TcpStream) -> Result<(Option<Player>,Protocol)> {
    //TODO validate message using initial bytes
    //103+ = 00, 00, 00, 01, 00
    //640+ = N/A
    stream.skip(5).await?;

    let protocol: Protocol = stream.read_u16::<LE>().await?.try_into()?;
    
    let mut name = String::new();
    stream.read_string(&mut name, 30).await?;

    let mut password = String::new();
    stream.read_string(&mut password, 30).await?;

    let gender = stream.read_gender(protocol).await?;

    //TODO find out what those bytes mean
    //103+ = 01, 01
    stream.skip(2).await?;

    let outfit_colors = stream.read_outfit_colors().await?;

    let mut real_name = String::new();
    stream.read_string(&mut real_name, 50).await?;

    let mut location = String::new();
    let location_size = if protocol == Protocol::Tibia103 {
        48
    } else {
        50
    };
    stream.read_string(&mut location, location_size).await?;

    let mut email = String::new();
    stream.read_string(&mut email, 50).await?;

    let mut comment = String::new();
    if protocol >= Protocol::Tibia400 && protocol <= Protocol::Tibia501 {
        stream.read_string(&mut comment, 500).await?;
    }

    log::trace!("New Game! Name={}, password={}, real name={}, location={}, e-mail={}, comment={}, protocol={:?}, outfit={:?}, gender={:?}", name, password, real_name, location, email, comment, protocol, outfit_colors, gender);
    
    let mut player = persistence::create_player(&name);
    player.outfit = outfit_colors;
    player.gender = gender;
    Ok((Some(player), protocol))
}

async fn account_login(stream: &mut TcpStream, message_length: u16) -> Result<(Option<Player>,Protocol)> {
    log::trace!("Account login attempt. length={}", message_length);

    //TODO validate message using initial bytes
    //640- = NA
    //650  = 01, 01, 00
    stream.skip(3).await?;

    let protocol: Protocol = stream.read_u16::<LE>().await?.try_into()?;

    if protocol >= Protocol::Tibia650 {
        let account_number = stream.read_u32::<LE>().await?;
        let password_length = stream.read_u16::<LE>().await?;

        let mut password = String::new();
        stream.read_string(&mut password, password_length).await?;

        log::trace!("Journey Onward! Account number={}, password={}, protocol={:?}", account_number, password, protocol);

        let msg = send::prepare_character_list().await?;
        stream.write_u16::<LE>(msg.len() as u16).await?;
        stream.write_all(&msg).await?;
        stream.flush().await?;

        //This connection is terminated. Client connects again using the character chosen and the ip/port sent
        Ok((None, protocol))
    } else {
        log::error!("Unrecognized login message. Protocol={:?}, length={}", protocol, message_length);
        return Err(anyhow!("Unrecognized login message. Protocol={:?}, length={}", protocol, message_length))
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        match self.stream.peer_addr() {
            Ok(peer_address) => log::info!("Connection with {} finished.", peer_address),
            Err(_) => log::warn!("Finishing connection")
        }
    }
}
