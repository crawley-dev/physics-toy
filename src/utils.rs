// Is the colour trait implemented for each format
// with each function hanging off the type or off the instance

use core::fmt;
use educe::Educe;
use num::{Num, ToPrimitive};
use winit::dpi::{PhysicalPosition, PhysicalSize};

// Colours (cell_sim.rs / gravity_sim.rs)
pub const GREEN: Rgba = Rgba::from_rgb(40, 255, 40);
pub const WHITE: Rgba = Rgba::from_rgb(255, 255, 255);
pub const BACKGROUND: Rgba = Rgba::from_rgb(44, 44, 44);

// simulation constants (gravity_sim.rs)
pub const MULTIPLIER: f64 = 2.0;
pub const RESISTANCE: f64 = 0.99;
pub const INIT_PARTICLES: usize = 0;
pub const GRAV_CONST: f64 = 6.67430e-18; // reduced by 10 for better precision?

// init (main.rs)
pub const INIT_WIDTH: u32 = 1600;
pub const INIT_HEIGHT: u32 = 1200;
pub const INIT_SCALE: u32 = 3;
pub const INIT_DRAW_SIZE: u32 = 8;
pub const INIT_TITLE: &str = "Gravity Sim";
pub const SIM_MAX_SCALE: u32 = 10;
pub const CAMERA_SPEED: f64 = 0.1;

// timing (app.rs)
pub const MOUSE_COOLDOWN_MS: u64 = 250;
pub const KEY_COOLDOWN_MS: u64 = 100;
pub const TARGET_FPS: f64 = 60.0;
pub const FRAME_TIME_MS: f64 = 1000.0 / TARGET_FPS;
pub const MS_BUFFER: f64 = 3.0;

macro_rules! create_vec2 {
    ($name:ident, $param1:ident, $param2: ident) => {
        #[derive(Educe, Clone, Copy, PartialEq, Eq)]
        #[educe(Debug(named_field = false))]
        pub struct $name<T: Copy> {
            pub $param1: T,
            pub $param2: T,
        }
        impl<T: Num + Copy + ToPrimitive> $name<T> {
            pub fn clamp(self, min: $name<T>, max: $name<T>) -> Self
            where
                T: PartialOrd,
            {
                Self {
                    $param1: num::clamp(self.$param1, min.$param1, max.$param1),
                    $param2: num::clamp(self.$param2, min.$param2, max.$param2),
                }
            }

            pub fn into<T2: Num + Copy + From<T>>(self) -> $name<T2> {
                $name {
                    $param1: self.$param1.into(),
                    $param2: self.$param2.into(),
                }
            }

            pub fn map<T2: Num + Copy, F: Fn(T) -> T2>(self, f: F) -> $name<T2> {
                $name {
                    $param1: f(self.$param1),
                    $param2: f(self.$param2),
                }
            }

            pub fn add<T2: Num + Copy + Into<T>>(&self, p1: T2, p2: T2) -> Self {
                Self {
                    $param1: self.$param1 + p1.into(),
                    $param2: self.$param2 + p2.into(),
                }
            }

            pub fn sub<T2: Num + Copy + Into<T>>(&self, p1: T2, p2: T2) -> Self {
                Self {
                    $param1: self.$param1 - p1.into(),
                    $param2: self.$param2 - p2.into(),
                }
            }

            pub fn div<T2: Num + Copy + Into<T>>(&self, p1: T2, p2: T2) -> Self {
                Self {
                    $param1: self.$param1 / p1.into(),
                    $param2: self.$param2 / p2.into(),
                }
            }

            pub fn mul<T2: Num + Copy + Into<T>>(&self, p1: T2, p2: T2) -> Self {
                Self {
                    $param1: self.$param1 * p1.into(),
                    $param2: self.$param2 * p2.into(),
                }
            }
        }

        impl<T: Num + Copy> From<(T, T)> for $name<T> {
            fn from((a, b): (T, T)) -> Self {
                Self {
                    $param1: a,
                    $param2: b,
                }
            }
        }
        impl<T: Num + Copy> From<PhysicalSize<T>> for $name<T> {
            fn from(size: PhysicalSize<T>) -> Self {
                Self {
                    $param1: size.width,
                    $param2: size.height,
                }
            }
        }
        impl<T: Num + Copy> From<PhysicalPosition<T>> for $name<T> {
            fn from(pos: PhysicalPosition<T>) -> Self {
                Self {
                    $param1: pos.x,
                    $param2: pos.y,
                }
            }
        }
        impl<T: Num + Copy> From<$name<T>> for PhysicalSize<T> {
            fn from(size: $name<T>) -> Self {
                Self {
                    width: size.$param1,
                    height: size.$param2,
                }
            }
        }
        impl<T: Num + Copy> From<$name<T>> for PhysicalPosition<T> {
            fn from(pos: $name<T>) -> Self {
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

// region: Impl Vec2 Items
impl<T: Num + Copy> GamePos<T> {
    pub fn to_window(self, scale: T) -> WindowPos<T> {
        WindowPos {
            x: self.x * scale,
            y: self.y * scale,
        }
    }
    pub fn to_size(self) -> GameSize<T> {
        GameSize {
            width: self.x,
            height: self.y,
        }
    }
}
impl<T: Num + Copy> WindowPos<T> {
    pub fn to_game(self, scale: T) -> GamePos<T> {
        GamePos {
            x: self.x / scale,
            y: self.y / scale,
        }
    }
    pub fn to_size(self) -> WindowSize<T> {
        WindowSize {
            width: self.x,
            height: self.y,
        }
    }
}
impl<T: Num + Copy> GameSize<T> {
    pub fn to_window(self, scale: T) -> WindowSize<T> {
        WindowSize {
            width: self.width * scale,
            height: self.height * scale,
        }
    }

    pub fn to_pos(self) -> GamePos<T> {
        GamePos {
            x: self.width,
            y: self.height,
        }
    }
}
impl<T: Num + Copy> WindowSize<T> {
    pub fn to_game(self, scale: T) -> GameSize<T> {
        GameSize {
            width: self.width / scale,
            height: self.height / scale,
        }
    }

    pub fn to_pos(self) -> WindowPos<T> {
        WindowPos {
            x: self.width,
            y: self.height,
        }
    }
}
// endregion

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)] // don't match shape, I index into it (app::handle_inputs)
pub enum Shape {
    CircleOutline,
    CircleFill,
    SquareCentered,
    Count,
}

impl Shape {
    pub fn draw<F: FnMut(i32, i32)>(self, size: u32, mut lambda: F) {
        match self {
            Self::CircleOutline => {
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
            Self::CircleFill => {
                let mut x = 0;
                let mut y = size as i32;
                let mut d = 3 - 2 * size as i32;
                let mut draw_line = |x1, x2, y| {
                    for x in x1..x2 {
                        lambda(x, y);
                    }
                };
                let mut draw_circle = |x: i32, y: i32| {
                    draw_line(-x, x, y);
                    draw_line(-x, x, -y);
                    draw_line(-y, y, x);
                    draw_line(-y, y, -x);
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
            Self::SquareCentered => {
                let half = (size / 2) as i32;
                for y_off in -(half)..(half) {
                    for x_off in -(half)..(half) {
                        lambda(x_off, y_off);
                    }
                }
            }
            Self::Count => {
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

pub fn fmt_limited_precision<T: std::fmt::Debug>(x: T, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:.2?}", x) // Specify precision here
}
