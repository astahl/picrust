pub trait BitContainer:
    Default
    + Copy
    + core::cmp::Eq
    + core::ops::Shr<usize, Output = Self>
    + core::ops::Shl<usize, Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::BitAnd<Output = Self>
    + core::ops::BitOr<Output = Self>
    + core::ops::Not<Output = Self>
{
    const ZERO: Self;
    const ONE: Self;
}
impl BitContainer for u8 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
impl BitContainer for u16 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
impl BitContainer for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
impl BitContainer for u64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
impl BitContainer for u128 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
impl BitContainer for usize {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
impl BitContainer for i8 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
impl BitContainer for i16 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
impl BitContainer for i32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
impl BitContainer for i64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
impl BitContainer for i128 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
impl BitContainer for isize {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}

#[repr(transparent)]
pub struct BitField<T>(pub T)
where
    T: BitContainer;

impl<T> BitField<T>
where
    T: BitContainer,
{
    pub const fn zero() -> Self {
        Self(T::ZERO)
    }

    pub const fn new(value: T) -> Self {
        Self(value)
    }

    fn mask_bit(position: usize) -> T {
        T::ONE << position
    }

    fn mask_up_to(position: usize) -> T {
        Self::mask_bit(position) - T::ONE
    }

    pub fn field(&self, lsb: usize, length: usize) -> T {
        (self.0 >> lsb) & Self::mask_up_to(length)
    }

    pub fn field_set(&mut self, lsb: usize, length: usize, value: T) {
        self.0 = ((value & Self::mask_up_to(length)) << lsb) | (self.0 & !(Self::mask_up_to(length) << lsb))
    }

    pub fn bit_test(&self, position: usize) -> bool {
        self.0 & Self::mask_bit(position) != T::ZERO
    }

    pub fn bit_set(&mut self, position: usize) {
        self.0 = self.0 | Self::mask_bit(position);
    }

    pub fn bit_clear(&mut self, position: usize) {
        self.0 = self.0 & !Self::mask_bit(position);
    }

    pub fn bit_value(&self, position: usize) -> T {
        (self.0 >> position) & T::ONE
    }
}
