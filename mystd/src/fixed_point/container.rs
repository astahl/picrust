pub trait FixedPointContainer:
    Copy
    // + core::cmp::Eq
    + core::ops::Shr<usize, Output = Self>
    + core::ops::Shl<usize, Output = Self>
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::BitAnd<Output = Self>
    + core::ops::Not<Output = Self>
    // + core::ops::BitXor<Output = T>
{
    const BIT_WIDTH: usize = (core::mem::size_of::<Self>() * 8) as usize;
    const BIT_SHIFT_MAX: usize = Self::BIT_WIDTH - 1;
    
    const ONE: Self;
    const ZERO: Self;

    fn signed_shl(self, amount: isize) -> Self {
        let abs = amount.unsigned_abs();
        if abs > Self::BIT_SHIFT_MAX {
            Self::ZERO
        } else if amount < 0 {
            self >> abs
        } else {
            self << abs
        }
    }

    fn signed_shift(self, from: isize, to: isize) -> Self {
        let diff = to - from;
        self.signed_shl(diff)
    }
}
impl FixedPointContainer for u8 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
}
impl FixedPointContainer for u16 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
}
impl FixedPointContainer for u32 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
}
impl FixedPointContainer for u64 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
}
impl FixedPointContainer for u128 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
}
impl FixedPointContainer for usize {
    const ONE: Self = 1;
    const ZERO: Self = 0;
}
impl FixedPointContainer for i8 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
}
impl FixedPointContainer for i16 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
}
impl FixedPointContainer for i32 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
}
impl FixedPointContainer for i64 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
}
impl FixedPointContainer for i128 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
}
impl FixedPointContainer for isize {
    const ONE: Self = 1;
    const ZERO: Self = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_shift_works() {
        assert_eq!(64, u64::BIT_WIDTH);
        assert_eq!(63, u64::BIT_SHIFT_MAX);
        assert_eq!(31, u32::BIT_SHIFT_MAX);
        assert_eq!(15, u16::BIT_SHIFT_MAX);

        assert_eq!(1 << 10, 1.signed_shift(-10, 0));
        assert_eq!(1 << 10, 1.signed_shift(0, 10));
        assert_eq!(1024 >> 10, 1024.signed_shift(10, 0));
        assert_eq!(1024 >> 10, 1024.signed_shift(5, -5));
        assert_eq!(1024 >> 10, 1024.signed_shift(0, -10));
        assert_eq!(1, 1.signed_shift(10, 10));
        assert_eq!(-2, -1_i8.signed_shl(1));
        assert_eq!(0, -1_i8.signed_shl(-1));

        assert_eq!(0, 0xFF_u8.signed_shl(8))
    }
}
