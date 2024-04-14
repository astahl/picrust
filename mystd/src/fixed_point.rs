pub mod convert;
pub mod ops;

#[derive(Debug)]
pub enum FixedPointConversionError {
    ValueIsNotFinite,
}

macro_rules! fxp_impl {
    ($underlying_type:ident $name:ident) => {
        #[derive(Clone, Copy)]
        #[repr(transparent)]
        pub struct $name <const P: isize>($underlying_type);

        impl<const P: isize> $name<P> {
            pub const PRECISION: isize = P;
            const MASK: $underlying_type = if Self::PRECISION < 0 {
                    0 
                } else if Self::PRECISION as u32 > $underlying_type::BITS {
                    $underlying_type::MAX
                } else {
                    (1 << Self::PRECISION) - 1
                };

            const fn signed_shl(value: $underlying_type, amount: isize) -> $underlying_type {
                let abs = amount.unsigned_abs();
                if abs > $underlying_type::BITS as usize {
                    0
                } else if amount < 0 {
                    value >> abs
                } else {
                    value << abs
                }
            }

            const fn signed_shift(value: $underlying_type, from: isize, to: isize) -> $underlying_type {
                let diff = to - from;
                Self::signed_shl(value, diff)
            }

            pub const fn default() -> Self {
                Self(0)
            }

            pub const fn new(value: $underlying_type) -> Self {
                Self(value)
            }
        
            pub const fn from_int(int: $underlying_type) -> Self {
                Self(Self::signed_shl(int, P))
            }
        
            pub const fn from_int_frac(int: $underlying_type, frac: $underlying_type) -> Self {
                Self(Self::signed_shl(int, P) + frac)
            }
        
            pub const fn from_shifted(value: $underlying_type, precision: isize) -> Self {
                Self::new(Self::signed_shift(value, precision, P))
            }
        
            pub const fn truncating_shift<const R: isize>(&self) -> $name<R> {
                $name::new(Self::signed_shift(self.0, P, R))
            }
        
            pub const fn truncate(&self) -> $underlying_type {
                Self::signed_shift(self.0, P, 0)
            }
        
            pub const fn raw(&self) -> $underlying_type {
                self.0
            }
        
            pub const fn split(&self) -> ($underlying_type, $underlying_type) {
                (self.0 & !Self::MASK, self.0 & Self::MASK)
            }

            pub const fn split_int_frac(&self) -> ($underlying_type, $underlying_type) {
                if P <= 0 {
                    (self.0, 0)
                } else if P.unsigned_abs() > $underlying_type::BITS as usize {
                    (0, self.0)
                } else {
                    let mask = (1 << P as usize) - 1;
                    (Self::signed_shl((self.0 & !mask), -P), self.0 & mask)
                }
            }
        }

        impl<const P: isize> Default for $name<P> {
            fn default() -> Self {
                Self::default()
            }
        }

        impl<const P: isize> core::fmt::Debug for $name<P> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&P)
                    .field(&self.0)
                    .finish()
            }
        }

        impl<const P: isize> core::convert::From<$underlying_type> for $name<P> {
            fn from(value: $underlying_type) -> Self {
                Self::from_int(value)
            }
        }

        impl<const P: isize> core::convert::Into<f64> for $name<P> {
            fn into(self) -> f64 {
                if $underlying_type::MIN == 0 {
                    convert::fixed_u64_to_float64(self.0 as u64, Self::PRECISION)
                } else {
                    convert::fixed_i64_to_float64(self.0 as i64, Self::PRECISION)
                }
            }
        }

        impl<const P: isize> core::convert::Into<f32> for $name<P> {
            fn into(self) -> f32 {
                if $underlying_type::MIN == 0 {
                    convert::fixed_u32_to_float32(self.0 as u32, Self::PRECISION)
                } else {
                    convert::fixed_i32_to_float32(self.0 as i32, Self::PRECISION)
                }
            }
        }

        impl<const P: isize> core::convert::TryFrom<f32> for $name<P> {
            type Error = FixedPointConversionError;

            fn try_from(value: f32) -> Result<Self, Self::Error> {
                let fixed = if $underlying_type::MIN == 0 {
                    convert::float32_to_fixed_u32(value, Self::PRECISION)? as $underlying_type
                } else {
                    convert::float32_to_fixed_i32(value, Self::PRECISION)? as $underlying_type
                };
                Ok(Self::new(fixed))
            }
        }

        
        impl<const P: isize> core::convert::TryFrom<f64> for $name<P> {
            type Error = FixedPointConversionError;

            fn try_from(value: f64) -> Result<Self, Self::Error> {
                let fixed = if $underlying_type::MIN == 0 {
                    convert::float64_to_fixed_u64(value, Self::PRECISION)? as $underlying_type
                } else {
                    convert::float64_to_fixed_i64(value, Self::PRECISION)? as $underlying_type
                };
                Ok(Self::new(fixed))
            }
        }

        impl<const P: isize, const Q: isize> core::ops::Mul<$name<Q>> for $name<P> {
            type Output = ops::Multiply<$underlying_type, P, Q>;
            
            fn mul(self, rhs: $name<Q>) -> Self::Output {
                ops::Multiply(self.0, rhs.0)
            }
        }

        impl<const P: isize> core::ops::Mul<$underlying_type> for $name<P> {
            type Output = $name<P>;
            
            fn mul(self, rhs: $underlying_type) -> Self::Output {
                Self::new(self.0 * rhs)
            }
        }

        impl<const P: isize, const Q: isize> ops::Multiply<$underlying_type, P, Q> {
            pub fn resolve<const R: isize> (&self) -> $name<R> {
                $name::<R>::from_shifted(self.0 * self.1, P + Q)
            }
        }

        impl<const P: isize, const Q: isize> core::ops::Div<$name<Q>> for $name<P> {
            type Output = ops::Divide<$underlying_type, P, Q>;
            
            fn div(self, rhs: $name<Q>) -> Self::Output {
                assert!(rhs.0 != 0, "Can't divide by zero");
                ops::Divide(self.0, rhs.0)
            }
        }

        impl<const P: isize> core::ops::Div<$underlying_type> for $name<P> {
            type Output = $name<P>;
            
            fn div(self, rhs: $underlying_type) -> Self::Output {
                assert!(rhs != 0, "Can't divide by zero");
                Self::new(self.0 / rhs)
            }
        }

        impl<const P: isize, const Q: isize> ops::Divide<$underlying_type, P, Q> {
            pub fn resolve<const R: isize> (&self) -> $name<R> {
                $name::<R>::from_shifted(self.0 / self.1, P - Q)
            }
        }

        impl<const P: isize, const Q: isize> core::ops::AddAssign<$name<Q>> for $name<P> {
            fn add_assign(&mut self, rhs: $name<Q>) {
                self.0 += Self::signed_shift(rhs.0, Q, P);
            }
        }

        impl<const P: isize, const Q: isize> core::ops::SubAssign<$name<Q>> for $name<P> {
            fn sub_assign(&mut self, rhs: $name<Q>) {
                self.0 -= Self::signed_shift(rhs.0, Q, P);
            }
        }
    };
}


