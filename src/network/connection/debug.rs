use std::{
    convert::TryInto,
    sync::atomic::{
        AtomicU8,
        AtomicU16,
        Ordering
    }
};
use async_std::io::Cursor;
use crate::{
    constants::MagicEffect,
    network::header::HeaderSend,
    character::{
        CharacterUpdateType,
        player::InventorySlot,
        Outfit
    },
    Protocol,
    chat::ChatType,
    io::WriteExt
};
use super::Connection;
use anyhow::Result;

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
    async fn debug_command(&self, command: &str, args: Vec<&str>) -> Result<()> {
        log::debug!("Received debug command {:?}", command);
        match command {
            "chars" => {
                let chars: Vec<u8> = (0x20_u8..=0x7f).collect();
                let chat_msg = unsafe { std::str::from_utf8_unchecked(&chars) };

                let message = if self.protocol < Protocol::Tibia501 {
                    self.prepare_chat(ChatType::Normal, chat_msg, Some(&self.player), Some(self.player.position)).await?
                } else {
                    self.prepare_chat(ChatType::RedConsoleWhiteScreen, chat_msg, None, Some(self.player.position)).await?
                };

                self.queue_message(message).await;
                Ok(())
            },
            "char"=> {
                let character = u8::from_str_radix(args[0], 16)?;
                let chat_msg = format!("0x{:02x?}={}", character, character as char);

                let message = if self.protocol < Protocol::Tibia501 {
                    self.prepare_chat(ChatType::Normal, &chat_msg, Some(&self.player), Some(self.player.position)).await?
                } else {
                    self.prepare_chat(ChatType::RedConsoleWhiteScreen, &chat_msg, None, Some(self.player.position)).await?
                };

                self.queue_message(message).await;
                Ok(())
            },
            "echo" => {
                let mut buf = Cursor::new(Vec::<u8>::new());
                buf.write_header(HeaderSend::Echo, self.protocol).await?;
                self.queue_message(buf.into_inner()).await;
                Ok(())
            },
            "item" => {
                let slot: InventorySlot = args[0].parse::<u8>()?.try_into()?;
                let item = u16::from_str_radix(args[1], 16)?;
                log::trace!("Giving item 0x{:04x?} on slot {:?}", item, slot);
                self.queue_message(self.prepare_equipped_item(slot, item, 0).await?).await;
                Ok(())
            },
            "i" => {
                let slot = InventorySlot::RightHand;
                let item = u16::from_str_radix(args[0], 16)?;
                log::trace!("Giving item 0x{:04x?} on slot {:?}", item, slot);
                self.queue_message(self.prepare_equipped_item(slot, item, 0).await?).await;
                Ok(())
            }
            "stats" => Ok(self.queue_message(self.prepare_stats().await?).await),
            "skills" => Ok(self.queue_message(self.prepare_skills().await?).await),
            "me" => {
                let effect = args[0].parse::<u8>()?.try_into()?;
                self.queue_message(self.prepare_magic_effect(effect, self.player.position).await?).await;
                Ok(())
            },
            "wlight"=> Ok(self.queue_message(self.prepare_world_light(args[0].parse::<u8>()?).await?).await),
            "plight"=> Ok(self.queue_message(self.prepare_update_character(self.player.id, CharacterUpdateType::LightLevel, args[0].parse::<u8>()?).await?).await),
            "userlist" => Ok(self.queue_message(self.prepare_user_list().await?).await),
            "userinfo" => Ok(self.queue_message(self.prepare_user_info(args[0]).await?).await),
            "info" => Ok(self.queue_message(self.prepare_info(&args.join(" ")).await?).await),
            "error" => Ok(self.queue_message(self.prepare_error(&args.join(" ")).await?).await),
            "motd" => {
                static NEXT_MOTD: AtomicU16 = AtomicU16::new(0x0102);
                let message_number = NEXT_MOTD.fetch_add(1, Ordering::SeqCst);

                Ok(self.queue_message(self.prepare_message_of_the_day(message_number, &args.join(" ")).await?).await)
            },
            "status" => Ok(self.queue_message(self.prepare_status_message(&args.join(" ")).await?).await),
            "panic" => panic!("{}", args.join(" ")),
            "chat" => {
                static NEXT_TYPE: AtomicU8 = AtomicU8::new(0x41);
                let chat_type = NEXT_TYPE.fetch_add(1, Ordering::SeqCst).try_into()?;

                let msg = &format!("chat_type=0x{:02x?}", chat_type);
                log::trace!("{}", msg);

                self.queue_message(self.prepare_chat(chat_type, msg, Some(&self.player), Some(self.player.position)).await?).await;
                Ok(())
            },
            "outfit" => {
                let outfit = args[0].parse::<u8>()?.try_into()?;
                let outfit_colors = Outfit::new(0, 0, 0, 0);
                self.queue_message(self.prepare_update_outfit(self.player_id, outfit, outfit_colors).await?).await;
                Ok(())
            }
            "cd" => {
                let direction = args[0].parse::<u8>()?.try_into()?;

                let mut msg = self.prepare_update_object(self.player.position, crate::constants::ObjectUpdateType::Update, 1).await?;
                msg.extend(self.prepare_change_direction(self.player.id, direction).await?);
                self.queue_message(msg).await;
                Ok(())
            },
            "gc" => {
                let chat_msg = args.join(" ");
                self.queue_message(self.prepare_green_chat(ChatType::Normal, &chat_msg, None, None).await?).await;
                Ok(())
            },
            "u0" => {
                Ok(self.queue_message(self.prepare_unknown_0x0000().await?).await)
            },
            "uf" => {
                Ok(self.queue_message(self.prepare_unknown_0x000f().await?).await)
            }
            "u33" => {
                Ok(self.queue_message(self.prepare_unknown_0x0033().await?).await)
            }
            "u34" => {
                Ok(self.queue_message(self.prepare_unknown_0x0034().await?).await)
            }
            _ => Ok(self.queue_message(self.prepare_magic_effect(MagicEffect::Puff, self.player.position).await?).await)
        }
    }
}
