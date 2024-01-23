pub trait BitContainer<T>:
    Default
    + Copy
    + core::cmp::Eq
    + core::ops::Shr<usize, Output = T>
    + core::ops::Shl<usize, Output = T>
    + core::ops::Sub<Output = T>
    + core::ops::BitAnd<Output = T>
    + core::ops::Not<Output = T>
{
}
impl BitContainer<u8> for u8 {}
impl BitContainer<u16> for u16 {}
impl BitContainer<u32> for u32 {}
impl BitContainer<u64> for u64 {}
impl BitContainer<u128> for u128 {}
impl BitContainer<usize> for usize {}
impl BitContainer<i8> for i8 {}
impl BitContainer<i16> for i16 {}
impl BitContainer<i32> for i32 {}
impl BitContainer<i64> for i64 {}
impl BitContainer<i128> for i128 {}
impl BitContainer<isize> for isize {}

pub struct Bitfield<T>(pub T)
where
    T: BitContainer<T>;

impl<T> Bitfield<T>
where
    T: BitContainer<T>,
{
    pub fn new(value: T) -> Self {
        Self(value)
    }

    fn zero() -> T {
        T::default()
    }

    fn one() -> T {
        !(!Self::zero() << 1)
    }

    fn mask_bit(position: usize) -> T {
        Self::one() << position
    }

    fn mask_up_to(position: usize) -> T {
        Self::mask_bit(position) - Self::one()
    }

    pub fn field(&self, lsb: usize, length: usize) -> T {
        (self.0 >> lsb) & Self::mask_up_to(length)
    }

    pub fn bit(&self, position: usize) -> bool {
        self.0 & Self::mask_bit(position) != Self::zero()
    }
}