fxp_impl!(u8 FxU8);
fxp_impl!(u16 FxU16);
fxp_impl!(u32 FxU32);
fxp_impl!(u64 FxU64);
fxp_impl!(u128 FxU128);
fxp_impl!(i8 FxS8);
fxp_impl!(i16 FxS16);
fxp_impl!(i32 FxS32);
fxp_impl!(i64 FxS64);
fxp_impl!(i128 FxS128);




#[cfg(test)]
mod tests {
    use crate::collections;

    use super::*;

    #[test]
    fn fmt_debug_works() {
        use core::fmt::Write;

        let mut buff = collections::ring::RingArray::<u8, 32>::new();
        write!(buff, "{:?}", FxS32::<10>::new(-1234)).expect("Writing should work");
        assert_eq!("FxS32(10, -1234)", buff.to_str().unwrap());
    }

    #[test]
    fn split_works() {
        let a: FxS32<8> = 3.25.try_into().unwrap();
        assert_eq!((3 << 8, 0b0100_0000), a.split());
    
        let a: FxS32<8> = (-3.5).try_into().unwrap();
        assert_eq!((-4, 0b1000_0000), a.split_int_frac());

        let a: FxS32<8> = (-3.375).try_into().unwrap();
        assert_eq!((-4, 0b1010_0000), a.split_int_frac());

        let f_uart_clk: FxU32<6> = 3_000_000.into();
        let baud_rate = 115200;
        let baud_rate_divisor = f_uart_clk / (16 * baud_rate);
        assert_eq!((1, 40), baud_rate_divisor.split_int_frac());
    }
}
