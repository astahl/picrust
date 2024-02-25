pub trait FixedPointContainer:
    Default
    + Copy
    // + core::cmp::Eq
    + core::ops::Shr<usize, Output = Self>
    + core::ops::Shl<usize, Output = Self>
    // + core::ops::Sub<Output = T>
    // + core::ops::BitAnd<Output = T>
    // + core::ops::BitXor<Output = T>
{
    const BIT_WIDTH: usize = (core::mem::size_of::<Self>() * 8) as usize;
    const BIT_SHIFT_MAX: usize = Self::BIT_WIDTH - 1;

    fn signed_shl(self, amount: isize) -> Self {
        let abs = amount.unsigned_abs();
        if abs > Self::BIT_SHIFT_MAX {
            Self::default()
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
impl FixedPointContainer for u8 {}
impl FixedPointContainer for u16 {}
impl FixedPointContainer for u32 {}
impl FixedPointContainer for u64 {}
impl FixedPointContainer for u128 {}
impl FixedPointContainer for usize {}
impl FixedPointContainer for i8 {}
impl FixedPointContainer for i16 {}
impl FixedPointContainer for i32 {}
impl FixedPointContainer for i64 {}
impl FixedPointContainer for i128 {}
impl FixedPointContainer for isize {}

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
