use super::{FixedPoint, FixedPointContainer};

mod consts {
    pub mod f32 {
        pub const EXPONENT_BIAS: isize = 127;
        pub const MAX_EXPONENT: isize = 127;
        pub const MIN_EXPONENT: isize = -126;
        pub const MANTISSA_WIDTH: isize = 23;
        pub const DENORM_EXPONENT: isize = MIN_EXPONENT - MANTISSA_WIDTH;
        pub const MANTISSA_MASK: u32 = (1 << MANTISSA_WIDTH) - 1;
        pub const EXPONENT_MASK: u32 = (1 << 8) - 1;
        pub const SIGN_MASK: u32 = 1 << 31;
    }
    pub mod f64 {
        pub const EXPONENT_BIAS: isize = 1023;
        pub const MAX_EXPONENT: isize = 1023;
        pub const MIN_EXPONENT: isize = -1022;
        pub const MANTISSA_WIDTH: isize = 52;
        pub const DENORM_EXPONENT: isize = MIN_EXPONENT - MANTISSA_WIDTH;
        pub const MANTISSA_MASK: u64 = (1 << MANTISSA_WIDTH) - 1;
        pub const EXPONENT_MASK: u64 = (1 << 11) - 1;
        pub const SIGN_MASK: u64 = 1 << 63;
    }
}

#[derive(Debug)]
pub enum FixedPointConversionError {
    ValueIsNotFinite
}

impl<T: FixedPointContainer<T>> core::convert::From<T> for FixedPoint<0, T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

fn fixed_u32_to_float32(value: u32, precision: isize) -> f32 {
    use consts::f32::*;    

    if value == 0 {
        return 0.0;
    }
    
    // take the number of leading zeros, so we can shift the mantissa so the first 1 is at lsb 24 (the mantissa in f32 is 23 bits long)
    let most_significant_one = 31 - value.leading_zeros() as isize;
    let exponent = most_significant_one - precision;
    
    if exponent > MAX_EXPONENT {
        f32::INFINITY
    } else if exponent >= MIN_EXPONENT {
        let mantissa = value.signed_shift(most_significant_one, MANTISSA_WIDTH);
        f32::from_bits(((exponent + EXPONENT_BIAS) as u32 & EXPONENT_MASK) << MANTISSA_WIDTH | (mantissa & MANTISSA_MASK))
    } else if exponent >= DENORM_EXPONENT {
        let mantissa = value.signed_shift(precision, -DENORM_EXPONENT);
        // handle denormalization
        f32::from_bits(mantissa & MANTISSA_MASK)
    } else {
        0.0
    }
} 

fn fixed_i32_to_float32(value: i32, precision: isize) -> f32 {
    if value < 0 {
        -fixed_u32_to_float32(value.unsigned_abs(), precision)
    } else {
        fixed_u32_to_float32(value as u32, precision)
    }
}

fn fixed_u64_to_float64(value: u64, precision: isize) -> f64 {
    use consts::f64::*;

    if value == 0 {
        return 0.0;
    }
    
    // take the number of leading zeros, so we can shift the mantissa so the first 1 is at lsb 53 (the mantissa in f32 is 52 bits long)
    let most_significant_one = 63 - value.leading_zeros() as isize;
    let exponent = most_significant_one - precision;
    
    if exponent > MAX_EXPONENT {
        f64::INFINITY
    } else if exponent >= MIN_EXPONENT {
        let mantissa = value.signed_shift(most_significant_one, MANTISSA_WIDTH);
        f64::from_bits(((exponent + EXPONENT_BIAS) as u64 & EXPONENT_MASK) << MANTISSA_WIDTH | (mantissa & MANTISSA_MASK))
    } else if exponent >= DENORM_EXPONENT {
        let mantissa = value.signed_shift(precision, -DENORM_EXPONENT);
        // handle denormalization
        f64::from_bits(mantissa & MANTISSA_MASK)
    } else {
        0.0
    }
} 

fn fixed_i64_to_float64(value: i64, precision: isize) -> f64 {
    if value < 0 {
        -fixed_u64_to_float64(value.unsigned_abs(), precision)
    } else {
        fixed_u64_to_float64(value as u64, precision)
    }
}

impl<const P: isize, T: FixedPointContainer<T> + Into<u32>> FixedPoint<P, T> {
    pub fn to_f32_unsigned(&self) -> f32 {
        fixed_u32_to_float32(self.0.into(), P)
    }
}

impl<const P: isize, T: FixedPointContainer<T> + Into<u64>> FixedPoint<P, T> {
    pub fn to_f64_unsigned(&self) -> f64 {
        fixed_u64_to_float64(self.0.into(), P)
    }
}

