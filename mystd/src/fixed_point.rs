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
}

impl<const P: isize, T: FixedPointContainer + Default> Default for FixedPoint<P, T> {
    fn default() -> Self {
        Self(Default::default())
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
    use crate::buffer;

    use super::*;

    #[test]
    fn fmt_debug_works() {
        use core::fmt::Write;
        
        let mut buff = buffer::RingArray::<u8, 32>::new();
        write!(buff, "{:?}", FixedPoint::<10, i32>::new(-1234)).expect("Writing should work");
        assert_eq!(b"FixedPoint(10, -1234)", buff.as_slices().0);
    }
}
