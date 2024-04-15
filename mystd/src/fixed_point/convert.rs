use crate::fixed_point::{FxS32, FxS64, SignedShifter};

use super::FixedPointConversionError;

mod consts {
    pub mod f32 {
        pub const EXPONENT_BIAS: i32 = 127;
        pub const MAX_EXPONENT: i32 = 127;
        pub const MIN_EXPONENT: i32 = -126;
        pub const MANTISSA_WIDTH: i32 = 23;
        pub const DENORM_EXPONENT: i32 = MIN_EXPONENT - MANTISSA_WIDTH;
        pub const MANTISSA_MASK: u32 = (1 << MANTISSA_WIDTH) - 1;
        pub const EXPONENT_MASK: u32 = (1 << 8) - 1;
        pub const SIGN_MASK: u32 = 1 << 31;
    }
    pub mod f64 {
        pub const EXPONENT_BIAS: i32 = 1023;
        pub const MAX_EXPONENT: i32 = 1023;
        pub const MIN_EXPONENT: i32 = -1022;
        pub const MANTISSA_WIDTH: i32 = 52;
        pub const DENORM_EXPONENT: i32 = MIN_EXPONENT - MANTISSA_WIDTH;
        pub const MANTISSA_MASK: u64 = (1 << MANTISSA_WIDTH) - 1;
        pub const EXPONENT_MASK: u64 = (1 << 11) - 1;
        pub const SIGN_MASK: u64 = 1 << 63;
    }
}



pub fn fixed_u32_to_float32(value: u32, precision: i32) -> f32 {
    use consts::f32::*;

    if value == 0 {
        return 0.0;
    }

    // take the number of leading zeros, so we can shift the mantissa so the first 1 is at lsb 24 (the mantissa in f32 is 23 bits long)
    let most_significant_one = 31 - value.leading_zeros() as i32;
    let exponent = most_significant_one - precision;

    if exponent > MAX_EXPONENT {
        f32::INFINITY
    } else if exponent >= MIN_EXPONENT {
        let mantissa = SignedShifter(value).zeroing_shift_from_to(most_significant_one, MANTISSA_WIDTH);
        f32::from_bits(
            ((exponent + EXPONENT_BIAS) as u32 & EXPONENT_MASK) << MANTISSA_WIDTH
                | (mantissa & MANTISSA_MASK),
        )
    } else if exponent >= DENORM_EXPONENT {
        let mantissa = SignedShifter(value).zeroing_shift_from_to(precision, -DENORM_EXPONENT);
        // handle denormalization
        f32::from_bits(mantissa & MANTISSA_MASK)
    } else {
        0.0
    }
}

pub fn fixed_i32_to_float32(value: i32, precision: i32) -> f32 {
    if value < 0 {
        -fixed_u32_to_float32(value.unsigned_abs(), precision)
    } else {
        fixed_u32_to_float32(value as u32, precision)
    }
}

pub fn fixed_u64_to_float64(value: u64, precision: i32) -> f64 {
    use consts::f64::*;

    if value == 0 {
        return 0.0;
    }

    // take the number of leading zeros, so we can shift the mantissa so the first 1 is at lsb 53 (the mantissa in f32 is 52 bits long)
    let most_significant_one = 63 - value.leading_zeros() as i32;
    let exponent = most_significant_one - precision;

    if exponent > MAX_EXPONENT {
        f64::INFINITY
    } else if exponent >= MIN_EXPONENT {
        let mantissa = SignedShifter(value).zeroing_shift_from_to(most_significant_one, MANTISSA_WIDTH);
        f64::from_bits(
            ((exponent + EXPONENT_BIAS) as u64 & EXPONENT_MASK) << MANTISSA_WIDTH
                | (mantissa & MANTISSA_MASK),
        )
    } else if exponent >= DENORM_EXPONENT {
        let mantissa = SignedShifter(value).zeroing_shift_from_to(precision, -DENORM_EXPONENT);
        // handle denormalization
        f64::from_bits(mantissa & MANTISSA_MASK)
    } else {
        0.0
    }
}

pub fn fixed_i64_to_float64(value: i64, precision: i32) -> f64 {
    if value < 0 {
        -fixed_u64_to_float64(value.unsigned_abs(), precision)
    } else {
        fixed_u64_to_float64(value as u64, precision)
    }
}

