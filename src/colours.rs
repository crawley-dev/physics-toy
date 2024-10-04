// Is the colour trait implemented for each format
// with each function hanging off the type or off the instance

pub type PackedRGB = (u8, u8, u8);
use sdl2::pixels::Color;

pub trait Create<T> {
    fn create(value: T) -> Self;
}

impl Create<u32> for PackedRGB {
    fn create(value: u32) -> Self {
        let r = (value >> 16) as u8;
        let g = (value >> 8) as u8;
        let b = value as u8;
        (r, g, b)
    }
}

impl Create<(u8, u8, u8)> for PackedRGB {
    fn create(value: (u8, u8, u8)) -> Self {
        value
    }
}

impl Create<Color> for PackedRGB {
    fn create(value: Color) -> Self {
        (value.r, value.g, value.b)
    }
}

impl Create<PackedRGB> for Color {
    fn create(value: PackedRGB) -> Self {
        Color {
            r: value.0 << 16,
            g: value.1 << 8,
            b: value.2,
            a: 255,
        }
    }
}

impl Create<PackedRGB> for u32 {
    fn create(value: PackedRGB) -> Self {
        (value.0 as u32) << 16 | (value.1 as u32) << 8 | value.2 as u32
    }
}
