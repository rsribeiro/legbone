use super::Connection;
use crate::{
    character::{player::InventorySlot, CharacterUpdateType, OutfitColors},
    chat::ChatType,
    constants::MagicEffect,
    io::WriteExt,
    network::header::HeaderSend,
    Protocol,
};
use anyhow::{
    Result,
    anyhow
};
use std::{
    io::Cursor,
    convert::TryInto,
    sync::atomic::{AtomicU16, AtomicU8, Ordering},
};

impl Connection {
    pub async fn send_debug_command(&mut self, command: &str) -> Result<()> {
        let mut args = command.split_ascii_whitespace();
        if let Some(command) = args.next() {
            let args: Vec<&str> = args.collect();
            self.debug_command(command, args).await?;
        }
        Ok(())
    }

    #[allow(clippy::unit_arg)]
    async fn debug_command(&mut self, command: &str, args: Vec<&str>) -> Result<()> {
        log::debug!("Received debug command {command:?}");
        match command {
            "chars" => self.command_chars().await,
            "char" => self.command_char(args[0]).await,
            "echo" => self.command_echo().await,
            "item" => self.command_item(args[0], args[1]).await,
            "i" => self.command_item_right_hand(args[0]).await,
            "stats" => Ok(self.queue_message(self.prepare_stats().await?).await),
            "skills" => Ok(self.queue_message(self.prepare_skills().await?).await),
            "me" => self.command_magic_effect(args[0]).await,
            "wlight" => Ok(self
                .queue_message(self.prepare_world_light(args[0].parse::<u8>()?).await?)
                .await),
            "plight" => Ok(self
                .queue_message(
                    self.prepare_update_character(
                        self.player.id,
                        CharacterUpdateType::LightLevel,
                        args[0].parse::<u8>()?,
                    )
                    .await?,
                )
                .await),
            "userlist" => Ok(self.queue_message(self.prepare_user_list().await?).await),
            "userinfo" => Ok(self
                .queue_message(self.prepare_user_info(args[0]).await?)
                .await),
            "info" => Ok(self
                .queue_message(self.prepare_info(&args.join(" ")).await?)
                .await),
            "error" => self.send_error(anyhow!(args.join(" "))).await,
            "error2" => Ok(self
                .queue_message(self.prepare_error(&args.join(" ")).await?)
                .await),
            "motd" => self.command_motd(args).await,
            "status" => Ok(self
                .queue_message(self.prepare_status_message(&args.join(" ")).await?)
                .await),
            "panic" => panic!("{}", args.join(" ")),
            "chat" => self.command_chat().await,
            "outfit" => self.command_outfit(args[0]).await,
            "cd" => self.command_change_direction(args[0]).await,
            "gc" => self.command_green_chat(args).await,
            "u0" => Ok(self
                .queue_message(self.prepare_unknown_0x0000().await?)
                .await),
            "uf" => Ok(self
                .queue_message(self.prepare_unknown_0x000f().await?)
                .await),
            "u33" => Ok(self
                .queue_message(self.prepare_unknown_0x0033().await?)
                .await),
            "u34" => Ok(self
                .queue_message(self.prepare_unknown_0x0034().await?)
                .await),
            _ => Ok(self
                .queue_message(
                    self.prepare_magic_effect(MagicEffect::Puff, self.player.position)
                        .await?,
                )
                .await),
        }
    }

    async fn command_chars(&self) -> Result<()> {
        let chars: Vec<u8> = (0x20_u8..=0x7f).collect();
        let chat_msg = unsafe { std::str::from_utf8_unchecked(&chars) };

        let message = if self.protocol < Protocol::Tibia501 {
            self.prepare_chat(
                ChatType::Normal,
                chat_msg,
                Some(&self.player),
                Some(self.player.position),
            )
            .await?
        } else {
            self.prepare_chat(
                ChatType::RedConsoleWhiteScreen,
                chat_msg,
                None,
                Some(self.player.position),
            )
            .await?
        };

        self.queue_message(message).await;
        Ok(())
    }

