// Is the colour trait implemented for each format
// with each function hanging off the type or off the instance

use educe::Educe;
use num::{pow::Pow, Num, NumCast};
use paste::paste;
use rand::distributions::uniform::UniformDuration;
use std::{
    cell::UnsafeCell,
    fmt,
    marker::PhantomData,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};
use wgpu::hal::auxil::db::intel::DEVICE_SKY_LAKE_MASK;
use winit::dpi::{PhysicalPosition, PhysicalSize};

// Colours (cell_sim.rs / gravity_sim.rs)
pub const GREEN: Rgba = Rgba::from_rgb(40, 255, 40);
pub const WHITE: Rgba = Rgba::from_rgb(255, 255, 255);
pub const DGRAY: Rgba = Rgba::from_rgb(44, 44, 44);
pub const RED: Rgba = Rgba::from_rgb(255, 40, 40);

// Generic Parameters (*)
pub const INIT_TITLE: &str = "Gravity Sim";
pub const INIT_WIDTH: u32 = 1600;
pub const INIT_HEIGHT: u32 = 1200;
pub const INIT_SCALE: u32 = 3;
pub const INIT_DRAW_SIZE: i32 = 8;
pub const SIM_MAX_SCALE: u32 = 10;
pub const MAX_DRAW_SIZE: i32 = 500;

// timing (app.rs)
pub const MOUSE_HOLD_THRESHOLD_MS: u64 = 250;
pub const MOUSE_PRESS_COOLDOWN_MS: u64 = 100;
pub const MOUSE_DRAG_THRESHOLD_PX: f64 = 5.0; // TODO(TOM): vary with dpi
pub const KEY_COOLDOWN_MS: u64 = 100;
pub const TARGET_FPS: f64 = 120.0;
pub const FRAME_TIME_MS: f64 = 1000.0 / TARGET_FPS;
pub const MS_BUFFER: f64 = 3.0;

// gravity_sim.rs
pub const MOUSE_DRAWBACK_MULTIPLIER: f64 = 10.0;
pub const CAMERA_RESISTANCE: f64 = 0.97;
pub const CAMERA_SPEED: f64 = 0.1;

pub const SMALL_VALUE: f64 = 1e-6;
pub const COLLISION_RESTITUTION: f64 = 0.8;
pub const PHYSICS_MULTIPLIER: f64 = 4e-11;
pub const PHYSICS_RESISTANCE: f64 = 0.999;

// SIM CONSTANTS
pub const DISTANCE_SCALE: f64 = 1.1970456e+15; // pixel to meters conversion scale. (not logarithmic!)

pub const GRAV_CONST: f64 = 6.6743e-11;
pub const EARTH_MASS: f64 = 5.972e24;
pub const EARTH_DENSITY: f64 = 5514.0 * DISTANCE_SCALE;
pub const SUN_MASS: f64 = 1.989e30;

/*
    Particle Conversion to real world units -- to not spaz float precision
    - pos: 1.0 ==  1e4 km
    - vel: 1.0 == 1e3km/s pixels/frame >> 1e4 km/(1/TARGET_FPS) << 1000 times more than it should be (make grav const e-4 less)
    - mass: 1.0 == 1e20 kg
    - radius: 1.0 == 1e4 km

    - distance: 1.0 == 1e7 m (1e4 km)
    - mass: 1.0 == 1e20 kg
    - velocity: 1.0 == (1e7 m) / 1.0s (calc per frame, but mult by dx to get seconds)

    // TLDR: e-11 grav const for m, e-14 for km, e-18 for 1000km
*/

// pub const DISTANCE_SCALE: f64 = 1e-7;
// pub const MASS_SCALE: f64 = 1e-20;
// // pub const VELOCITY_SCALE: f64 = 1e3;
// pub const DENSITY_SCALE: f64 = MASS_SCALE / (DISTANCE_SCALE * DISTANCE_SCALE * DISTANCE_SCALE);

// pub const SUN_DENSITY: f64 = 1403.0 * DENSITY_SCALE; // kg/m3 --> val * 1e-20 * (1e7)^3
// * DENSITY_SCALE;
// pub const SUN_RADIUS: f64 = 696_340_000.0 * DISTANCE_SCALE; //69.634;
// pub const EARTH_RADIUS: f64 = 6_378_000.0 * DISTANCE_SCALE; //6.371;

// pub const SUN_DENSITY: f64 = 1403.0; // kg/m3 --> val * 1e-20 * (1e7)^3
// pub const EARTH_DENSITY: f64 = 5514.0;
// pub const SUN_RADIUS: f64 = 696_340_000.0; //69.634;
// pub const EARTH_RADIUS: f64 = 6_378_000.0; //6.371;

