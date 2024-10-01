// Is the colour trait implemented for each format
// with each function hanging off the type or off the instance

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ARGB(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RGB(pub u8, pub u8, pub u8);

impl From<RGB> for ARGB {
    fn from(value: RGB) -> Self {
        ARGB((value.0 as u32) << 16 | (value.1 as u32) << 8 | value.2 as u32)
    }
}

impl From<ARGB> for RGB {
    fn from(value: ARGB) -> Self {
        let r = (u32::from(value) >> 16) as u8;
        let g = (u32::from(value) >> 8) as u8;
        let b = u32::from(value) as u8;
        RGB(r, g, b)
    }
}

impl From<ARGB> for u32 {
    fn from(value: ARGB) -> Self {
        value.0
    }
}

impl From<RGB> for (u8, u8, u8) {
    fn from(value: RGB) -> Self {
        (value.0, value.1, value.2)
    }
}
