//https://www.compart.com/en/unicode/charsets/x-DIN_66303
//https://www.compart.com/en/unicode/charsets/DIN_66003
//https://en.wikipedia.org/wiki/ISO/IEC_646
//https://en.wikipedia.org/wiki/ISO/IEC_8859-1

///Text console and screen seems to use different charsets, couldn't find out exactly
///what is going on. Translating these bytes keeps greater coherence between text
///console and screen chars
pub fn translate(input: &str) -> Vec<u8> {
    input
        .as_bytes()
        .iter()
        .map(|&c| translate_char(c))
        .collect::<Vec<u8>>()
}

pub fn translate_upper(input: &str) -> Vec<u8> {
    input
        .as_bytes()
        .iter()
        .map(|&c| translate_char_upper(c))
        .collect::<Vec<u8>>()
}

const fn translate_char(input: u8) -> u8 {
    match input {
        0x5b => 0xc4, //Ä
        0x5c => 0xd6, //Ö
        0x5d => 0xdc, //Ü
        0x7b => 0xe4, //ä
        0x7c => 0xf6, //ö
        0x7d => 0xfc, //ü
        0x7f => 0xdf, //ß
        _ => input,
    }
}

const fn translate_char_upper(input: u8) -> u8 {
    match input {
        0x5b => 0xc4, //Ä
        0x5c => 0xd6, //Ö
        0x5d => 0xdc, //Ü
        0x7b => 0xc4, //ä
        0x7c => 0xd6, //ö
        0x7d => 0xdc, //ü
        0x7f => 0xdf, //ß
        _ => input,
    }
}