impl<const P: isize, T: FixedPointContainer<T> + Into<i32>> FixedPoint<P, T> {
    pub fn to_f32_signed(&self) -> f32 {
        fixed_i32_to_float32(self.0.into(), P)
    }
}

impl<const P: isize, T: FixedPointContainer<T> + Into<i64>> FixedPoint<P, T> {
    pub fn to_f64_signed(&self) -> f64 {
        fixed_i64_to_float64(self.0.into(), P)
    }
}

impl<const P: isize, T: FixedPointContainer<T> + From<i32>> core::convert::TryFrom<f32> for FixedPoint<P, T> {
    type Error = FixedPointConversionError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        use consts::f32::*;
        if !value.is_finite() {
            return Err(FixedPointConversionError::ValueIsNotFinite);
        }
        let bits = value.to_bits();
        if bits & !SIGN_MASK == 0 {
            return Ok(Self::default())
        }
        let biased_exponent = EXPONENT_MASK & (bits >> MANTISSA_WIDTH);
        
        let mut mantissa = MANTISSA_MASK & bits;
        let precision = if biased_exponent == 0 {
            // denormal value
            -DENORM_EXPONENT
        } else {
            // normal value, add the implicit leading 1
            mantissa |= 1 << MANTISSA_WIDTH;
            MANTISSA_WIDTH + EXPONENT_BIAS - biased_exponent as isize
        };

        let signed_value: i32 = if bits & SIGN_MASK != 0 {
            -(mantissa as i32)
        } else {
            mantissa as i32
        };

        Ok(Self::from_shifted(signed_value.into(), precision))
    }
}


impl<const P: isize, T: FixedPointContainer<T> + From<i64>> core::convert::TryFrom<f64> for FixedPoint<P, T> {
    type Error = FixedPointConversionError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        use consts::f64::*;
        if !value.is_finite() {
            return Err(FixedPointConversionError::ValueIsNotFinite);
        }
        let bits = value.to_bits();
        if bits & !SIGN_MASK == 0 {
            return Ok(Self::default())
        }
        let biased_exponent = EXPONENT_MASK & (bits >> MANTISSA_WIDTH);
        
        let mut mantissa = MANTISSA_MASK & bits;
        let precision = if biased_exponent == 0 {
            // denormal value
            -DENORM_EXPONENT
        } else {
            // normal value, add the implicit leading 1
            mantissa |= 1 << MANTISSA_WIDTH;
            MANTISSA_WIDTH + EXPONENT_BIAS - biased_exponent as isize
        };

        let signed_value = if bits & SIGN_MASK != 0 {
            -(mantissa as i64)
        } else {
            mantissa as i64
        };

        Ok(Self::from_shifted(signed_value.into(), precision))
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn to_f32_works() {
        assert_eq!(0.0, FixedPoint::<1000, u32>::new(0).to_f32_unsigned());
        assert_eq!(0.0, FixedPoint::<-1000, u32>::new(0).to_f32_unsigned());
        assert_eq!(0.0, FixedPoint::<0, u32>::new(0).to_f32_unsigned());

        assert_eq!(1.0, FixedPoint::<0, u32>::new(1).to_f32_unsigned());
        assert_eq!(0.5, FixedPoint::<1, u32>::new(1).to_f32_unsigned());
        assert_eq!(1.5, FixedPoint::<1, u32>::new(3).to_f32_unsigned());
        assert_eq!(1.0, FixedPoint::<10, u32>::new(1024).to_f32_unsigned());
        assert_eq!(1.25, FixedPoint::<10, u32>::new(1024 + 256).to_f32_unsigned());

        assert!(FixedPoint::<126, u32>::new(1).to_f32_unsigned().is_normal());
        assert!(FixedPoint::<-127, u32>::new(1).to_f32_unsigned().is_normal());

        assert!(FixedPoint::<149, u32>::new(1).to_f32_unsigned().is_subnormal());
        assert_eq!(f32::from_bits(0b1), FixedPoint::<149, u32>::new(1).to_f32_unsigned());
        assert_eq!(f32::from_bits(0b10), FixedPoint::<148, u32>::new(1).to_f32_unsigned());
        assert_eq!(f32::from_bits(0b1), FixedPoint::<150, u32>::new(2).to_f32_unsigned());

        assert_eq!(f32::INFINITY, FixedPoint::<-128, u32>::new(1).to_f32_unsigned());
        assert_eq!(0.0, FixedPoint::<150, u32>::new(1).to_f32_unsigned());

        assert_eq!(-1.0, FixedPoint::<0, i32>::new(-1).to_f32_signed());
        assert_eq!(-0.25, FixedPoint::<2, i32>::new(-1).to_f32_signed());
    }

