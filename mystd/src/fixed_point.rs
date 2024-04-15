pub mod convert;
pub mod ops;

#[derive(Debug)]
pub enum FixedPointConversionError {
    ValueIsNotFinite,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct SignedShifter<T>(T);

macro_rules! fxp_impl {
    ($underlying_type:ident $name:ident) => {

        impl SignedShifter<$underlying_type> {
            const fn overflowing_signed_shl(self, amount: i32) -> (bool, $underlying_type) {
                if amount == 0 || self.0 == 0 {
                    return (false, self.0);
                } 
            
                let abs = amount.unsigned_abs();
                if amount < 0 {
                    if abs < self.0.trailing_zeros() {
                        (false, self.0 >> abs)
                    } else {
                        (true, 0)
                    }
                } else {
                    #[allow(unused_comparisons)]
                    if (self.0 < 0 && abs < self.0.leading_ones()) || (abs < self.0.leading_zeros()) {
                        (false, self.0 << abs)
                    } else {
                        (true, 0)
                    }
                }
            }    

            const fn overflowing_shift_from_to(self, from: i32, to: i32) -> (bool, $underlying_type) {
                let diff = to - from;
                self.overflowing_signed_shl(diff)
            }  

            const fn zeroing_signed_shl(self, amount: i32) -> $underlying_type {
                let abs = amount.unsigned_abs();
                if abs > $underlying_type::BITS {
                    0
                } else if amount < 0 {
                    self.0 >> abs
                } else {
                    self.0 << abs
                }
            }   

            const fn zeroing_shift_from_to(self, from: i32, to: i32) -> $underlying_type {
                let diff = to - from;
                self.zeroing_signed_shl(diff)
            }     
        }


        #[derive(Clone, Copy)]
        #[repr(transparent)]
        pub struct $name <const P: i32>(SignedShifter<$underlying_type>);

        impl<const P: i32> $name<P> {
            pub const PRECISION: i32 = P;
            const MASK: $underlying_type = if Self::PRECISION < 0 {
                0 
            } else if Self::PRECISION as u32 > $underlying_type::BITS {
                $underlying_type::MAX
            } else {
                (1 << Self::PRECISION) - 1
            };
            
            pub const fn new(value: $underlying_type) -> Self {
                Self(SignedShifter(value))
            }

            pub const fn default() -> Self {
                Self::new(0)
            }
        
            pub const fn from_int(int: $underlying_type) -> Self {
                Self::new(SignedShifter(int).zeroing_signed_shl(Self::PRECISION))
            }

            pub const fn from_int_checked(int: $underlying_type) -> Option<Self> {
                if let (false, value) = SignedShifter(int).overflowing_signed_shl(Self::PRECISION) {
                    Some(Self::new(value))
                } else {
                    None
                }
            }
        
            pub const fn from_shifted(value: $underlying_type, source_precision: i32) -> Self {
                Self::new(SignedShifter(value).zeroing_shift_from_to(source_precision, Self::PRECISION))
            }

            pub const fn from_shifted_checked(value: $underlying_type, source_precision: i32) -> Option<Self> {
                if let (false, value) = SignedShifter(value).overflowing_shift_from_to(source_precision, Self::PRECISION) {
                    Some(Self::new(value))
                } else {
                    None
                }
            }
        
            pub const fn truncating_shift<const R: i32>(&self) -> $name<R> {
                $name::from_shifted(self.raw(), P)
            }
        
            pub const fn truncate(&self) -> $underlying_type {
                self.0.zeroing_shift_from_to(P, 0)
            }
        
            pub const fn raw(&self) -> $underlying_type {
                self.0.0
            }
        
            pub const fn split(&self) -> ($underlying_type, $underlying_type) {
                (self.raw() & !Self::MASK, self.raw() & Self::MASK)
            }

            pub const fn split_int_frac(&self) -> ($underlying_type, $underlying_type) {
                if P <= 0 {
                    (self.raw(), 0)
                } else if P.unsigned_abs() > $underlying_type::BITS {
                    (0, self.raw())
                } else {
                    
                    (self.0.zeroing_signed_shl(-P), self.raw() & Self::MASK)
                }
            }
        }

        impl<const P: i32> Default for $name<P> {
            fn default() -> Self {
                Self::default()
            }
        }

        impl<const P: i32> core::fmt::Debug for $name<P> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&P)
                    .field(&self.raw())
                    .finish()
            }
        }

        impl<const P: i32> core::convert::From<$underlying_type> for $name<P> {
            fn from(value: $underlying_type) -> Self {
                Self::from_int(value)
            }
        }

        impl<const P: i32> core::convert::Into<f64> for $name<P> {
            fn into(self) -> f64 {
                if $underlying_type::MIN == 0 {
                    convert::fixed_u64_to_float64(self.raw() as u64, Self::PRECISION)
                } else {
                    convert::fixed_i64_to_float64(self.raw() as i64, Self::PRECISION)
                }
            }
        }

        impl<const P: i32> core::convert::Into<f32> for $name<P> {
            fn into(self) -> f32 {
                if $underlying_type::MIN == 0 {
                    convert::fixed_u32_to_float32(self.raw() as u32, Self::PRECISION)
                } else {
                    convert::fixed_i32_to_float32(self.raw() as i32, Self::PRECISION)
                }
            }
        }

        impl<const P: i32> core::convert::TryFrom<f32> for $name<P> {
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

        
        impl<const P: i32> core::convert::TryFrom<f64> for $name<P> {
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

        impl<const P: i32, const Q: i32> core::ops::Mul<$name<Q>> for $name<P> {
            type Output = ops::Multiply<$underlying_type, P, Q>;
            
            fn mul(self, rhs: $name<Q>) -> Self::Output {
                ops::Multiply(self.raw(), rhs.raw())
            }
        }

        impl<const P: i32> core::ops::Mul<$underlying_type> for $name<P> {
            type Output = $name<P>;
            
            fn mul(self, rhs: $underlying_type) -> Self::Output {
                Self::new(self.raw() * rhs)
            }
        }

        impl<const P: i32, const Q: i32> ops::Multiply<$underlying_type, P, Q> {
            pub fn resolve<const R: i32> (&self) -> $name<R> {
                $name::<R>::from_shifted(self.0 * self.1, P + Q)
            }
        }

        impl<const P: i32, const Q: i32> core::ops::Div<$name<Q>> for $name<P> {
            type Output = ops::Divide<$underlying_type, P, Q>;
            
            fn div(self, rhs: $name<Q>) -> Self::Output {
                assert!(rhs.raw() != 0, "Can't divide by zero");
                ops::Divide(self.raw(), rhs.raw())
            }
        }

        impl<const P: i32> core::ops::Div<$underlying_type> for $name<P> {
            type Output = $name<P>;
            
            fn div(self, rhs: $underlying_type) -> Self::Output {
                assert!(rhs != 0, "Can't divide by zero");
                Self::new(self.raw() / rhs)
            }
        }

        impl<const P: i32, const Q: i32> ops::Divide<$underlying_type, P, Q> {
            pub fn resolve<const R: i32> (&self) -> $name<R> {
                $name::<R>::from_shifted(self.0 / self.1, P - Q)
            }
        }

        impl<const P: i32, const Q: i32> core::ops::AddAssign<$name<Q>> for $name<P> {
            fn add_assign(&mut self, rhs: $name<Q>) {
                self.0.0 += rhs.0.zeroing_shift_from_to(Q, P);
            }
        }

        impl<const P: i32, const Q: i32> core::ops::SubAssign<$name<Q>> for $name<P> {
            fn sub_assign(&mut self, rhs: $name<Q>) {
                self.0.0 -= rhs.0.zeroing_shift_from_to(Q, P);
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