pub fn float64_to_fixed_i64(value: f64, target_precision: i32) -> Result<i64, FixedPointConversionError> {
    use consts::f64::*;
    if !value.is_finite() {
        return Err(FixedPointConversionError::ValueIsNotFinite);
    }
    let bits = value.to_bits();
    if bits & !SIGN_MASK == 0 {
        return Ok(0);
    }
    let biased_exponent = EXPONENT_MASK & (bits >> MANTISSA_WIDTH);

    let mut mantissa = MANTISSA_MASK & bits;
    let precision = if biased_exponent == 0 {
        // denormal value
        -DENORM_EXPONENT
    } else {
        // normal value, add the implicit leading 1
        mantissa |= 1 << MANTISSA_WIDTH;
        MANTISSA_WIDTH + EXPONENT_BIAS - biased_exponent as i32
    };

    let signed_value = if bits & SIGN_MASK != 0 {
        -(mantissa as i64)
    } else {
        mantissa as i64
    };

    Ok(SignedShifter(signed_value).zeroing_shift_from_to(precision, target_precision))
}

pub fn float64_to_fixed_u64(value: f64, target_precision: i32) -> Result<u64, FixedPointConversionError> {
    if value.is_sign_negative() {
        Ok(0)
    } else {
        Ok(float64_to_fixed_i64(value, target_precision)? as u64)
    }
}

pub fn float32_to_fixed_i32(value: f32, target_precision: i32) -> Result<i32, FixedPointConversionError> {
    use consts::f32::*;
    if !value.is_finite() {
        return Err(FixedPointConversionError::ValueIsNotFinite);
    }
    let bits = value.to_bits();
    if bits & !SIGN_MASK == 0 {
        return Ok(0);
    }
    let biased_exponent = EXPONENT_MASK & (bits >> MANTISSA_WIDTH);

    let mut mantissa = MANTISSA_MASK & bits;
    let precision = if biased_exponent == 0 {
        // denormal value
        -DENORM_EXPONENT
    } else {
        // normal value, add the implicit leading 1
        mantissa |= 1 << MANTISSA_WIDTH;
        MANTISSA_WIDTH + EXPONENT_BIAS - biased_exponent as i32
    };

    let signed_value = if bits & SIGN_MASK != 0 {
        -(mantissa as i32)
    } else {
        mantissa as i32
    };

    Ok(SignedShifter(signed_value).zeroing_shift_from_to(precision, target_precision))
}

