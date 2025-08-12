pub trait Zero {
    const ZERO: Self;
}

pub trait Identity {
    const IDENTITY: Self;
}

pub trait Sqrt {
    fn sqrt(self) -> Self;
}

pub trait Atan2 {
    fn atan2(self, other: Self) -> Self;
}

macro_rules! number {
    ($type: ty, $zero: expr, $identity: expr) => {
        impl Zero for $type {
            const ZERO: Self = $zero;
        }

        impl Identity for $type {
            const IDENTITY: Self = $identity;
        }
    };
}

macro_rules! sqrt {
    (float, $($type: ty)+) => {
        $(
            impl Sqrt for $type {
                fn sqrt(self) -> Self {
                    self.sqrt()
                }
            }
        )+
    };
    (int, $($type: ty)+) => {
        $(
            impl Sqrt for $type {
                fn sqrt(self) -> Self {
                    self.isqrt()
                }
            }
        )+
    };
}

macro_rules! atan2 {
    ($($type: ty)+) => {
        $(
            impl Atan2 for $type {
                fn atan2(self, other: Self) -> Self {
                    self.atan2(other)
                }
            }
        )+
    };
}

sqrt!(float, f32 f64);
sqrt!(int, i32 i64 u32 u64);

number!(i32, 0, 1);
number!(i64, 0, 1);
number!(u32, 0, 1);
number!(u64, 0, 1);
number!(f32, 0.0, 1.0);
number!(f64, 0.0, 1.0);

atan2!(f32 f64);

pub mod vector {
    use super::{Identity, Sqrt, Zero};
    use crate::util::{as_u8_slice, as_u8_slice_from_slice, AsBytes};
    use serde::{Deserialize, Serialize};
    use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

