#[allow(dead_code)]
pub trait Colour {
    fn to_rgb(&self) -> (u8, u8, u8);
    fn to_rgba(&self) -> (u8, u8, u8, u8);
    fn to_argb(&self) -> (u8, u8, u8, u8);
    fn from_rgb(r: u8, g: u8, b: u8) -> Self;
    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ARGB(u32);
impl Colour for ARGB {
    fn to_rgb(&self) -> (u8, u8, u8) {
        todo!("to_rgb")
    }
    fn to_rgba(&self) -> (u8, u8, u8, u8) {
        todo!("to_rgba")
    }
    fn to_argb(&self) -> (u8, u8, u8, u8) {
        todo!("to_argb")
    }

    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let (r, g, b) = (r as u32, g as u32, b as u32);
        ARGB((r << 16) | (g << 8) | b)
    }
    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        let (r, g, b, a) = (r as u32, g as u32, b as u32, a as u32);
        ARGB((a << 24) | (r << 16) | (g << 8) | b)
    }
}
