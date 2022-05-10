use std::{
    time::Duration,
    convert::TryInto
};
use async_std::{
    prelude::*,
    io::{
        self,
        Cursor
    },
};
use super::Connection;
use crate::{
    network::header::HeaderReceive,
    chat::ChatType,
    character::{
        Direction,
        FightMode,
        FightStance,
        CharacterOutfit
    },
    map::position::PositionQualifier,
    Protocol,
    constants::{
        MagicEffect,
        ObjectUpdateType
    },
    io::ReadExt,
    world::message::{
        PlayerToWorldMessage,
        WorldToPlayerMessage
    },
    Opts,
};
use byteorder_async::{
    LE,
    AsyncReadByteOrder
};
use anyhow::Result;
use clap::Parser;

impl Connection {
    pub async fn handle_connection(&mut self) -> Result<()> {
        loop {
            //TODO use non blocking io instead of timeout
            match io::timeout(Duration::from_millis(100), self.stream.read_u16::<LE>()).await {
                Ok(length) => {
                    let mut message = vec![0_u8; length as usize];
                    self.stream.read_exact(&mut message).await?;
                    log::trace!("Message received: length={}, bytes={:02x?}", length, message);
        
                    let mut message = Cursor::new(message);
        
                    match message.read_u16::<LE>().await?.try_into() {
                        Ok(header) => {
                            log::trace!("Message received from client: {:?}", header);
                            match header {
                                HeaderReceive::PlayerInfo => self.receive_player_info(&mut message).await?,
                                HeaderReceive::UserList => self.receive_user_list(&mut message).await?,
                                HeaderReceive::Walk => self.receive_walk(&mut message).await?,
                                HeaderReceive::AutoWalk => self.receive_auto_walk(&mut message).await?,
                                HeaderReceive::LookAt => self.receive_look_at(&mut message).await?,
                                HeaderReceive::Chat => self.receive_chat(&mut message).await?,
                                HeaderReceive::ChangeDirection => self.receive_change_direction(&mut message).await?,
                                HeaderReceive::Comment => self.receive_comment(&mut message).await?,
                                HeaderReceive::Push => self.receive_push(&mut message).await?,
                                HeaderReceive::UseItem => self.receive_use_item(&mut message).await?,
                                HeaderReceive::CloseContainer => self.receive_close_container(&mut message).await?,
                                HeaderReceive::RequestChangeData => self.receive_change_data(&mut message).await?,
                                HeaderReceive::SetData => self.receive_set_data(&mut message).await?,
                                HeaderReceive::SetText => self.receive_set_text(&mut message).await?,
                                HeaderReceive::HouseText => self.receive_house_text(&mut message).await?,
                                HeaderReceive::ChangeMode => self.receive_change_mode(&mut message).await?,
                                HeaderReceive::ExitBattle => self.receive_exit_battle(&mut message).await?,
                                HeaderReceive::SetTarget => self.receive_set_target(&mut message).await?,
                                HeaderReceive::Echo => {}
                                HeaderReceive::Logout => {
                                    self.sender.send_async(PlayerToWorldMessage::UnloadPlayer(self.player_id)).await?;
                                    break
                                }
                            }
                        },
                        Err(err) => {
                            log::error!("Error reading header: {:?}", err);
                        }
                    }
                },
                Err(err) => {
                    match err.kind() {
                        std::io::ErrorKind::UnexpectedEof => {
                            log::info!("Client disconnected.");
                            break;
                        },
                        std::io::ErrorKind::TimedOut => { /* do nothing */ }
                        _ => return Err(err.into())
                    }
                }
            };

            if self.protocol >= Protocol::Tibia300 {
                if let Ok(msg) = self.receiver.try_recv() {
                    match msg {
                        WorldToPlayerMessage::WorldLight(light_level) => self.queue_message(self.prepare_world_light(light_level).await?).await
                    }
                }
            }

            self.flush_message_queue().await?;
        }
        Ok(())
    }

    async fn receive_set_text(&mut self, _message: &mut Cursor<Vec<u8>>) -> Result<()> {
        log::trace!("received set_text");
        Ok(())
    }

    async fn receive_house_text(&mut self, _message: &mut Cursor<Vec<u8>>) -> Result<()> {
        log::trace!("received house_text");
        Ok(())
    }

    async fn receive_change_mode(&mut self, message: &mut Cursor<Vec<u8>>) -> Result<()> {
        let fight_mode: FightMode = message.read_u8().await?.try_into()?;
        let fight_stance: FightStance = message.read_u8().await?.try_into()?;

        log::trace!("Change mode: mode={:?}, stance={:?}", fight_mode, fight_stance);

        Ok(())
    }

    async fn receive_exit_battle(&mut self, _message: &mut Cursor<Vec<u8>>) -> Result<()> {
        log::trace!("Exit battle");
        Ok(())
    }