    macro_rules! create_vec {
        ($name: ident, $($fields: ident)+) => {
            #[repr(C)]
            #[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
            pub struct $name<T = f32> {
                $(pub $fields: T),+
            }

            impl<T> $name<T> {
                #[inline(always)]
                pub const fn new($($fields: T,)+) -> Self {
                    Self {
                        $($fields,)+
                    }
                }
            }

            impl<T: Add<Output = T> + Zero> $name<T> {
                #[inline(always)]
                pub fn sum(self) -> T {
                    ( $(self.$fields + )+ T::ZERO )
                }
            }

            impl<T: Mul<Output = T> + Add<Output = T> + Zero> $name<T> {
                #[inline(always)]
                pub fn dot(self, other: Self) -> T {
                    (self * other).sum()
                }
            }

            impl<T: Add<Output = T> + Zero + Mul<Output = T> + Sqrt + Copy> $name<T> {
                #[inline(always)]
                pub fn len(self) -> T {
                    self.dot(self).sqrt()
                }
            }

            impl<T: Add<Output = T> + Div<Output = T> + Zero + Mul<Output = T> + Copy> $name<T> {
                #[inline(always)]
                pub fn proj(self, to: Self) -> Self {
                    to * (self.dot(to) / to.dot(to))
                }
            }

            impl<T: Add<Output = T> + Div<Output = T> + Mul<Output = T> + Zero + Sqrt + Eq + Copy> $name<T> {
                #[inline(always)]
                pub fn normalized(self) -> Self {
                    let len = self.len();

                    if len == T::ZERO {
                        self / len
                    } else {
                        Self::ZERO
                    }

                }
            }

            impl<T: Add<Output = T> + Div<Output = T> + Mul<Output = T> + Zero + Sqrt + PartialOrd + Copy> $name<T> {
                #[inline(always)]
                pub fn limit_length(self, limit: T) -> Self {
                    let len = self.len();

                    if len > limit {
                        self / self.len()
                    } else {
                        self
                    }
                }
            }

            impl<'a, T> AsBytes<'a> for $name<T> {
                #[inline(always)]
                fn as_bytes(&'a self) -> &'a [u8] {
                    unsafe { as_u8_slice(self) }
                }
            }

            impl<'a, T> AsBytes<'a> for [$name<T>] {
                #[inline(always)]
                fn as_bytes(&'a self) -> &'a [u8] {
                    unsafe { as_u8_slice_from_slice(self) }
                }
            }

            impl<T: Zero> Zero for $name<T> {
                const ZERO: Self = Self {
                  $($fields: T::ZERO,)+
                };
            }

            impl<T: Identity> Identity for $name<T> {
                const IDENTITY: Self = Self {
                  $($fields: T::IDENTITY,)+
                };
            }

            impl<T: Add<Output = T>> Add for $name<T> {
                type Output = Self;

                #[inline(always)]
                fn add(self, rhs: Self) -> Self::Output {
                    Self {
                      $($fields: self.$fields + rhs.$fields,)+
                    }
                }
            }

            impl<T: Add<Output = T> + Copy> AddAssign for $name<T> {
                #[inline(always)]
                fn add_assign(&mut self, rhs: Self) {
                      $(self.$fields = self.$fields + rhs.$fields;)+
                }
            }

            impl<T: Div<Output = T>>  Div for $name<T> {
                type Output = Self;

                #[inline(always)]
                fn div(self, rhs: Self) -> Self::Output {
                    Self {
                      $($fields: self.$fields / rhs.$fields,)+
                    }
                }
            }

            impl<T: Div<Output = T> + Copy> DivAssign for $name<T> {
                #[inline(always)]
                fn div_assign(&mut self, rhs: Self) {
                      $(self.$fields = self.$fields / rhs.$fields;)+
                }
            }

            impl<T: Div<Output = T> + Copy>  Div<T> for $name<T> {
                type Output = Self;

                #[inline(always)]
                fn div(self, rhs: T) -> Self::Output {
                    Self {
                      $($fields: self.$fields / rhs,)+
                    }
                }
            }

            impl<T: Div<Output = T> + Copy> DivAssign<T> for $name<T> {
                #[inline(always)]
                fn div_assign(&mut self, rhs: T) {
                      $(self.$fields = self.$fields / rhs;)+
                }
            }

            impl<T: Mul<Output = T>>  Mul for $name<T> {
                type Output = Self;

                #[inline(always)]
                fn mul(self, rhs: Self) -> Self::Output {
                    Self {
                      $($fields: self.$fields * rhs.$fields,)+
                    }
                }
            }


            impl<T: Mul<Output = T> + Copy> MulAssign for $name<T> {
                #[inline(always)]
                fn mul_assign(&mut self, rhs: Self) {
                      $(self.$fields = self.$fields * rhs.$fields;)+
                }
            }

            impl<T: Mul<Output = T> + Copy>  Mul<T> for $name<T> {
                type Output = Self;

                #[inline(always)]
                fn mul(self, rhs: T) -> Self::Output {
                    Self {
                      $($fields: self.$fields * rhs,)+
                    }
                }
            }

            impl<T: Mul<Output = T> + Copy> MulAssign<T> for $name<T> {
                #[inline(always)]
                fn mul_assign(&mut self, rhs: T) {
                      $(self.$fields = self.$fields * rhs;)+
                }
            }

            impl<T: Neg<Output = T>> Neg for $name<T> {
                type Output = Self;

                #[inline(always)]
                fn neg(self) -> Self {
                    Self {
                      $($fields: -self.$fields,)+
                    }
                }

            }

            impl<T: Sub<Output = T>> Sub for $name<T> {
                type Output = Self;

                #[inline(always)]
                fn sub(self, rhs: Self) -> Self::Output {
                    Self {
                      $($fields: self.$fields - rhs.$fields,)+
                    }
                }
            }

            impl<T: Sub<Output = T> + Copy> SubAssign for $name<T> {
                #[inline(always)]
                fn sub_assign(&mut self, rhs: Self) {
                      $(self.$fields = self.$fields - rhs.$fields;)+
                }
            }
        };
    }

    impl<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy> Vec3<T> {
        #[inline(always)]
        pub fn cross(self, rhs: Self) -> Self {
            Self {
                x: self.y * rhs.z - self.z * rhs.y,
                y: self.z * rhs.x - self.x * rhs.z,
                z: self.x * rhs.y - self.y * rhs.x,
            }
        }
    }

    create_vec!(Vec2, x y);
    create_vec!(Vec3, x y z);
    create_vec!(Vec4, x y z w);
}

mod matrix {
    use super::vector::{Vec2, Vec3};
    use super::{vector::Vec4, Identity, Zero};
    use std::ops::{Add, Mul};

    macro_rules! create_matrix {
        ($name: ident, $vec: ident, $($field: ident)+) => {
            struct $name<T> {
                $(pub $field: $vec<T>,)+
            }

            impl<T: Zero> Zero for $name<T> {
                const ZERO: Self = Self {
                    $($field: $vec::ZERO,)+
                };
            }

            impl<T: Add<Output = T> + Mul<Output = T> + Copy + Zero> Mul<$vec<T>> for $name<T> {
                type Output = $vec<T>;

                #[inline(always)]
                fn mul(self, rhs: $vec<T>) -> Self::Output {
                    Self::Output {
                        $($field: rhs.dot(self.$field),)+
                    }
                }
            }
        };
    }

    create_matrix!(Mat2, Vec2, x y);
    create_matrix!(Mat3, Vec3, x y z);
    create_matrix!(Mat4, Vec4, x y z w);

    impl<T: Identity + Zero> Identity for Mat2<T> {
        const IDENTITY: Self = Self {
            x: Vec2::new(T::IDENTITY, T::ZERO),
            y: Vec2::new(T::ZERO, T::IDENTITY),
        };
    }

    impl<T: Identity + Zero> Identity for Mat3<T> {
        const IDENTITY: Self = Self {
            x: Vec3::new(T::IDENTITY, T::ZERO, T::ZERO),
            y: Vec3::new(T::ZERO, T::IDENTITY, T::ZERO),
            z: Vec3::new(T::ZERO, T::ZERO, T::IDENTITY),
        };
    }

    impl<T: Identity + Zero> Identity for Mat4<T> {
        const IDENTITY: Self = Self {
            x: Vec4::new(T::IDENTITY, T::ZERO, T::ZERO, T::ZERO),
            y: Vec4::new(T::ZERO, T::IDENTITY, T::ZERO, T::ZERO),
            z: Vec4::new(T::ZERO, T::ZERO, T::IDENTITY, T::ZERO),
            w: Vec4::new(T::ZERO, T::ZERO, T::ZERO, T::IDENTITY),
        };
    }
}

mod quaternion {
    use super::{vector::Vec3, Atan2, Identity, Sqrt, Zero};
    use std::ops::{Add, Div, Mul};

    pub struct Quat<T>(Vec3<T>, T);

    impl<
            T: Add<Output = T>
                + Mul<Output = T>
                + Div<Output = T>
                + Identity
                + Zero
                + Sqrt
                + Atan2
                + Eq
                + Copy,
        > Quat<T>
    {
        #[inline(always)]
        fn to_axis_angle(self) -> Vec3<T> {
            let len = self.0.len();

            if len == T::ZERO {
                Vec3::ZERO
            } else {
                (self.0 / len) * (T::IDENTITY + T::IDENTITY) * len.atan2(self.1)
            }
        }
    }

    // struct Quat<T = f32> {
    //     a: T,
    //     b: T,
    //     c: T,
    //     d: T,
    // }
}

mod sdf {
    use super::vector::{Vec2, Vec3};

    impl Vec2 {
        pub fn rect(self, pos: Vec2, scale: Vec2) -> f32 {
            let d = Vec2::new((self.x - pos.x).abs(), (self.y - pos.y).abs()) - scale;

            (Vec2::new(d.x.max(0.0), d.y.max(0.0))).len() + d.x.max(d.y).min(0.0)
        }

        pub fn polygon(self, points: Vec<Vec2>) -> f32 {
            let d = self - points[0];
            let mut d = d.dot(d);
            let mut s = 1.0;
            let mut j = points.len() - 1;

            for i in 0..points.len() {
                let e = points[j] - points[i];
                let w = self - points[i];
                let b = w - w.proj(e).limit_length(1.0);
                d = d.min(b.dot(b));
                let c = Vec3::new(
                    self.y >= points[i].y,
                    self.y < points[j].y,
                    e.x * w.y > e.y * w.x,
                );

                if (c.x && c.y && c.z) || (!c.x && !c.y && !c.z) {
                    s *= -1.0;
                }
            }

            return s * d.sqrt();
        }
    }
}

fn scratch() {
    use quaternion::Quat;
    use vector::Vec4;

    let v = Vec4::IDENTITY;
    v.limit_length(0.5);
}
