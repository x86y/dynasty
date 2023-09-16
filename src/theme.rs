use std::num::ParseIntError;

use iced::Color;

macro_rules! str {
    ($a: expr) => {
        u8::from_str_radix($a, 16)? as f32 / 255.0
    };
}

pub fn h2c(h: &str) -> Result<Color, ParseIntError> {
    let r = str![&h[0..2]];
    let g = str![&h[2..4]];
    let b = str![&h[4..6]];
    Ok(Color { r, g, b, a: 1.0 })
}
