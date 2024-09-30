pub trait Colour {
    fn to_rgb(&self) -> (u8, u8, u8);
    fn to_rgba(&self) -> (u8, u8, u8, u8);
    fn to_argb(&self) -> (u8, u8, u8, u8);
    fn from_rgb(&mut self, r: u8, g: u8, b: u8);
    fn from_rgba(&mut self, r: u8, g: u8, b: u8, a: u8);
}

type ARGB = u32;
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

    fn from_rgb(&mut self, r: u8, g: u8, b: u8) {
        let (r, g, b) = (r as u32, g as u32, b as u32);
        self.colour = (r << 16) | (g << 8) | b;
    }
    fn from_rgba(&mut self, r: u8, g: u8, b: u8, a: u8) {
        let (r, g, b, a) = (r as u32, g as u32, b as u32, a as u32);
        self.colour = (a << 24) | (r << 16) | (g << 8) | b;
    }
}
