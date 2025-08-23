#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[allow(dead_code)] // maybe one day I will use this
impl Rgba {
    pub const fn as_u32(self) -> u32 {
        (self.r as u32) << 24 | (self.g as u32) << 16 | (self.b as u32) << 8 | self.a as u32
    }

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_u32(colour: u32) -> Self {
        Self {
            r: ((colour >> 24) & 0xFF) as u8,
            g: ((colour >> 16) & 0xFF) as u8,
            b: ((colour >> 8) & 0xFF) as u8,
            a: (colour & 0xFF) as u8,
        }
    }
}