    async fn receive_player_info(&mut self, message: &mut Cursor<Vec<u8>>) -> Result<()> {
        let mut player_name = String::new();
        unsafe { message.read_string_until_end(&mut player_name).await? };
        self.queue_message(self.prepare_user_info(&player_name).await?).await;

        Ok(())
    }

    async fn receive_user_list(&mut self, _message: &mut Cursor<Vec<u8>>) -> Result<()> {
        self.queue_message(self.prepare_user_list().await?).await;
        Ok(())
    }

    async fn receive_push(&mut self, message: &mut Cursor<Vec<u8>>) -> Result<()> {
        let (position_from, object_id, stack_pos, position_to, count) = if self.protocol == Protocol::Tibia103 {
            let position_from = message.read_position(self.protocol).await?;
            let object_id = message.read_u16::<LE>().await?;
            let stack_pos = message.read_u8().await?;
            let position_to = message.read_position(self.protocol).await?;            

            (position_from, object_id, stack_pos, position_to, None)
        } else {
            let position_from = message.read_position(self.protocol).await?;
            let object_id = message.read_u16::<LE>().await?;
            let stack_pos = message.read_u8().await?;
            let position_to = message.read_position(self.protocol).await?;
            let count = message.read_u8().await?;

            (position_from, object_id, stack_pos, position_to, Some(count))
        };

        let msg_from = match position_from.get_qualifier(self.protocol)? {
            PositionQualifier::None => format!("{}", position_from),
            PositionQualifier::Container(container_index, item_index) => {                
                format!("(container={}, index={})", item_index, container_index)
            },
            PositionQualifier::Inventory(inventory_slot) => {
                format!("{:?}", inventory_slot)
            }
        };

        let msg_to = match position_to.get_qualifier(self.protocol)? {
            PositionQualifier::None => format!("{}", position_to),
            PositionQualifier::Container(container_index, item_index) => {                
                format!("(container={}, index={})", item_index, container_index)
            },
            PositionQualifier::Inventory(inventory_slot) => {
                format!("{:?}", inventory_slot)
            }
        };

        log::trace!("PUSH object=0x{:04x?}, from={}->to={}, stack_pos={:?}, count={:?}", object_id, msg_from, msg_to, stack_pos, count);
        
        Ok(())
    }

    async fn receive_set_data(&mut self, message: &mut Cursor<Vec<u8>>) -> Result<()> {
        let outfit = if self.protocol <= Protocol::Tibia501 {
            let mut password = String::new();
            message.read_string(&mut password, 30).await?;

            let outfit = message.read_outfit_colors().await?;

            let mut real_name = String::new();
            message.read_string(&mut real_name, 50).await?;

            let mut location = String::new();
            message.read_string(&mut location, 50).await?;

            let mut email = String::new();
            message.read_string(&mut email, 50).await?;

            if self.protocol >= Protocol::Tibia400 {
                let mut comment = String::new();
                message.read_string(&mut comment, 500).await?;

                log::trace!("Change Data: password={}, outfit={:?}, real name={}, location={}, e-mail={}, comment={}", password, outfit, real_name, location, email, comment);
            } else {
                log::trace!("Change Data: password={}, outfit={:?}, real name={}, location={}, e-mail={}", password, outfit, real_name, location, email);
            }

            outfit
        } else {
            let outfit = message.read_outfit_colors().await?;

            log::trace!("Change Data: outfit={:?}", outfit);

            outfit
        };
        
        self.queue_message(self.prepare_update_outfit(self.player.id, CharacterOutfit::Human, outfit).await?).await;
        self.player.outfit = outfit;

        Ok(())
    }

    async fn receive_change_data(&mut self, _message: &mut Cursor<Vec<u8>>) -> Result<()> {
        self.queue_message(self.prepare_data_window().await?).await;
        Ok(())
    }

    async fn receive_set_target(&mut self, message: &mut Cursor<Vec<u8>>) -> Result<()> {
        let target_id = message.read_u32::<LE>().await?;
        log::trace!("Set target, id={}", target_id);
        
        Ok(())
    }

    async fn receive_use_item(&mut self, message: &mut Cursor<Vec<u8>>) -> Result<()> {
        let item_type = message.read_u8().await?; //1 = regular, 2 = usable with
        let pos = message.read_position(self.protocol).await?;
        let item_id = message.read_u16::<LE>().await?;
        let stack_pos = message.read_u8().await?;
        let unknown = message.read_u8().await?;

        log::trace!("item_type={}, pos={}, item_id=0x{:04x?}, stack_pos={}, unknown={}", item_type, pos, item_id, stack_pos, unknown);
        
        let message = self.prepare_open_container().await?;
        self.queue_message(message).await;

        Ok(())
    }

