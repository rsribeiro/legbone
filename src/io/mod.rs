pub mod byteorder_async;

use crate::{
    character::Gender, character::OutfitColors, map::position::Position,
    network::header::HeaderSend, Protocol,
};
use anyhow::Result;
use async_std::{
    io::{Read, Write},
    prelude::*,
};
use byteorder_async::{AsyncReadByteOrder, AsyncWriteByteOrder};
use byteorder::LE;

impl<R: Read + Unpin> ReadExt for R {}

pub trait ReadExt: Read + Unpin + Sized {
    async fn read_gender(&mut self, protocol: Protocol) -> Result<Gender> {
        let gender = match self.read_u8().await? {
            1 => Gender::Male,
            0 if protocol < Protocol::Tibia501 => Gender::Female,
            2 if protocol >= Protocol::Tibia501 => Gender::Female,
            raw_gender => {
                log::error!(
                    "Unknown gender byte {} for protocol {:?}, assuming 'male'.",
                    raw_gender,
                    protocol
                );
                Gender::Male
            }
        };

        Ok(gender)
    }

    /// There are 4 colors, stored in 2 bytes. Each byte encodes 2 colors.
    async fn read_outfit_colors(&mut self) -> Result<OutfitColors> {
        let (legs, shoes) = self.read_u4().await?;
        let (head, body) = self.read_u4().await?;
        let unknown_byte = self.read_u8().await?;

        Ok(OutfitColors::new_with_unknown_byte(
            head,
            body,
            legs,
            shoes,
            unknown_byte,
        ))
    }

    /// The 4 lower and 4 higher bits in the byte stores different values.
    async fn read_u4(&mut self) -> Result<(u8, u8)> {
        let byte = self.read_u8().await?;
        let high = byte / 16;
        let low = byte % 16;
        Ok((high, low))
    }

    async fn read_position(&mut self, protocol: Protocol) -> Result<Position> {
        let position = if protocol == Protocol::Tibia103 {
            let x = self.read_u8().await?;
            let y = self.read_u8().await?;

            Position::new(x as u16, y as u16, 7)
        } else {
            let x = self.read_u16::<LE>().await?;
            let y = self.read_u16::<LE>().await?;
            let z = self.read_u8().await?;

            Position::new(x, y, z)
        };

        Ok(position)
    }

    /// Reads string until null byte is found. Unsafe because it can enter an infinite loop
    async unsafe fn read_string_until_end(&mut self, buf: &mut String) -> Result<usize> {
        loop {
            match self.read_u8().await? {
                b'\0' => break,
                c => buf.push(c as char),
            }
        }
        Ok(buf.len())
    }

    /// Reads string until null byte or max_size. Consumes max_size bytes from the stream
    async fn read_string(&mut self, buf: &mut String, max_size: u16) -> Result<usize> {
        for n in 1..=max_size {
            match self.read_u8().await? {
                b'\0' => {
                    self.skip(max_size - n).await?;
                    break;
                }
                c => buf.push(c as char),
            }
        }
        Ok(buf.len())
    }

    /// Skips an ammount of bytes. Is used in some places where the meaning of the received
    /// is currently unknown.
    async fn skip(&mut self, bytes: u16) -> Result<()> {
        let mut buf = vec![0_u8; bytes as usize];
        self.read_exact(&mut buf).await?;
        log::trace!("Skipped {} bytes: {:02x?}", bytes, buf);
        Ok(())
    }
}

impl<W: Write + Unpin> WriteExt for W {}

pub trait WriteExt: Write + Unpin + Sized {
    async fn write_outfit_colors(&mut self, outfit: OutfitColors) -> Result<()> {
        self.write_u4(outfit.legs, outfit.shoes).await?;
        self.write_u4(outfit.head, outfit.body).await?;
        self.write_u8(outfit.unknown_byte).await?;
        Ok(())
    }

    async fn write_outfit_colors_with_unknown_byte(
        &mut self,
        outfit: OutfitColors,
        unknown_byte: u8,
    ) -> Result<()> {
        self.write_u4(outfit.legs, outfit.shoes).await?;
        self.write_u4(outfit.head, outfit.body).await?;
        self.write_u8(unknown_byte).await?;
        Ok(())
    }

    ///Writes two u4's as an u8
    async fn write_u4(&mut self, high: u8, low: u8) -> Result<()> {
        self.write_u8((high << 4) + low).await?;
        Ok(())
    }