pub fn float32_to_fixed_u32(value: f32, target_precision: i32) -> Result<u32, FixedPointConversionError> {
    if value.is_sign_negative() {
        Ok(0)
    } else {
        Ok(float32_to_fixed_i32(value, target_precision)? as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;
    #[test]
    fn to_f32_works() {
        assert_eq!(0.0_f32, FxU32::<1000>::new(0).into());
        assert_eq!(0.0_f32, FxU32::<-1000>::new(0).into());
        assert_eq!(0.0_f32, FxU32::<0>::new(0).into());

        assert_eq!(1.0_f32, FxU32::<0>::new(1).into());
        assert_eq!(0.5_f32, FxU32::<1>::new(1).into());
        assert_eq!(1.5_f32, FxU32::<1>::new(3).into());
        assert_eq!(1.0_f32, FxU32::<10>::new(1024).into());
        assert_eq!(
            1.25_f32,
            FxU32::<10>::new(1024 + 256).into()
        );

        assert!(<FxU32<126> as Into<f32>>::into(FxU32::<126>::new(1)).is_normal());
        assert!(<FxU32<-127> as Into<f32>>::into(FxU32::<-127>::new(1)).is_normal());
        assert!(<FxU32<149> as Into<f32>>::into(FxU32::<149>::new(1)).is_subnormal());

        assert_eq!(
            f32::from_bits(0b1),
            FxU32::<149>::new(1).into()
        );
        assert_eq!(
            f32::from_bits(0b10),
            FxU32::<148>::new(1).into()
        );
        assert_eq!(
            f32::from_bits(0b1),
            FxU32::<150>::new(2).into()
        );

        assert_eq!(
            f32::INFINITY,
            FxU32::<-128>::new(1).into()
        );
        assert_eq!(0.0_f32, FxU32::<150>::new(1).into());

        assert_eq!(-1.0_f32, FxS32::<0>::new(-1).into());
        assert_eq!(-0.25_f32, FxS32::<2>::new(-1).into());
    }

    #[test]
    fn to_f64_works() {
        assert_eq!(0.0, FxU64::<2000>::new(0).into());
        assert_eq!(0.0, FxU64::<-2000>::new(0).into());
        assert_eq!(0.0, FxU64::<0>::new(0).into());

        assert_eq!(1.0, FxU64::<0>::new(1).into());
        assert_eq!(0.5, FxU64::<1>::new(1).into());
        assert_eq!(1.5, FxU64::<1>::new(3).into());
        assert_eq!(1.0, FxU64::<10>::new(1024).into());
        assert_eq!(
            1.25,
            FxU64::<10>::new(1024 + 256).into()
        );

        assert!(<FxU64<1022> as Into<f64>>::into(FxU64::<1022>::new(1)).is_normal());
        assert!(<FxU64<-1023> as Into<f64>>::into(FxU64::<-1023>::new(1))
            .is_normal());

        assert!(
            <FxU64<1074> as Into<f64>>::into(FxU64::<1074>::new(1))
            .is_subnormal());
        assert_eq!(
            f64::from_bits(0b1),
            FxU64::<1074>::new(1).into()
        );
        assert_eq!(
            f64::from_bits(0b10),
            FxU64::<1073>::new(1).into()
        );
        assert_eq!(
            f64::from_bits(0b1),
            FxU64::<1075>::new(2).into()
        );

        assert_eq!(
            f64::INFINITY,
            FxU64::<-1024>::new(1).into()
        );
        assert_eq!(0.0, FxU64::<1075>::new(1).into());

        assert_eq!(-1.0, FxS64::<0>::new(-1).into());
        assert_eq!(-0.25, FxS64::<2>::new(-1).into());
    }

    #[test]
    fn from_f32_works() {
        assert!(FxS32::<0>::try_from(f32::INFINITY).is_err());
        assert!(FxS32::<0>::try_from(f32::NEG_INFINITY).is_err());
        assert!(FxS32::<0>::try_from(f32::NAN).is_err());

        assert_eq!(0, FxS32::<0>::try_from(0.0).unwrap().raw());
        assert_eq!(1, FxS32::<0>::try_from(1.0).unwrap().raw());
        assert_eq!(1, FxS32::<1>::try_from(0.5).unwrap().raw());
        assert_eq!(1, FxS32::<3>::try_from(0.125).unwrap().raw());
        assert_eq!(-1, FxS32::<3>::try_from(-0.125).unwrap().raw());

        assert_eq!(8, FxS32::<3>::try_from(1.0).unwrap().raw());
        assert_eq!(-8, FxS32::<3>::try_from(-1.0).unwrap().raw());
        assert_eq!(-16, FxS32::<3>::try_from(-2.0).unwrap().raw());
        assert_eq!(-24, FxS32::<3>::try_from(-3.0).unwrap().raw());
        assert_eq!(24, FxS32::<3>::try_from(3.0).unwrap().raw());
        assert_eq!(3, FxS32::<3>::try_from(3.0).unwrap().truncate());
        assert_eq!(-4, FxS32::<3>::try_from(-3.1).unwrap().truncate());

        // denorms
        assert_eq!(
            0b1,
            FxS32::<149>::try_from(f32::from_bits(0b1))
                .unwrap()
                .raw()
        );
        assert_eq!(
            0b101,
            FxS32::<148>::try_from(f32::from_bits(0b1010))
                .unwrap()
                .raw()
        );

        let a: FxS32<22> = core::f32::consts::PI.try_into().unwrap();
        assert_eq!(core::f32::consts::PI, a.into());
    }

    #[test]
    fn from_f64_works() {
        assert!(FxS64::<0>::try_from(f64::INFINITY).is_err());
        assert!(FxS64::<0>::try_from(f64::NEG_INFINITY).is_err());
        assert!(FxS64::<0>::try_from(f64::NAN).is_err());

        assert_eq!(0, FxS64::<0>::try_from(0.0).unwrap().raw());
        assert_eq!(1, FxS64::<0>::try_from(1.0).unwrap().raw());
        assert_eq!(1, FxS64::<1>::try_from(0.5).unwrap().raw());
        assert_eq!(1, FxS64::<3>::try_from(0.125).unwrap().raw());
        assert_eq!(-1, FxS64::<3>::try_from(-0.125).unwrap().raw());

        // denorms
        assert_eq!(
            0b1,
            FxS64::<1074>::try_from(f64::from_bits(0b1))
                .unwrap()
                .raw()
        );
        assert_eq!(
            0b101,
            FxS64::<1073>::try_from(f64::from_bits(0b1010))
                .unwrap()
                .raw()
        );

        let a: FxS64::<48> = core::f64::consts::PI.try_into().unwrap();
        assert_eq!(core::f64::consts::PI, a.into());
    }

    // #[test]
    // fn from_t_works() {
    //     type Fx = FixedPoint<0, i32>;
    //     let a: Fx = 10.into();
    //     assert_eq!(10, a.raw());
    // }
}