    #[test]
    fn to_f64_works() {

        assert_eq!(0.0, FixedPoint::<2000, u64>::new(0).to_f64_unsigned());
        assert_eq!(0.0, FixedPoint::<-2000, u64>::new(0).to_f64_unsigned());
        assert_eq!(0.0, FixedPoint::<0, u64>::new(0).to_f64_unsigned());

        assert_eq!(1.0, FixedPoint::<0, u64>::new(1).to_f64_unsigned());
        assert_eq!(0.5, FixedPoint::<1, u64>::new(1).to_f64_unsigned());
        assert_eq!(1.5, FixedPoint::<1, u64>::new(3).to_f64_unsigned());
        assert_eq!(1.0, FixedPoint::<10, u64>::new(1024).to_f64_unsigned());
        assert_eq!(1.25, FixedPoint::<10, u64>::new(1024 + 256).to_f64_unsigned());

        assert!(FixedPoint::<1022, u64>::new(1).to_f64_unsigned().is_normal());
        assert!(FixedPoint::<-1023, u64>::new(1).to_f64_unsigned().is_normal());

        assert!(FixedPoint::<1074, u64>::new(1).to_f64_unsigned().is_subnormal());
        assert_eq!(f64::from_bits(0b1), FixedPoint::<1074, u64>::new(1).to_f64_unsigned());
        assert_eq!(f64::from_bits(0b10), FixedPoint::<1073, u64>::new(1).to_f64_unsigned());
        assert_eq!(f64::from_bits(0b1), FixedPoint::<1075, u64>::new(2).to_f64_unsigned());

        assert_eq!(f64::INFINITY, FixedPoint::<-1024, u64>::new(1).to_f64_unsigned());
        assert_eq!(0.0, FixedPoint::<1075, u64>::new(1).to_f64_unsigned());

        assert_eq!(-1.0, FixedPoint::<0, i64>::new(-1).to_f64_signed());
        assert_eq!(-0.25, FixedPoint::<2, i64>::new(-1).to_f64_signed());
    }

    #[test]
    fn from_f32_works() {
        assert!(FixedPoint::<0, i32>::try_from(f32::INFINITY).is_err());
        assert!(FixedPoint::<0, i32>::try_from(f32::NEG_INFINITY).is_err());
        assert!(FixedPoint::<0, i32>::try_from(f32::NAN).is_err());

        assert_eq!(0, FixedPoint::<0, i32>::try_from(0.0).unwrap().raw());
        assert_eq!(1, FixedPoint::<0, i32>::try_from(1.0).unwrap().raw());
        assert_eq!(1, FixedPoint::<1, i32>::try_from(0.5).unwrap().raw());
        assert_eq!(1, FixedPoint::<3, i32>::try_from(0.125).unwrap().raw());
        assert_eq!(-1, FixedPoint::<3, i32>::try_from(-0.125).unwrap().raw());

        // denorms
        assert_eq!(0b1, FixedPoint::<149, i32>::try_from(f32::from_bits(0b1)).unwrap().raw());
        assert_eq!(0b101, FixedPoint::<148, i32>::try_from(f32::from_bits(0b1010)).unwrap().raw());

        let a: FixedPoint<22, i32> = core::f32::consts::PI.try_into().unwrap();
        assert_eq!(core::f32::consts::PI, a.to_f32_signed());
    }

    #[test]
    fn from_f64_works() {
        assert!(FixedPoint::<0, i64>::try_from(f64::INFINITY).is_err());
        assert!(FixedPoint::<0, i64>::try_from(f64::NEG_INFINITY).is_err());
        assert!(FixedPoint::<0, i64>::try_from(f64::NAN).is_err());

        assert_eq!(0, FixedPoint::<0, i64>::try_from(0.0).unwrap().raw());
        assert_eq!(1, FixedPoint::<0, i64>::try_from(1.0).unwrap().raw());
        assert_eq!(1, FixedPoint::<1, i64>::try_from(0.5).unwrap().raw());
        assert_eq!(1, FixedPoint::<3, i64>::try_from(0.125).unwrap().raw());
        assert_eq!(-1, FixedPoint::<3, i64>::try_from(-0.125).unwrap().raw());

        // denorms
        assert_eq!(0b1, FixedPoint::<1074, i64>::try_from(f64::from_bits(0b1)).unwrap().raw());
        assert_eq!(0b101, FixedPoint::<1073, i64>::try_from(f64::from_bits(0b1010)).unwrap().raw());

        let a: FixedPoint<48, i64> = core::f64::consts::PI.try_into().unwrap();
        assert_eq!(core::f64::consts::PI, a.to_f64_signed());
    }


    #[test]
    fn from_t_works() {
        type Fx = FixedPoint<0, i32>;
        let a: Fx = 10.into();
        assert_eq!(10, a.raw());
    }
}