    async fn write_gender(&mut self, gender: Gender, protocol: Protocol) -> Result<()> {
        match gender {
            Gender::Male => self.write_u8(1).await?,
            Gender::Female if protocol < Protocol::Tibia501 => self.write_u8(0).await?,
            Gender::Female => self.write_u8(2).await?,
        }

        Ok(())
    }

    /// For Tibia 3.x and 4.x the client receives an u8 header. Later clients use u16
    /// Client 1.03 seems to need 4 bytes before header. Meaning of bytes unknown
    async fn write_header(&mut self, header: HeaderSend, protocol: Protocol) -> Result<()> {
        if protocol > Protocol::Tibia400 {
            self.write_u16::<LE>(header as u16).await?;
        } else {
            if protocol == Protocol::Tibia103 {
                self.write_zeroes(4).await?;
            }
            self.write_u8(header as u8).await?;
        }
        Ok(())
    }

    async fn write_position(&mut self, position: Position, protocol: Protocol) -> Result<()> {
        if protocol == Protocol::Tibia103 {
            self.write_u8(position.x as u8).await?;
            self.write_u8(position.y as u8).await?;
        } else {
            self.write_u16::<LE>(position.x).await?;
            self.write_u16::<LE>(position.y).await?;
            self.write_u8(position.z).await?;
        }

        Ok(())
    }

    async fn write_length_and_string(&mut self, string: &str) -> Result<()> {
        self.write_u16::<LE>(string.len() as u16).await?;
        self.write_all(string.as_bytes()).await?;
        Ok(())
    }

    async fn write_null_terminated_string(&mut self, string: &str) -> Result<()> {
        self.write_all(string.as_bytes()).await?;
        self.write_u8(0).await?;
        Ok(())
    }

    async fn write_string_with_fixed_length(&mut self, string: &str, length: u16) -> Result<()> {
        if string.len() > length as usize {
            self.write_all(string[0..length as usize].as_bytes())
                .await?;
        } else {
            self.write_all(string.as_bytes()).await?;
            self.write_zeroes(length as usize - string.len()).await?;
        }

        Ok(())
    }

    async fn write_zeroes(&mut self, length: usize) -> Result<()> {
        self.write_repeated_number(0, length).await
    }

    async fn write_repeated_number(&mut self, number: u8, length: usize) -> Result<()> {
        self.write_all(&vec![number; length]).await?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use async_std::io::Cursor;

    #[async_std::test]
    async fn test_read_write_gender() -> Result<()> {
        let gender_before = Gender::Female;

        let mut buf = Cursor::new(Vec::<u8>::new());
        buf.write_gender(gender_before, Protocol::Tibia300).await?;

        let mut buf = Cursor::new(buf.into_inner());
        let gender_after = buf.read_gender(Protocol::Tibia300).await?;

        assert_eq!(gender_before, gender_after);

        Ok(())
    }

    #[async_std::test]
    async fn test_read_write_outfit_colors() -> Result<()> {
        let outfit_before = OutfitColors::new_with_unknown_byte(1, 2, 3, 4, 5);

        let mut buf = Cursor::new(Vec::<u8>::new());
        buf.write_outfit_colors(outfit_before).await?;

        let mut buf = Cursor::new(buf.into_inner());
        let outfit_after = buf.read_outfit_colors().await?;

        assert_eq!(outfit_before, outfit_after);

        Ok(())
    }

    #[async_std::test]
    async fn test_read_write_position() -> Result<()> {
        let position_before = Position::new(1, 2, 3);
        let protocol = Protocol::Tibia650;

        let mut buf = Cursor::new(Vec::<u8>::new());
        buf.write_position(position_before, protocol).await?;

        let mut buf = Cursor::new(buf.into_inner());
        let position_after = buf.read_position(protocol).await?;

        assert_eq!(position_before, position_after);

        Ok(())
    }

    #[async_std::test]
    async fn test_read_write_u4() -> Result<()> {
        let high_before = 1;
        let low_before = 2;

        let mut buf = Cursor::new(Vec::<u8>::new());
        buf.write_u4(high_before, low_before).await?;

        let mut buf = Cursor::new(buf.into_inner());
        let (high_after, low_after) = buf.read_u4().await?;

        assert_eq!(high_before, high_after);
        assert_eq!(low_before, low_after);

        Ok(())
    }
}