    async fn command_char(&self, arg: &str) -> Result<()> {
        let character = u8::from_str_radix(arg, 16)?;
        let chat_msg = format!("0x{:02x?}={}", character, character as char);

        let message = if self.protocol < Protocol::Tibia501 {
            self.prepare_chat(
                ChatType::Normal,
                &chat_msg,
                Some(&self.player),
                Some(self.player.position),
            )
            .await?
        } else {
            self.prepare_chat(
                ChatType::RedConsoleWhiteScreen,
                &chat_msg,
                None,
                Some(self.player.position),
            )
            .await?
        };

        self.queue_message(message).await;
        Ok(())
    }

    async fn command_echo(&self) -> Result<()> {
        let mut buf = Cursor::new(Vec::<u8>::new());
        buf.write_header(HeaderSend::Echo, self.protocol).await?;
        self.queue_message(buf.into_inner()).await;
        Ok(())
    }

    async fn command_item(&self, slot: &str, item: &str) -> Result<()> {
        let slot: InventorySlot = slot.parse::<u8>()?.try_into()?;
        let item = u16::from_str_radix(item, 16)?;
        log::trace!("Giving item 0x{item:04x?} on slot {slot:?}");
        self.queue_message(self.prepare_equipped_item(slot, item, 0).await?)
            .await;
        Ok(())
    }

    async fn command_item_right_hand(&self, item: &str) -> Result<()> {
        let slot = InventorySlot::RightHand;
        let item = u16::from_str_radix(item, 16)?;
        log::trace!("Giving item 0x{item:04x?} on slot {slot:?}");
        self.queue_message(self.prepare_equipped_item(slot, item, 0).await?)
            .await;
        Ok(())
    }

    async fn command_magic_effect(&self, effect: &str) -> Result<()> {
        let effect = effect.parse::<u8>()?.try_into()?;
        self.queue_message(
            self.prepare_magic_effect(effect, self.player.position)
                .await?,
        )
        .await;
        Ok(())
    }

    async fn command_motd(&self, args: Vec<&str>) -> Result<()> {
        static NEXT_MOTD: AtomicU16 = AtomicU16::new(0x0102);
        let message_number = NEXT_MOTD.fetch_add(1, Ordering::SeqCst);
        self.queue_message(
            self.prepare_message_of_the_day(message_number, &args.join(" "))
                .await?,
        )
        .await;
        Ok(())
    }

    async fn command_chat(&self) -> Result<()> {
        static NEXT_TYPE: AtomicU8 = AtomicU8::new(0x41);
        let chat_type = NEXT_TYPE.fetch_add(1, Ordering::SeqCst).try_into()?;

        let msg = &format!("chat_type=0x{chat_type:02x?}");
        log::trace!("{msg}");

        self.queue_message(
            self.prepare_chat(
                chat_type,
                msg,
                Some(&self.player),
                Some(self.player.position),
            )
            .await?,
        )
        .await;
        Ok(())
    }

    async fn command_outfit(&self, outfit: &str) -> Result<()> {
        let outfit = outfit.parse::<u8>()?.try_into()?;
        let outfit_colors = OutfitColors::new(0, 0, 0, 0);
        self.queue_message(
            self.prepare_update_outfit(self.player_id, outfit, outfit_colors)
                .await?,
        )
        .await;
        Ok(())
    }

    async fn command_change_direction(&self, direction: &str) -> Result<()> {
        let direction = direction.parse::<u8>()?.try_into()?;

        let mut msg = self
            .prepare_update_object(
                self.player.position,
                crate::constants::ObjectUpdateType::Update,
                1,
            )
            .await?;
        msg.extend(
            self.prepare_change_direction(self.player.id, direction)
                .await?,
        );
        self.queue_message(msg).await;
        Ok(())
    }

    async fn command_green_chat(&self, args: Vec<&str>) -> Result<()> {
        let chat_msg = args.join(" ");
        self.queue_message(
            self.prepare_green_chat(ChatType::Normal, &chat_msg, None, None)
                .await?,
        )
        .await;
        Ok(())
    }
}
