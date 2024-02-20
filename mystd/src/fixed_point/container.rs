
pub trait FixedPointContainer<T>:
    Default
    + Copy
    // + core::cmp::Eq
    + core::ops::Shr<usize, Output = T>
    + core::ops::Shl<usize, Output = T>
    + core::ops::Mul<T, Output = T>
    // + core::ops::Sub<Output = T>
    // + core::ops::BitAnd<Output = T>
    // + core::ops::BitXor<Output = T>
{
    const BITS: u32 = (core::mem::size_of::<Self>() * 8).ilog2();
    const BIT_SHIFT_MASK: usize = (1 << Self::BITS) - 1;

    fn signed_shl(self, amount: isize) -> T where T: Default {
        let abs = amount.unsigned_abs();
        if abs & !Self::BIT_SHIFT_MASK > 0 {
            T::default()
        } else if amount < 0 {
            self >> abs
        } else {
            self << abs
        }
    }
    
    fn signed_shift(self, from: isize, to: isize) -> T where T: Default {
        let diff = to - from;
        self.signed_shl(diff)
    }
}
impl FixedPointContainer<u8> for u8 {}
impl FixedPointContainer<u16> for u16 {}
impl FixedPointContainer<u32> for u32 {}
impl FixedPointContainer<u64> for u64 {}
impl FixedPointContainer<u128> for u128 {}
impl FixedPointContainer<usize> for usize {}
impl FixedPointContainer<i8> for i8 {}
impl FixedPointContainer<i16> for i16 {}
impl FixedPointContainer<i32> for i32 {}
impl FixedPointContainer<i64> for i64 {}
impl FixedPointContainer<i128> for i128 {}
impl FixedPointContainer<isize> for isize {}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_shift_works() {
        assert_eq!(63, u64::BIT_SHIFT_MASK);
        assert_eq!(31, u32::BIT_SHIFT_MASK);
        assert_eq!(15, u16::BIT_SHIFT_MASK);

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