    async fn receive_close_container(&mut self, message: &mut Cursor<Vec<u8>>) -> Result<()> {
        let local_id = message.read_u8().await?;

        self.queue_message(self.prepare_close_container(local_id).await?).await;

        Ok(())
    }

    async fn receive_look_at(&mut self, message: &mut Cursor<Vec<u8>>) -> Result<()> {
        let position = message.read_position(self.protocol).await?;

        let msg = match position.get_qualifier(self.protocol)? {
            PositionQualifier::None => format!("Looking at position {}", position),
            PositionQualifier::Container(container_index, item_index) => {                
                format!("Looking at index {} inside container {}.", item_index, container_index)
            },
            PositionQualifier::Inventory(inventory_slot) => {
                format!("Looking at {:?}", inventory_slot)
            }
        };

        log::trace!("{}", msg);
        self.queue_message(self.prepare_chat(ChatType::GreenScreenOnly, &msg, None, Some(self.player.position)).await?).await;

        Ok(())
    }

    async fn receive_change_direction(&mut self, message: &mut Cursor<Vec<u8>>) -> Result<()> {
        let direction: Direction = message.read_u8().await?.try_into()?;
        log::trace!("Change direction to {:?}", direction);

        //todo use real stack pos
        let mut msg = self.prepare_update_object(self.player.position, ObjectUpdateType::Update, 1).await?;
        msg.extend(self.prepare_change_direction(self.player.id, direction).await?);
        self.queue_message(msg).await;
        
        Ok(())
    }

    async fn receive_auto_walk(&mut self, message: &mut Cursor<Vec<u8>>) -> Result<()> {
        let position = message.read_position(self.protocol).await?;
        log::trace!("Auto walk to {:?}", position);
       
        Ok(())
    }

    async fn receive_walk(&mut self, message: &mut Cursor<Vec<u8>>) -> Result<()> {
        let direction: Direction = message.read_u8().await?.try_into()?;
        log::trace!("Walk 1 tile {:?}", direction);

        self.sender.send_async(PlayerToWorldMessage::Walk(self.player.id)).await?;

        let old_position = self.player.position;
        let new_position = self.player.position + direction;
        self.player.position = new_position;

        //Remove character from old tile
        let msg = self.prepare_update_object(old_position, ObjectUpdateType::Remove, 1).await?;
        self.queue_message(msg).await;

        //Add character to new tile
        let mut msg = self.prepare_update_object(new_position, ObjectUpdateType::Add, 1).await?;
        msg.extend(self.prepare_change_direction(self.player.id, direction).await?);
        self.queue_message(msg).await;

        //Move character and update map
        self.queue_message(self.prepare_move_character(direction, old_position, new_position).await?).await;

        Ok(())
    }

    async fn receive_chat(&mut self, message: &mut Cursor<Vec<u8>>) -> Result<()> {
        let length = message.read_u16::<LE>().await?;
        let opts: Opts = Opts::parse();

        let mut raw_msg = vec![0_u8; length as usize];
        message.read_exact(&mut raw_msg).await?;
        log::trace!("raw message = {:02x?}", raw_msg);
        let msg = unsafe { String::from_utf8_unchecked(raw_msg) };
        log::trace!("message = '{}'", msg);

        if !opts.nodebug && msg.starts_with("\\d ") {
            self.receive_debug_command(&msg[2..]).await?;
        } else if msg.starts_with('#') {
            self.receive_qualified_chat(&msg).await?;
        } else {
            self.queue_message(self.prepare_chat(ChatType::Normal, &msg, Some(&self.player), Some(self.player.position)).await?).await;
        }

        Ok(())
    }

    async fn receive_debug_command(&mut self, msg: &str) -> Result<()> {
        if let Err(err) = self.send_debug_command(msg).await {
            self.queue_message(self.prepare_magic_effect(MagicEffect::Puff, self.player.position).await?).await;
            log::trace!("Error on debug command: {:?}", err);
        }
        
        Ok(())
    }

    async fn receive_qualified_chat(&mut self, msg: &str) -> Result<()> {
        match TryInto::<ChatType>::try_into(msg.chars().skip(1).next()) {
            Ok(chat_type) => {
                self.queue_message(self.prepare_chat(chat_type, &msg[3..], Some(&self.player), Some(self.player.position)).await?).await;
            },
            Err(_err) => {
                self.queue_message(self.prepare_magic_effect(MagicEffect::Puff, self.player.position).await?).await;
            }
        }
        Ok(())
    }

    async fn receive_comment(&self, message: &mut Cursor<Vec<u8>>) -> Result<()> {
        let mut msg = String::new();
        unsafe { message.read_string_until_end(&mut msg).await? };

        log::info!("Received comment from client: {}", msg);

        Ok(())
    }
}
