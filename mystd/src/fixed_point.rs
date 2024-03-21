pub mod container;
pub mod convert;
pub mod ops;
use container::FixedPointContainer;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct FixedPoint<const P: isize, T: FixedPointContainer>(T);

impl<const P: isize, T: FixedPointContainer> FixedPoint<P, T> {
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    pub fn from_int(int: T) -> Self {
        Self(int.signed_shl(P))
    }

    pub fn from_int_frac(int: T, frac: T) -> Self {
        Self(int.signed_shl(P) + frac)
    }

    pub fn from_shifted(value: T, precision: isize) -> Self {
        FixedPoint::new(value.signed_shift(precision, P))
    }

    pub fn truncating_shift<const R: isize>(&self) -> FixedPoint<R, T> {
        FixedPoint::new(self.0.signed_shift(P, R))
    }

    pub fn truncate(&self) -> T {
        self.0.signed_shift(P, 0)
    }

    pub const fn raw(&self) -> T {
        self.0
    }

    pub fn split_int_frac(&self) -> (T, T) {
        if P <= 0 {
            (self.0, T::ZERO)
        } else if P.unsigned_abs() > T::BIT_WIDTH {
            (T::ZERO, self.0)
        } else {
            let mask = (T::ONE << P as usize) - T::ONE;
            ((self.0 & !mask).signed_shl(-P), self.0 & mask)
        }
    }
}

impl<const P: isize, T: FixedPointContainer> Default for FixedPoint<P, T> {
    fn default() -> Self {
        Self(T::ZERO)
    }
}

impl<const P: isize, T: FixedPointContainer + core::fmt::Debug> core::fmt::Debug
    for FixedPoint<P, T>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("FixedPoint")
            .field(&P)
            .field(&self.0)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::collections;

    use super::*;

    #[test]
    fn fmt_debug_works() {
        use core::fmt::Write;

        let mut buff = collections::ring::RingArray::<u8, 32>::new();
        write!(buff, "{:?}", FixedPoint::<10, i32>::new(-1234)).expect("Writing should work");
        assert_eq!(b"FixedPoint(10, -1234)", buff.as_slices().0);
    }

    #[test]
    fn split_works() {
        let a: FixedPoint<8, i32> = (3.25_f32).try_into().unwrap();
        assert_eq!((3, 0b0100_0000), a.split_int_frac());

        let a: FixedPoint<8, i32> = (-3.5_f32).try_into().unwrap();
        assert_eq!((-4, 0b1000_0000), a.split_int_frac());

        // let a: FixedPoint::<8, i32> = (-3.375_f32).try_into().unwrap();
        // assert_eq!((-4, 0b0101_0000), a.split_int_frac());

        let f_uart_clk = FixedPoint::<6, u32>::from_int(3_000_000);
        let baud_rate = 115200;
        let baud_rate_divisor = f_uart_clk / (16 * baud_rate);
        assert_eq!((1, 40), baud_rate_divisor.split_int_frac());
    }
}
