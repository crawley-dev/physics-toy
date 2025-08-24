use educe::Educe;
use num::{Float, Num, NumCast, Signed};
use paste::paste;
use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};
use wgpu::Texture;

pub trait CoordSpace {}
macro_rules! create_coordinate_space {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name;
        impl CoordSpace for $name {}
    };
}

create_coordinate_space!(WindowSpace); // Space of the window e.g. 720x480
create_coordinate_space!(TextureSpace);
create_coordinate_space!(CentredTextureSpace); // Texture space situated around the centre of the screen, i.e. 0,0 is the screen's centre.
create_coordinate_space!(WorldSpace); // Space of the world, any number

#[derive(Educe, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[educe(Debug)]
pub struct Vec2<T: Debug, U: CoordSpace> {
    #[educe(Debug(method("fmt_limited_precision")))]
    pub x: T,
    #[educe(Debug(method("fmt_limited_precision")))]
    pub y: T,
    #[educe(Debug(ignore))]
    _unit: PhantomData<U>,
}

#[inline]
pub fn vec2<T: Debug, U: CoordSpace>(p1: T, p2: T) -> Vec2<T, U> {
    Vec2 {
        x: p1,
        y: p2,
        _unit: PhantomData,
    }
}

impl<T: Debug + Num + Copy + NumCast, U: CoordSpace> Vec2<T, U> {
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

    pub fn map<T2: Debug, F: Fn(T) -> T2>(self, f: F) -> Vec2<T2, U> {
        Vec2 {
            x: f(self.x),
            y: f(self.y),
            _unit: PhantomData,
        }
    }

    /// Casts the values of the vector to another type, e.g. f64 -> i32
    pub fn cast<DstT: Debug + NumCast>(self) -> Vec2<DstT, U> {
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
}

// region: Vec2 Math Operations
impl<T: Debug + Signed + Copy + NumCast, U: CoordSpace> Vec2<T, U> {
    pub fn perpendicular(&self) -> Self {
        Vec2 {
            x: -self.y,
            y: self.x,
            _unit: PhantomData,
        }
    }

    pub fn cross_product<V: Debug + Num + Copy + NumCast>(self, other: Vec2<V, U>) -> T
    where
        T: Mul<V, Output = T>,
    {
        self.x * other.y - self.y * other.x
    }

    pub fn dot_product<V: Debug + Num + Copy + NumCast>(self, other: Vec2<V, U>) -> T
    where
        T: Mul<V, Output = T>,
    {
        self.x * other.x + self.y * other.y
    }

    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }

    pub fn length(&self) -> T
    where
        T: Float,
    {
        self.length_squared().sqrt()
    }

    pub fn normalise(&self) -> Self
    where
        T: Float,
    {
        let length = self.length();
        if length > T::zero() {
            Vec2 {
                x: self.x / length,
                y: self.y / length,
                _unit: PhantomData,
            }
        } else {
            Vec2 {
                x: T::zero(),
                y: T::zero(),
                _unit: PhantomData,
            }
        }
    }
}

// region: Vec2 CoordSpace translations
impl<T: Debug + Num + Copy + NumCast> Vec2<T, WindowSpace> {
    pub fn to_texture_space<X: num::ToPrimitive + Copy>(
        self,
        texture_scale: X,
    ) -> Vec2<T, TextureSpace> {
        Vec2 {
            x: self.x / T::from(texture_scale).unwrap(),
            y: self.y / T::from(texture_scale).unwrap(),
            _unit: PhantomData,
        }
    }

    pub fn to_world_space<X: num::ToPrimitive + Copy, T2: Debug + Num + Copy + NumCast>(
        self,
        texture_scale: X,
        camera: Vec2<T2, WorldSpace>,
    ) -> Vec2<T, WorldSpace> {
        let scale = T::from(texture_scale).unwrap();
        Vec2 {
            x: self.x / scale + T::from(camera.x).unwrap(),
            y: self.y / scale - T::from(camera.y).unwrap(),
            _unit: PhantomData,
        }
    }
}

impl<T: Debug + Num + Copy + NumCast> Vec2<T, TextureSpace> {
    pub fn to_world_space<T2: Debug + Num + Copy + NumCast>(
        self,
        camera: Vec2<T2, WorldSpace>,
    ) -> Vec2<T, WorldSpace> {
        Vec2 {
            x: self.x + T::from(camera.x).unwrap(),
            y: self.y - T::from(camera.y).unwrap(),
            _unit: PhantomData,
        }
    }

    pub fn to_centred_texture<T2: Debug + Num + Copy + NumCast>(
        self,
        viewport_centre: Vec2<T2, CentredTextureSpace>,
    ) -> Vec2<T, CentredTextureSpace> {
        Vec2 {
            x: T::from(viewport_centre.x).unwrap() + self.x,
            y: T::from(viewport_centre.y).unwrap() - self.y,
            _unit: PhantomData,
        }
    }
}

impl<T: Debug + Num + Copy + NumCast> Vec2<T, WorldSpace> {
    pub fn to_texture_space<T2: Debug + Num + Copy + NumCast>(
        self,
        camera: Vec2<T2, WorldSpace>,
    ) -> Vec2<T, TextureSpace> {
        Vec2 {
            x: self.x - T::from(camera.x).unwrap(),
            y: self.y + T::from(camera.y).unwrap(),
            _unit: PhantomData,
        }
    }
}
// endregion

// region: Vec2 Operators
impl<T, U> Neg for Vec2<T, U>
where
    T: Debug + Neg<Output = T>,
    U: CoordSpace,
{
    type Output = Self;
    fn neg(self) -> Self {
        Vec2 {
            x: -self.x,
            y: -self.y,
            _unit: PhantomData,
        }
    }
}

macro_rules! impl_vec2_op {
    ($op_name:ident) => {
        paste! {
            impl<T: Debug + $op_name<Output = T> + Copy, U: CoordSpace> $op_name for Vec2<T,U> {
                type Output = Vec2<T, U>;
                fn [<$op_name:lower>](self, rhs: Self) -> Self::Output {
                    Vec2 {
                        x: self.x.[<$op_name:lower>](rhs.x),
                        y: self.y.[<$op_name:lower>](rhs.y),
                        _unit: PhantomData,
                    }
                }
            }
            impl<T: Debug + $op_name<Output = T> + Copy, U: CoordSpace> $op_name<T> for Vec2<T,U> {
                type Output = Vec2<T, U>;
                fn [<$op_name:lower>](self, rhs: T) -> Self::Output {
                    Vec2 {
                        x: self.x.[<$op_name:lower>](rhs),
                        y: self.y.[<$op_name:lower>](rhs),
                        _unit: PhantomData,
                    }
                }
            }
            impl<T: Debug + [<$op_name Assign>] + Copy, U: CoordSpace> [<$op_name Assign>] for Vec2<T, U> {
                fn [<$op_name:lower _assign>](&mut self, rhs: Vec2<T, U>) {
                    self.x.[<$op_name:lower _assign>](rhs.x);
                    self.y.[<$op_name:lower _assign>](rhs.y);
                }
            }
            impl<T: Debug + [<$op_name Assign>] + Copy, U: CoordSpace> [<$op_name Assign>]<T> for Vec2<T, U> {
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

pub fn fmt_limited_precision<T: Debug>(x: T, format: &mut Formatter) -> std::fmt::Result {
    write!(format, "{x:.2?}") // Specify precision here
}
