pub mod vectors {
    use crate::util::{as_u8_slice, as_u8_slice_from_slice, AsBytes};
    use serde::{Deserialize, Serialize};
    use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

    pub trait Zero {
        const ZERO: Self;
    }

    pub trait Identity {
        const IDENTITY: Self;
    }

    pub trait Sqrt {
        fn sqrt(self) -> Self;
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

    macro_rules! create_vec {
        ($name: ident, $($fields: ident)+) => {
            #[repr(C)]
            #[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
            pub struct $name<T = f32> {
                $(pub $fields: T),+
            }

            impl<T> $name<T> {
                #[inline(always)]
                pub fn new($($fields: T,)+) -> Self {
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
                pub fn dot(self, rhs: Self) -> T {
                    (self * rhs).sum()
                }
            }

            impl<T: Add<Output = T> + Zero + Mul<Output = T> + Sqrt + Copy> $name<T> {
                #[inline(always)]
                pub fn len(self) -> T {
                    self.dot(self).sqrt()
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

    sqrt!(float, f32 f64);
    sqrt!(int, i32 i64 u32 u64);

    number!(i32, 0, 1);
    number!(i64, 0, 1);
    number!(u32, 0, 1);
    number!(u64, 0, 1);
    number!(f32, 0.0, 1.0);
    number!(f64, 0.0, 1.0);

    create_vec!(Vec2, x y);
    create_vec!(Vec3, x y z);
    create_vec!(Vec4, x y z w);
}