// region: Vec2
pub trait CoordSpace {}
macro_rules! create_coordinate_space {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name;
        impl CoordSpace for $name {}
    };
}

create_coordinate_space!(ScreenSpace); // Space of the window e.g. 720x480
create_coordinate_space!(RenderSpace); // Space of the simulation e.g. 360x240
create_coordinate_space!(WorldSpace); // Space of the world, any number, could be offscreen!
create_coordinate_space!(Unknown);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Scale<T: Num + Copy + Mul, Src: CoordSpace, Dst: CoordSpace>(T, PhantomData<(Src, Dst)>);
impl<T: Num + Copy + Mul, Src: CoordSpace, Dst: CoordSpace> Scale<T, Src, Dst> {
    pub fn new(val: T) -> Self {
        Self(val, PhantomData)
    }

    pub fn get(&self) -> T {
        self.0
    }
}
impl<T: fmt::Display + Num + Copy + Mul, Src: CoordSpace, Dst: CoordSpace> fmt::Debug
    for Scale<T, Src, Dst>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Scale({}, ({} -> {}))",
            self.0,
            std::any::type_name::<Src>(),
            std::any::type_name::<Dst>()
        )
    }
}

#[derive(Educe, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[educe(Debug)]
pub struct Vec2<T: fmt::Debug, U: CoordSpace> {
    #[educe(Debug(method("fmt_limited_precision")))]
    pub x: T,
    #[educe(Debug(method("fmt_limited_precision")))]
    pub y: T,
    #[educe(Debug(ignore))]
    _unit: PhantomData<U>,
}

#[inline]
pub fn vec2<T: fmt::Debug, U: CoordSpace>(p1: T, p2: T) -> Vec2<T, U> {
    Vec2 {
        x: p1,
        y: p2,
        _unit: PhantomData,
    }
}

impl<T: fmt::Debug + Num + Copy + NumCast, U: CoordSpace> Vec2<T, U> {
    pub fn clamp(self, min: Vec2<T, U>, max: Vec2<T, U>) -> Vec2<T, U>
    where
        T: PartialOrd,
    {
        Vec2 {
            x: num::clamp(self.x, min.x, max.x),
            y: num::clamp(self.y, min.y, max.y),
            _unit: PhantomData,
        }
    }

    pub fn map<T2: fmt::Debug, F: Fn(T) -> T2>(self, f: F) -> Vec2<T2, U> {
        Vec2 {
            x: f(self.x),
            y: f(self.y),
            _unit: PhantomData,
        }
    }

    /// Casts the values of the vector to another type, e.g. f64 -> i32
    pub fn cast<DstT: fmt::Debug + NumCast>(self) -> Vec2<DstT, U> {
        Vec2 {
            x: DstT::from(self.x).unwrap(),
            y: DstT::from(self.y).unwrap(),
            _unit: PhantomData,
        }
    }

    /// Force transforms one unit to another, this function should be used carefully,
    /// As it does not scale the values, it just changes the unit type.
    pub fn cast_unit<DstU: CoordSpace>(self) -> Vec2<T, DstU> {
        Vec2 {
            x: self.x,
            y: self.y,
            _unit: PhantomData,
        }
    }

    pub fn to_array(self) -> [T; 2] {
        [self.x, self.y]
    }

    pub fn scale<SrcT: Num + Copy + NumCast, Dst: CoordSpace>(
        self,
        scale: Scale<SrcT, U, Dst>,
    ) -> Vec2<T, Dst>
    where
        T: Mul,
    {
        Vec2 {
            x: self.x / T::from(scale.get()).unwrap(),
            y: self.y / T::from(scale.get()).unwrap(),
            _unit: PhantomData,
        }
    }
}

