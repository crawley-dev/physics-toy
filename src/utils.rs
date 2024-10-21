// Is the colour trait implemented for each format
// with each function hanging off the type or off the instance

use winit::dpi::{PhysicalPosition, PhysicalSize};

pub const KEY_COOLDOWN_MS: u64 = 250;
pub const INIT_WIDTH: u32 = 800;
pub const INIT_HEIGHT: u32 = 600;
pub const INIT_SCALE: u32 = 8;
pub const INIT_DRAW_SIZE: u32 = 10;
pub const SIM_MAX_SCALE: u32 = 10;
pub const TARGET_FPS: f64 = 144.0;
pub const OUTPUT_EVERY_N_FRAMES: u64 = 30;
pub const FRAME_TIME_MS: f64 = 1000.0 / TARGET_FPS;
pub const INIT_TITLE: &str = "Conway's Game of Life";

// Types are identical to winit, but I want explicit type errors
macro_rules! create_vec2 {
    ($name:ident, $param1:ident, $param2: ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name<T: num::Integer> {
            pub $param1: T,
            pub $param2: T,
        }
        impl<T: num::Integer> $name<T> {
            pub fn new($param1: T, $param2: T) -> Self {
                Self { $param1, $param2 }
            }
        }
        impl<T: num::Integer> From<PhysicalSize<T>> for $name<T> {
            fn from(size: PhysicalSize<T>) -> Self {
                Self {
                    $param1: size.width,
                    $param2: size.height,
                }
            }
        }
        impl<T: num::Integer> From<PhysicalPosition<T>> for $name<T> {
            fn from(pos: PhysicalPosition<T>) -> Self {
                Self {
                    $param1: pos.x,
                    $param2: pos.y,
                }
            }
        }
        impl From<$name<u32>> for PhysicalSize<u32> {
            fn from(size: $name<u32>) -> Self {
                Self {
                    width: size.$param1,
                    height: size.$param2,
                }
            }
        }
        impl From<$name<u32>> for PhysicalPosition<u32> {
            fn from(pos: $name<u32>) -> Self {
                Self {
                    x: pos.$param1,
                    y: pos.$param2,
                }
            }
        }
    };
}

create_vec2!(CellPos, x, y);
create_vec2!(WindowPos, x, y);
create_vec2!(CellSize, width, height);
create_vec2!(WindowSize, width, height);

impl<T: num::Integer + Copy> CellPos<T> {
    pub fn to_window(self, scale: T) -> WindowPos<T> {
        WindowPos {
            x: self.x * scale,
            y: self.y * scale,
        }
    }
}

impl<T: num::Integer + Copy> WindowPos<T> {
    pub fn to_cell(self, scale: T) -> CellPos<T> {
        CellPos {
            x: self.x / scale,
            y: self.y / scale,
        }
    }
}

impl<T: num::Integer + Copy> CellSize<T> {
    pub fn to_window(self, scale: T) -> WindowSize<T> {
        WindowSize {
            width: self.width * scale,
            height: self.height * scale,
        }
    }
}

impl<T: num::Integer + Copy> WindowSize<T> {
    pub fn to_cell(self, scale: T) -> CellSize<T> {
        CellSize {
            width: self.width / scale,
            height: self.height / scale,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub enum Shape {
    Circle { radius: u32 },
    Square { side: u32 },
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
