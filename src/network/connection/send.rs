use super::Connection;
use crate::{
    character::{
        player::{InventorySlot, Player},
        CharacterUpdateType, Direction, HealthStatus, Outfit, OutfitColors, OutfitType,
    },
    chat::{encoding, ChatType},
    constants::*,
    io::WriteExt,
    map::{position::Position, TileObject, MAP},
    network::header::{AuxiliaryHeaderSend, HeaderSend},
    Protocol,
};
use anyhow::{anyhow, Error, Result};
use async_std::{io::Cursor, net::SocketAddr, prelude::*};
use crate::io::byteorder_async::AsyncWriteByteOrder;
use byteorder::LE;

impl Connection {
    async fn send_message(&mut self, message: &[u8]) -> Result<()> {
        self.stream
            .write_u16::<LE>(message.len() as u16 + 2)
            .await?;
        self.stream.write(message).await?;
        self.stream.flush().await?;

        // log::trace!("SENDING: {:02x?} (len={})", message, message.len());
        // log::trace!("SENDING: len={}", message.len());

        Ok(())
    }

    pub async fn flush_message_queue(&mut self) -> Result<()> {
        if !self.message_queue.is_empty() {
            if self.protocol > Protocol::Tibia501 {
                self.send_big_message().await?;
            } else {
                self.send_individual_messages().await?;
            }
        }
        Ok(())
    }

    /// For older clients, sends each message individually. Sends length + actual message
    /// for every queued message.
    async fn send_individual_messages(&mut self) -> Result<()> {
        let mut big_message = Cursor::new(Vec::<u8>::new());
        while let Some(message) = self.message_queue.pop() {
            big_message
                .write_u16::<LE>(message.len() as u16 + 2)
                .await?;
            big_message.write_all(&message).await?;
            log::trace!("SENDING: {:02x?} (len={})", message, message.len());
        }

        let message = big_message.into_inner();
        self.stream.write(&message).await?;
        self.stream.flush().await?;

        // log::trace!("SENDING: {:02x?} (len={})", message, message.len());
        // log::trace!("SENDING: len={}", message.len());

        Ok(())
    }

    /// For newer clients, send all queued messages as one, concatenating every message,
    /// with just one length
    async fn send_big_message(&mut self) -> Result<()> {
        let mut big_message = Cursor::new(Vec::<u8>::new());
        while let Some(message) = self.message_queue.pop() {
            big_message.write_all(&message).await?;
        }

        self.send_message(big_message.into_inner().as_slice()).await
    }

    pub async fn queue_message(&self, message: Vec<u8>) {
        if !message.is_empty() {
            self.message_queue.push(message);
        }
    }

    pub async fn prepare_info(&self, message: &str) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_header(HeaderSend::Info, self.protocol).await?;
        buf.write_null_terminated_string(message).await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_error(&self, message: &str) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_header(HeaderSend::Error, self.protocol).await?;
        buf.write_null_terminated_string(message).await?;