macro_rules! impl_vec2_op {
    ($op_name:ident) => {
        paste! {
            impl<T: fmt::Debug + $op_name<Output = T> + Copy, U: CoordSpace> $op_name for Vec2<T,U> {
                type Output = Vec2<T, U>;
                fn [<$op_name:lower>](self, rhs: Self) -> Self::Output {
                    Vec2 {
                        x: self.x.[<$op_name:lower>](rhs.x),
                        y: self.y.[<$op_name:lower>](rhs.y),
                        _unit: PhantomData,
                    }
                }
            }
            impl<T: fmt::Debug + $op_name<Output = T> + Copy, U: CoordSpace> $op_name<T> for Vec2<T,U> {
                type Output = Vec2<T, U>;
                fn [<$op_name:lower>](self, rhs: T) -> Self::Output {
                    Vec2 {
                        x: self.x.[<$op_name:lower>](rhs),
                        y: self.y.[<$op_name:lower>](rhs),
                        _unit: PhantomData,
                    }
                }
            }
            impl<T: fmt::Debug + [<$op_name Assign>] + Copy, U: CoordSpace> [<$op_name Assign>] for Vec2<T, U> {
                fn [<$op_name:lower _assign>](&mut self, rhs: Vec2<T, U>) {
                    self.x.[<$op_name:lower _assign>](rhs.x);
                    self.y.[<$op_name:lower _assign>](rhs.y);
                }
            }
            impl<T: fmt::Debug + [<$op_name Assign>] + Copy, U: CoordSpace> [<$op_name Assign>]<T> for Vec2<T, U> {
                fn [<$op_name:lower _assign>](&mut self, rhs: T) {
                    self.x.[<$op_name:lower _assign>](rhs);
                    self.y.[<$op_name:lower _assign>](rhs);
                }
            }
        }
    };
}

impl_vec2_op!(Add);
impl_vec2_op!(Sub);
impl_vec2_op!(Mul);
impl_vec2_op!(Div);
// endregion
// region: Shape
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)] // don't match shape, I index into it (app::handle_inputs)
pub enum Shape {
    CircleOutline,
    CircleFill,
    SquareCentered,
}

impl Shape {
    // Provides Offsets relative to be used with a a presumed central point of origin.
    // The lambda captures the offsets, combines with the central point and does stuff with the data (drawing).
    pub fn draw(self, size: i32, mut lambda: impl FnMut(i32, i32)) {
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
        }
    }

    // Bresenham's Line Algorithm
    pub fn draw_line<T: CoordSpace>(
        mut start: Vec2<i32, T>,
        mut end: Vec2<i32, T>,
        mut plot: &mut impl FnMut(i32, i32),
    ) {
        let dx = (end.x - start.x).abs();
        let sx = if start.x < end.x { 1 } else { -1 };
        let dy = -(end.y - start.y).abs();
        let sy = if start.y < end.y { 1 } else { -1 };

        // crazy branchless code
        // let sx = -1 + ((start.x < end.x) as i32 * 2);
        // let sy = -1 + ((start.y < end.y) as i32 * 2);

        let mut error = dx + dy;

        loop {
            plot(start.x, start.y);
            if start.x == end.x && start.y == end.y {
                break;
            }
            let e2 = 2 * error;
            if e2 >= dy {
                error += dy;
                start.x += sx;
            }
            if e2 <= dx {
                error += dx;
                start.y += sy;
            }
        }
    }

    pub fn draw_arrow<T: CoordSpace + Copy>(
        start: Vec2<i32, T>,
        end: Vec2<i32, T>,
        mut plot: impl FnMut(i32, i32),
    ) {
        // Draw arrow body
        Self::draw_line(start, end, &mut plot);

        /*
                                ARROW_RIGHT

                        End

            ARROW_LEFT          Start
        */

        // const SCALE: f64 = 0.1;
        // Self::draw_line(
        //     start,
        //     vec2(
        //         start.x + (end.x as f64 * SCALE) as i32,
        //         start.y - (end.y as f64 * SCALE) as i32,
        //     ),
        //     &mut plot,
        // );
        // Self::draw_line(
        //     start,
        //     vec2(
        //         start.x - (end.x as f64 * SCALE) as i32,
        //         start.y + (end.y as f64 * SCALE) as i32,
        //     ),
        //     &mut plot,
        // );
    }
}
// endregion
// region: Rgba
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
// endregion
// region: SyncCell
// This is a simple wrapper on UnsafeCell for parallelism. (impl Sync)
// UnsafeCell is an unsafe primitive for interior mutability (bypassing borrow checker)
// UnsafeCell provides no thread safety gurantees, I don't care though so I made this wrapper
pub struct SyncCell<T>(UnsafeCell<T>);
unsafe impl<T: Send> Sync for SyncCell<T> {}
impl<T> SyncCell<T> {
    pub const fn new(val: T) -> Self {
        Self(UnsafeCell::new(val))
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.0.get() }
    }

    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for SyncCell<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let item = self.get();
        f.debug_struct("SyncCell").field("Item", item).finish()
    }
}

impl<T: Clone> Clone for SyncCell<T> {
    fn clone(&self) -> Self {
        Self::new(self.get().clone())
    }
}

pub fn fmt_limited_precision<T: fmt::Debug>(x: T, format: &mut fmt::Formatter) -> fmt::Result {
    write!(format, "{x:.2?}") // Specify precision here
}
// endregion
