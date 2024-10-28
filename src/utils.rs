// Is the colour trait implemented for each format
// with each function hanging off the type or off the instance

use winit::dpi::{PhysicalPosition, PhysicalSize};

pub const KEY_COOLDOWN_MS: u64 = 100;
pub const INIT_WIDTH: u32 = 800;
pub const INIT_HEIGHT: u32 = 600;
pub const INIT_SCALE: u32 = 4;
pub const INIT_DRAW_SIZE: u32 = 4;
pub const SIM_MAX_SCALE: u32 = 10;
pub const TARGET_FPS: f64 = 144.0;
pub const FRAME_TIME_MS: f64 = 1000.0 / TARGET_FPS;
pub const INIT_TITLE: &str = "Conway's Game of Life";

// Types are identical to winit, but I want explicit type errors
macro_rules! create_vec2 {
    ($name:ident, $param1:ident, $param2: ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name<T: num::Num + Copy> {
            pub $param1: T,
            pub $param2: T,
        }
        impl<T: num::Num + Copy> $name<T> {
            pub fn new($param1: T, $param2: T) -> Self {
                Self { $param1, $param2 }
            }
        }
        impl<T: num::Num + Copy> From<PhysicalSize<T>> for $name<T> {
            fn from(size: PhysicalSize<T>) -> Self {
                Self {
                    $param1: size.width,
                    $param2: size.height,
                }
            }
        }
        impl<T: num::Num + Copy> From<PhysicalPosition<T>> for $name<T> {
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

create_vec2!(GamePos, x, y);
create_vec2!(WindowPos, x, y);
create_vec2!(GameSize, width, height);
create_vec2!(WindowSize, width, height);

impl<T: num::Num + Copy> GamePos<T> {
    pub fn to_window(self, scale: T) -> WindowPos<T> {
        WindowPos {
            x: self.x * scale,
            y: self.y * scale,
        }
    }
}
impl<T: num::Num + Copy> WindowPos<T> {
    pub fn to_game(self, scale: T) -> GamePos<T> {
        GamePos {
            x: self.x / scale,
            y: self.y / scale,
        }
    }
}
impl<T: num::Num + Copy> GameSize<T> {
    pub fn to_window(self, scale: T) -> WindowSize<T> {
        WindowSize {
            width: self.width * scale,
            height: self.height * scale,
        }
    }
}
impl<T: num::Num + Copy> WindowSize<T> {
    pub fn to_game(self, scale: T) -> GameSize<T> {
        GameSize {
            width: self.width / scale,
            height: self.height / scale,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shape {
    CircleOutline,
    CircleFill,
    SquareCentered,
    Count,
}

impl Shape {
    pub fn draw<F: FnMut(i32, i32)>(&self, size: u32, mut lambda: F) {
        match self {
            Shape::CircleOutline => {
                let mut x = 0;
                let mut y = size as i32;
                let mut d = 3 - 2 * size as i32;
                let mut draw_circle = |x, y| {
                    lambda(x, y);
                    lambda(-x, y);
                    lambda(x, -y);
                    lambda(-x, -y);
                    lambda(y, x);
                    lambda(-y, x);
                    lambda(y, -x);
                    lambda(-y, -x);
                };
                draw_circle(x, y);
                while x < y {
                    if d < 0 {
                        d = d + 4 * x + 6;
                    } else {
                        y -= 1;
                        d = d + 4 * (x - y) + 10;
                    }
                    x += 1;
                    draw_circle(x, y);
                }
            }
            Shape::CircleFill => {
                let r2 = size as i32 * size as i32;
                let area = r2 << 2;
                let rr = (size as i32) << 1;

                for i in 0..area {
                    let tx = (i % rr) - size as i32;
                    let ty = (i / rr) - size as i32;

                    if tx * tx + ty * ty <= r2 {
                        lambda(tx, ty);
                    }
                }
            }
            Shape::SquareCentered => {
                let half = (size / 2) as i32;
                for y_off in -(half)..(half) {
                    for x_off in -(half)..(half) {
                        lambda(x_off, y_off);
                    }
                }
            }
            Shape::Count => {
                panic!("Shape::Count is not a valid shape");
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn as_u32(&self) -> u32 {
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
