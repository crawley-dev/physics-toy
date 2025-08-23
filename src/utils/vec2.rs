use educe::Educe;
use num::{Num, NumCast};
use paste::paste;
use std::{
    fmt,
    marker::PhantomData,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

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
pub struct Scale<T: Num + Copy + std::ops::Mul, Src: CoordSpace, Dst: CoordSpace>(
    T,
    PhantomData<(Src, Dst)>,
);
impl<T: Num + Copy + std::ops::Mul, Src: CoordSpace, Dst: CoordSpace> Scale<T, Src, Dst> {
    pub fn new(val: T) -> Self {
        Self(val, PhantomData)
    }

    pub fn get(&self) -> T {
        self.0
    }
}
impl<T: fmt::Display + Num + Copy + std::ops::Mul, Src: CoordSpace, Dst: CoordSpace> fmt::Debug
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

    pub fn cross_product<V: fmt::Debug + Num + Copy + NumCast>(self, other: Vec2<V, U>) -> T
    where
        T: Mul<V, Output = T>,
    {
        self.x * other.y - self.y * other.x
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

impl<T, U> Neg for Vec2<T, U>
where
    T: fmt::Debug + Neg<Output = T>,
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

pub fn fmt_limited_precision<T: fmt::Debug>(x: T, format: &mut fmt::Formatter) -> fmt::Result {
    write!(format, "{x:.2?}") // Specify precision here
}