        Ok(buf.into_inner())
    }

    pub async fn send_error(&mut self, err: Error) -> Result<()> {
        log::info!("Sending error {err:?} to client");
        self.send_message(&self.prepare_error(&err.to_string()).await?)
            .await
    }

    async fn prepare_login(&self) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_header(HeaderSend::Login, self.protocol).await?;
        if self.protocol >= Protocol::Tibia300 {
            buf.write_u32::<LE>(self.player.id).await?;
        }

        Ok(buf.into_inner())
    }

    pub async fn queue_login_info(&mut self) -> Result<()> {
        let player_id = self.player.id;
        let position = self.player.position;

        if self.protocol == Protocol::Tibia103 {
            self.queue_message(self.prepare_login().await?).await;

            self.queue_message(
                self.prepare_equipped_item(InventorySlot::Bag, 0x013d, 0)
                    .await?,
            )
            .await;
            self.queue_message(
                self.prepare_equipped_item(InventorySlot::RightHand, 0x015a, 0)
                    .await?,
            )
            .await;
            self.queue_message(
                self.prepare_equipped_item(InventorySlot::LeftHand, 0x025a, 0)
                    .await?,
            )
            .await;

            self.queue_message(self.prepare_map(self.player.position, 18, 14, 1).await?)
                .await;
            self.queue_message(self.prepare_status_message("Hello, World!").await?)
                .await;
            // self.queue_message(self.prepare_message_of_the_day(0x0101, "Hello, World!").await?).await;
        } else {
            self.queue_message(self.prepare_login().await?).await;
            self.queue_message(self.prepare_stats().await?).await;
            if self.protocol >= Protocol::Tibia400 {
                self.queue_message(self.prepare_skills().await?).await;
            }
            self.queue_message(
                self.prepare_equipped_item(InventorySlot::Helmet, 0x005c, 0)
                    .await?,
            )
            .await;
            self.queue_message(
                self.prepare_equipped_item(InventorySlot::Necklace, 0x007b, 0)
                    .await?,
            )
            .await;
            self.queue_message(
                self.prepare_equipped_item(InventorySlot::Bag, 0x013d, 0)
                    .await?,
            )
            .await;
            self.queue_message(
                self.prepare_equipped_item(InventorySlot::Armor, 0x007a, 0)
                    .await?,
            )
            .await;
            self.queue_message(
                self.prepare_equipped_item(InventorySlot::LeftHand, 0x085d, 0)
                    .await?,
            )
            .await;
            self.queue_message(
                self.prepare_equipped_item(InventorySlot::RightHand, 0x065a, 0)
                    .await?,
            )
            .await;
            self.queue_message(
                self.prepare_equipped_item(InventorySlot::Legs, 0x0079, 0)
                    .await?,
            )
            .await;
            self.queue_message(
                self.prepare_equipped_item(InventorySlot::Boots, 0x0378, 0)
                    .await?,
            )
            .await;

            self.queue_message(self.prepare_map(self.player.position, 18, 14, 3).await?)
                .await;
            self.queue_message(
                self.prepare_update_character(player_id, CharacterUpdateType::LightLevel, 0)
                    .await?,
            )
            .await;
            self.queue_message(
                self.prepare_magic_effect(MagicEffect::Teleport, position)
                    .await?,
            )
            .await;
            self.queue_message(self.prepare_world_light(6).await?).await;
            self.queue_message(self.prepare_status_message("Hello, World!").await?)
                .await;
            // self.queue_message(self.prepare_message_of_the_day(0x0101, "Hello, World!").await?).await;
        }

        Ok(())
    }

    pub async fn prepare_world_light(&self, light_level: u8) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_header(HeaderSend::WorldLight, self.protocol)
            .await?;
        buf.write_u8(light_level).await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_update_outfit(
        &self,
        id: u32,
        outfit: OutfitType,
        outfit_colors: OutfitColors,
    ) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        if self.protocol == Protocol::Tibia103 {
            // buf.write_header(HeaderSend::UpdateObject, self.protocol).await?;
            // buf.write_position(self.player.position, self.protocol).await?;
            // buf.write_u8(ObjectUpdateType::Update as u8).await?;
        } else {
            buf.write_header(HeaderSend::UpdateCharacter, self.protocol)
                .await?;
            buf.write_u32::<LE>(id).await?;
            buf.write_u8(CharacterUpdateType::Outfit as u8).await?;
            buf.write_u8(outfit as u8).await?;
            buf.write_outfit_colors(outfit_colors).await?;
        }

        Ok(buf.into_inner())
    }

    pub async fn prepare_update_character(
        &self,
        id: u32,
        update_type: CharacterUpdateType,
        value: u8,
    ) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_header(HeaderSend::UpdateCharacter, self.protocol)
            .await?;
        buf.write_u32::<LE>(id).await?;
        buf.write_u8(update_type as u8).await?;
        buf.write_u8(value).await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_update_object(
        &self,
        position: Position,
        update_type: ObjectUpdateType,
        stack_pos: u8,
    ) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        if self.protocol == Protocol::Tibia103 {
            // buf.write_header(HeaderSend::UpdateObject, self.protocol).await?;
            // buf.write_position(position, self.protocol).await?;
            // buf.write_u8(update_type.to_protocol_103_type() as u8).await?;
            // buf.write_u8(stack_pos).await?;
        } else {
            buf.write_header(HeaderSend::UpdateObject, self.protocol)
                .await?;
            buf.write_position(position, self.protocol).await?;
            buf.write_u8(update_type as u8).await?;
            buf.write_u8(stack_pos).await?;

            //remove light?
            if update_type == ObjectUpdateType::Remove {
                buf.write_zeroes(6).await?;
            }
        }

        Ok(buf.into_inner())
    }

    pub async fn prepare_magic_effect(
        &self,
        effect: MagicEffect,
        position: Position,
    ) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_header(HeaderSend::MagicEffect, self.protocol)
            .await?;
        buf.write_position(position, self.protocol).await?;
        buf.write_u8(effect as u8).await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_skills(&self) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        let skills = self.player.skills;

        buf.write_header(HeaderSend::Skills, self.protocol).await?;
        if self.protocol == Protocol::Tibia400 {
            buf.write_u8(skills.sword).await?;
            buf.write_u8(skills.club).await?;
            buf.write_u8(skills.gauche).await?;
            buf.write_u8(skills.fist).await?;
            buf.write_u8(skills.missile).await?;
            buf.write_u8(skills.shield).await?;
            buf.write_u8(skills.distance).await?; //throwing on v4
            buf.write_u8(skills.fishing).await?;
        } else {
            buf.write_u8(skills.sword).await?;
            buf.write_u8(skills.club).await?;
            buf.write_u8(skills.axe).await?;
            buf.write_u8(skills.distance).await?;
            buf.write_u8(skills.shield).await?;
            buf.write_u8(skills.fist).await?;
            buf.write_u8(skills.fishing).await?;
        }

        Ok(buf.into_inner())
    }

    async fn prepare_map(
        &self,
        position: Position,
        width: u16,
        height: u16,
        layers: u8,
    ) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_header(HeaderSend::Map, self.protocol).await?;
        buf.write_position(self.player.position, self.protocol)
            .await?;
        buf.write_all(
            self.prepare_map_internal(position, width, height, layers)
                .await?
                .as_slice(),
        )
        .await?;

        Ok(buf.into_inner())
    }

    async fn prepare_map_internal(
        &self,
        position: Position,
        width: u16,
        height: u16,
        layers: u8,
    ) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        let corner = position - (((width as i16 - 1) / 2), ((height as i16 - 1) / 2), 0);
        let corner_2 = corner + (width as i16 - 1, height as i16 - 1, layers as i8 - 1);

        log::trace!(
            "center = {position:?}, corner_1 = {corner:?}, corner_2 = {corner_2:?}"
        );
        log::trace!(
            "width = {width:?}, height={height:?}, layers={layers:?}"
        );

        for z in 0..layers {
            let position = Position::new(position.x, position.y, z);
            buf.write_all(&self.prepare_layer(position, corner, width, height).await?)
                .await?;
        }
        buf.set_position(buf.position() - 1);
        buf.write_u8(0xfe).await?;
        buf.write_u8(0x00).await?;

        // log::trace!("MAP = {:02x?}", buf);

        Ok(buf.into_inner())
    }

    async fn prepare_layer(
        &self,
        position: Position,
        corner: Position,
        width: u16,
        height: u16,
    ) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        for x in 0..width {
            for y in 0..height {
                let position = corner + (x as i16, y as i16, position.z as i8);
                buf.write_all(&self.prepare_tile(position).await?).await?;
            }
        }

        Ok(buf.into_inner())
    }

    async fn prepare_tile(&self, position: Position) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());
        if let Some(tile) = MAP.get().unwrap().get_tile_objects(position) {
            for tile_object in tile {
                match tile_object {
                    TileObject::Other(tile_id) => buf.write_u16::<LE>(*tile_id).await?,
                    TileObject::FluidContainer(tile_id, fluid) => {
                        buf.write_u16::<LE>(*tile_id).await?;
                        if self.protocol >= Protocol::Tibia300 {
                            buf.write_u8(*fluid as u8).await?;
                        }
                    }
                    TileObject::LightSource(tile_id, light_level) => {
                        buf.write_u16::<LE>(*tile_id).await?;
                        if self.protocol >= Protocol::Tibia300 {
                            buf.write_u8(*light_level).await?;
                        }
                    }
                    TileObject::Stackable(tile_id, count) => {
                        buf.write_u16::<LE>(*tile_id).await?;
                        if self.protocol >= Protocol::Tibia300 {
                            buf.write_u8(*count).await?;
                        }
                    }
                    TileObject::Creature(id, name, outfit) => {
                        if self.protocol >= Protocol::Tibia300 {
                            buf.write_all(&self.prepare_character(*id, name, *outfit).await?)
                                .await?;
                        }
                    }
                }
            }
        }

        if position == self.player.position {
            buf.write_all(&self.prepare_player_character().await?)
                .await?;
        }

        if self.protocol == Protocol::Tibia103 {
            buf.write_u8(0xff).await?;
        }

        buf.write_u8(0xff).await?;
        Ok(buf.into_inner())
    }

    async fn prepare_player_character(&self) -> Result<Vec<u8>> {
        if self.protocol == Protocol::Tibia103 {
            let mut buf = Cursor::new(Vec::<u8>::new());
            buf.write_u8(AuxiliaryHeaderSend::Character as u8).await?;
            buf.write_outfit_colors(self.player.outfit).await?;
            Ok(buf.into_inner())
        } else {
            self.prepare_character(
                self.player.id,
                &self.player.name,
                Outfit::human(self.player.outfit),
            )
            .await
        }
    }

    async fn prepare_character(&self, id: u32, name: &str, outfit: Outfit) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_u8(AuxiliaryHeaderSend::Character as u8).await?;
        buf.write_u32::<LE>(0).await?; //knows creature
        buf.write_u32::<LE>(id).await?;
        buf.write_string_with_fixed_length(name, 30).await?;
        buf.write_u8(HealthStatus::Healthy as u8).await?;
        buf.write_u8(Direction::South as u8).await?;

        buf.write_u8(outfit.outfit_type as u8).await?;
        buf.write_outfit_colors(outfit.colors).await?;

        //light level=0
        buf.write_u8(0).await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_status_message(&self, status: &str) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_header(HeaderSend::StatusMessage, self.protocol)
            .await?;
        buf.write_null_terminated_string(status).await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_message_of_the_day(
        &self,
        message_number: u16,
        message: &str,
    ) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_header(HeaderSend::MessageOfTheDay, self.protocol)
            .await?;
        if self.protocol > Protocol::Tibia400 {
            buf.write_u16::<LE>(message_number).await?;
            buf.write_u8(0x0a).await?;
        }

        buf.write_null_terminated_string(message).await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_equipped_item(
        &self,
        slot: InventorySlot,
        item: u16,
        stack: u8,
    ) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_header(HeaderSend::EquippedItem, self.protocol)
            .await?;
        if self.protocol == Protocol::Tibia103 {
            buf.write_u16::<LE>(item).await?;
            buf.write_u8(slot as u8).await?;
        } else {
            buf.write_u8(slot as u8).await?;
            buf.write_u16::<LE>(item).await?;
            buf.write_u8(stack).await?;
        }

        Ok(buf.into_inner())
    }

    pub async fn prepare_stats(&self) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        let stats = self.player.stats;
        buf.write_header(HeaderSend::Stats, self.protocol).await?;

        buf.write_u16::<LE>(stats.health_points).await?;
        buf.write_u16::<LE>(stats.capacity).await?;
        if self.protocol >= Protocol::Tibia400 {
            buf.write_u32::<LE>(stats.experience_points).await?;
            buf.write_u8(stats.experience_level).await?;
            buf.write_u16::<LE>(stats.mana_points).await?;
            buf.write_u8(stats.magic_level).await?;
            buf.write_u16::<LE>(stats.ammunition).await?;
        } else if self.protocol >= Protocol::Tibia300 {
            buf.write_u8(stats.intelligence).await?;
            buf.write_u8(stats.strength).await?;
            buf.write_u8(stats.dexterity).await?;
            buf.write_u16::<LE>(stats.experience_points as u16).await?;
            buf.write_u8(stats.experience_level).await?;
        }

        Ok(buf.into_inner())
    }

    pub async fn prepare_chat(
        &self,
        chat_type: ChatType,
        msg: &str,
        sender: Option<&Player>,
        position: Option<Position>,
    ) -> Result<Vec<u8>> {
        let msg = match chat_type {
            ChatType::Yell => encoding::translate_upper(&msg.to_uppercase()),
            _ => encoding::translate(msg),
        };
        self.prepare_raw_chat(chat_type, &msg, sender, position)
            .await
    }

    async fn prepare_raw_chat(
        &self,
        chat_type: ChatType,
        msg: &[u8],
        sender: Option<&Player>,
        position: Option<Position>,
    ) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        let position = match self.protocol {
            Protocol::Tibia300 => position.map(|p| p + (1, 1, 0)),
            _ => position,
        };

        buf.write_header(HeaderSend::Chat, self.protocol).await?;
        buf.write_position(position.unwrap_or(Position::new(0, 0, 0)), self.protocol)
            .await?;
        buf.write_u8(chat_type as u8).await?;
        if let Some(player) = sender {
            buf.write_all(player.name.as_bytes()).await?;
            buf.write_u8(0x09).await?; //TAB
        }
        buf.write_all(msg).await?;
        buf.write_u8(0x00).await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_user_info(&self, player_name: &str) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_header(HeaderSend::UserInfo, self.protocol)
            .await?;
        buf.write_u16::<LE>(0x1010).await?; //# of bytes to allocate for text
        let info = &format!("INFO: name={player_name}");
        buf.write_null_terminated_string(info).await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_user_list(&self) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_header(HeaderSend::UserList, self.protocol)
            .await?;
        buf.write_u16::<LE>(0x1010).await?; //# of bytes to allocate for text

        //for each player online
        {
            buf.write_all(self.player.name.as_bytes()).await?;
            buf.write_u8(b'\n').await?;
        }

        buf.write_u8(0).await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_data_window(&self) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_header(HeaderSend::DataWindow, self.protocol)
            .await?;
        buf.write_string_with_fixed_length(&self.player.name, 30)
            .await?;

        if self.protocol <= Protocol::Tibia501 {
            buf.write_string_with_fixed_length("password", 30).await?;
            buf.write_gender(self.player.gender, self.protocol).await?;
            buf.write_outfit_colors(self.player.outfit).await?;
            buf.write_string_with_fixed_length("realname", 50).await?;
            buf.write_string_with_fixed_length("location", 50).await?;
            buf.write_string_with_fixed_length("email", 50).await?;
            if self.protocol >= Protocol::Tibia400 {
                buf.write_string_with_fixed_length("comment", 500).await?;
            }
        } else {
            buf.write_gender(self.player.gender, self.protocol).await?;
            buf.write_outfit_colors(self.player.outfit).await?;
        }

        Ok(buf.into_inner())
    }

    pub async fn prepare_open_container(&self) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_header(HeaderSend::OpenContainer, self.protocol)
            .await?;
        buf.write_u8(1).await?; //local_id
        buf.write_u16::<LE>(0x013d).await?; //item_id

        //Add each item inside container
        //TODO send non hardcoded items
        for _ in 0..5 {
            buf.write_u16::<LE>(0x005a).await?; //item_id
        }

        buf.write_u16::<LE>(0xffff).await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_close_container(&self, local_id: u8) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        buf.write_header(HeaderSend::CloseContainer, self.protocol)
            .await?;
        buf.write_u8(local_id).await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_change_direction(&self, id: u32, direction: Direction) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        if self.protocol == Protocol::Tibia103 {
        } else {
            buf.write_u8(AuxiliaryHeaderSend::ChangeDirection as u8)
                .await?;
            buf.write_u8(direction as u8).await?;
            buf.write_u32::<LE>(id).await?;
        }

        Ok(buf.into_inner())
    }

    pub async fn prepare_move_character(
        &self,
        direction: Direction,
        from: Position,
        to: Position,
    ) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        log::trace!(
            "move character from {from:?} to {to:?}, direction={direction:?}"
        );

        let (width, height) = match direction {
            Direction::North | Direction::South => (18, 1),
            Direction::East | Direction::West => (1, 14),
        };
        let center = to
            + match direction {
                Direction::North => (0, -6, 0),
                Direction::East => (9, 0, 0),
                Direction::South => (0, 7, 0),
                Direction::West => (-8, 0, 0),
            };
        let layers = if self.protocol == Protocol::Tibia103 {
            1
        } else {
            3
        };

        log::trace!("center = {center:?}");

        buf.write_header(direction.into(), self.protocol).await?;
        buf.write_all(
            self.prepare_map_internal(center, width, height, layers)
                .await?
                .as_slice(),
        )
        .await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_green_chat(
        &self,
        _chat_type: ChatType,
        msg: &str,
        _sender: Option<&Player>,
        _position: Option<Position>,
    ) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());

        // let position = match self.protocol {
        //     Protocol::Tibia300 => position.map(|p| p + (1,1,0)),
        //     _ => position
        // };

        buf.write_header(HeaderSend::GreenChat, self.protocol)
            .await?;
        // buf.write_position(position.unwrap_or(Position::new(0, 0, 0)), self.protocol).await?;
        // buf.write_u8(chat_type as u8).await?;
        // if let Some(player) = sender {
        //     buf.write_all(player.name.as_bytes()).await?;
        //     buf.write_u8(0x09).await?;//TAB
        // }
        buf.write_null_terminated_string(msg).await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_unknown_0x0000(&self) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());
        buf.write_header(HeaderSend::Unknown0x0000, self.protocol)
            .await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_unknown_0x000f(&self) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());
        buf.write_header(HeaderSend::Unknown0x000f, self.protocol)
            .await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_unknown_0x0033(&self) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());
        buf.write_header(HeaderSend::Unknown0x0033, self.protocol)
            .await?;

        Ok(buf.into_inner())
    }

    pub async fn prepare_unknown_0x0034(&self) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::<u8>::new());
        buf.write_header(HeaderSend::Unknown0x0034, self.protocol)
            .await?;

        Ok(buf.into_inner())
    }
}

pub async fn prepare_character_list(server_address: SocketAddr) -> Result<Vec<u8>> {
    match server_address {
        SocketAddr::V4(server_address) => {
            log::trace!("Local Address = {server_address:?}");

            let ip = server_address.ip();
            let port = server_address.port();
            let chararacter_count = 1;
            let character_name = "Player";
            let world = "legbone";

            let mut buf = Cursor::new(Vec::<u8>::new());
            buf.write_u8(0x64).await?;
            buf.write_u8(chararacter_count).await?;
            for _ in 0..chararacter_count {
                buf.write_length_and_string(character_name).await?;
                buf.write_length_and_string(world).await?;
                for &octet in ip.octets().iter() {
                    buf.write_u8(octet).await?;
                }
                buf.write_u16::<LE>(port).await?;
            }

            Ok(buf.into_inner())
        }
        SocketAddr::V6(_) => Err(anyhow!("Game does not support ipv6")),
    }
}
