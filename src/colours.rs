// Is the colour trait implemented for each format
// with each function hanging off the type or off the instance

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGBA {
    pub const fn as_u32(&self) -> u32 {
        (self.r as u32) << 24 | (self.g as u32) << 16 | (self.b as u32) << 8 | self.a as u32
    }

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> RGBA {
        RGBA { r, g, b, a: 255 }
    }

    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> RGBA {
        RGBA { r, g, b, a }
    }

    pub const fn from_u32(colour: u32) -> RGBA {
        RGBA {
            r: ((colour >> 24) & 0xFF) as u8,
            g: ((colour >> 16) & 0xFF) as u8,
            b: ((colour >> 8) & 0xFF) as u8,
            a: (colour & 0xFF) as u8,
        }
    }
}

impl RGB {
    pub const fn as_u32(&self) -> u32 {
        (self.r as u32) << 24 | (self.g as u32) << 16 | (self.b as u32) << 8 | 255
    }

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> RGB {
        RGB { r, g, b }
    }

    pub const fn from_rgba(r: u8, g: u8, b: u8, _a: u8) -> RGB {
        RGB { r, g, b }
    }

    pub const fn from_u32(colour: u32) -> RGB {
        RGB {
            r: ((colour >> 24) & 0xFF) as u8,
            g: ((colour >> 16) & 0xFF) as u8,
            b: ((colour >> 8) & 0xFF) as u8,
        }
    }
}
