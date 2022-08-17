use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

pub mod encoding;

pub struct InvalidChatQualifier(Option<char>);

#[repr(u8)]
#[derive(Debug, Copy, Clone, TryFromPrimitive, Eq, PartialEq)]
pub enum ChatType {
    RedScreenOnly = 0x41, //#a
    GreyConsoleOnly = 0x42,
    GreenConsoleYellowScreen = 0x43,
    GreyConsoleWhiteScreen = 0x44, //#b
    GreyConsoleYellowScreen = 0x45,
    GreyConsoleYellowScreen2 = 0x46,
    RedConsoleWhiteScreen = 0x47, //#g
    RedConsoleYellowScreen = 0x48,
    RedConsoleYellowScreen2 = 0x49,
    RedConsoleYellowScreen3 = 0x4a,
    GreenScreenOnly = 0x4d, //Green Anonymous??
    BlueConsoleYellowScreen = 0x4e,
    BlueConsoleYellowScreen2 = 0x4f,
    BlueConsoleWhiteScreen = 0x50, //private message?
    BlueConsoleYellowScreen3 = 0x51,
    BlueConsoleYellowScreen4 = 0x52,
    Normal = 0x53,
    Whisper = 0x57, //#w
    Yell = 0x59,    //#y
}

impl TryFrom<Option<char>> for ChatType {
    type Error = InvalidChatQualifier;

    fn try_from(value: Option<char>) -> std::result::Result<Self, Self::Error> {
        if let Some(value) = value {
            match value {
                'a' | 'A' => Ok(Self::RedScreenOnly),
                'g' | 'G' => Ok(Self::RedConsoleWhiteScreen),
                'y' | 'Y' => Ok(Self::Yell),
                'b' | 'B' => Ok(Self::GreyConsoleWhiteScreen),
                'w' | 'W' => Ok(Self::Whisper),
                _ => Err(InvalidChatQualifier(Some(value))),
            }
        } else {
            Err(InvalidChatQualifier(None))
        }
    }
}
