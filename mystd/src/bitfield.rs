use core::fmt::{Binary, Debug, Write};

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
    + core::fmt::Binary
    + core::fmt::Octal
    + core::fmt::LowerHex
    + core::fmt::UpperHex
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
#[derive(Clone, Copy)]
pub struct BitField<T>(pub T)
where
    T: BitContainer;

impl<T> Debug for BitField<T>
where
    T: BitContainer
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            let nibbles = self.nibbles();
            let len = nibbles.len();
            let mut r = f.debug_struct("BitField");
            for i in 0..(len / 8) {
                let bunch = nibbles.skip(i * 8).take(8);
                let bin = Biner(bunch.clone());
                let hex = Hexer(bunch.clone());
                r.field("bin", &bin);
                r.field("hex", &hex);
            }
            r.finish()
        } else {
            write!(f, "BitField[{}]({})", Self::BIT_WIDTH, &self)
        }
    }
}

impl<T> core::fmt::Display for BitField<T>
where
    T: BitContainer
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:b}", &self)
    }
}

impl<T> core::fmt::Binary for BitField<T>
where
    T: BitContainer
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:0width$b}", &self.0, width=Self::BIT_WIDTH)
    }
}

impl<T> core::fmt::LowerHex for BitField<T>
where
    T: BitContainer
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:0width$x}", &self.0, width=Self::HEX_WIDTH)
    }
}

impl<T> core::fmt::UpperHex for BitField<T>
where
    T: BitContainer
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:0width$X}", &self.0, width=Self::HEX_WIDTH)
    }
}

impl<T> core::fmt::Octal for BitField<T>
where
    T: BitContainer
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:0width$o}", &self.0, width=Self::OCTAL_WIDTH)
    }
}





impl<T> BitField<T>
where
    T: BitContainer,
{
    const BIT_WIDTH: usize = core::mem::size_of::<T>() * 8;
    const OCTAL_WIDTH: usize = (Self::BIT_WIDTH + 2) / 3;
    const HEX_WIDTH: usize = core::mem::size_of::<T>() * 2;

    pub const fn zero() -> Self {
        Self(T::ZERO)
    }

    pub const fn new(value: T) -> Self {
        Self(value)
    }

    pub const fn len(&self) -> usize {
        Self::BIT_WIDTH
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

    pub fn with_field_set(&self, lsb: usize, length: usize, value: T) -> Self {
        Self(((value & Self::mask_up_to(length)) << lsb) | (self.0 & !(Self::mask_up_to(length) << lsb)))
    }

    pub fn bit_test(&self, position: usize) -> bool {
        self.0 & Self::mask_bit(position) != T::ZERO
    }

    pub fn bit_set(&mut self, position: usize) {
        self.0 = self.0 | Self::mask_bit(position);
    }

    pub fn with_bit_set(&self, position: usize) -> Self {
        Self(self.0 | Self::mask_bit(position))
    }

    pub fn bit_clear(&mut self, position: usize) {
        self.0 = self.0 & !Self::mask_bit(position);
    }

    pub fn with_bit_cleared(&self, position: usize) -> Self {
        Self(self.0 & !Self::mask_bit(position))
    }

    pub fn bit_value(&self, position: usize) -> T {
        (self.0 >> position) & T::ONE
    }

    pub fn nibbles(&self) -> FieldIterator<T, 4> {
        FieldIterator::new(*self)
    }
}

#[derive(Clone, Copy)]
pub struct FieldIterator<T, const LEN: usize> where T: BitContainer {
    top: usize,
    bottom: usize,
    bit_field: BitField<T>
}

impl<T, const LEN: usize> ExactSizeIterator for FieldIterator<T, LEN>
where T: BitContainer
{
    fn len(&self) -> usize {
        ((self.top - self.bottom) + (LEN - 1)) / LEN
    }
}

struct Hexer<T, I>(I) where T: BitContainer, I: Iterator<Item = (T, usize)> + Clone;
struct Biner<T, I>(I) where T: BitContainer, I: Iterator<Item = (T, usize)> + Clone;

impl<T, I> core::fmt::Debug for Biner<T, I>
where T: BitContainer, I: Iterator<Item = (T, usize)> + Clone
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (i, field) in self.0.clone().enumerate() {
            if i != 0 {
                f.write_char(' ')?;
            }
            write!(f, "{:0width$b}", field.0, width=field.1)?
        }
        Ok(())
    }
}

impl<T, I> core::fmt::Debug for Hexer<T, I>
where T: BitContainer, I: Iterator<Item = (T, usize)> + Clone
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (i, field) in self.0.clone().enumerate() {
            if i != 0 {
                f.write_char(' ')?;
            }
            write!(f, "{:>width$x}", field.0, width=field.1)?
        }
        Ok(())
    }
}


impl<T, const LEN: usize> FieldIterator<T, LEN>
where T: BitContainer
{
    const TOP_START: usize = BitField::<T>::BIT_WIDTH;
    const REM: usize = BitField::<T>::BIT_WIDTH % LEN;

    const BOTTOM_END: usize = BitField::<T>::BIT_WIDTH - Self::REM;

    pub const fn new(bit_field: BitField<T>) -> Self {
        Self {
            top: Self::TOP_START,
            bottom: 0,
            bit_field
        }
    }
}



impl<T, const LEN: usize> core::iter::Iterator for FieldIterator<T, LEN>
where T: BitContainer
{
    type Item = (T, usize);
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.top <= self.bottom {
            None
        } else {
            if Self::REM != 0 && self.top == Self::TOP_START {
                self.top -= Self::REM;
                Some((self.bit_field.field(self.top, Self::REM), Self::REM))
            } else {
                self.top -= LEN;
                Some((self.bit_field.field(self.top, LEN), LEN))
            }
        }
    }
}


impl<T, const LEN: usize> core::iter::DoubleEndedIterator for FieldIterator<T, LEN>
where T: BitContainer
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.bottom >= self.top {
            None
        } else {
            if Self::REM != 0 && self.bottom == Self::BOTTOM_END {
                let r = self.bit_field.field(self.bottom, Self::REM);
                self.bottom += Self::REM;
                Some((r, Self::REM))
            } else {
                let r = self.bit_field.field(self.bottom, LEN);
                self.bottom += LEN;
                Some((r, LEN))
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use crate::collections::ring::RingArray;
    use core::fmt::Write;

    use super::*;

    #[test]
    fn debug_fmt_works() {
        let mut buf: RingArray<u8, 32> = RingArray::new();
        write!(&mut buf, "{:?}", BitField::<u8>::new(0b101010)).expect("Debug printing should work");
        assert_eq!("BitField[8](00101010)", buf.to_str().unwrap());
    }
    #[test]
    fn debug_alt_fmt_works() {
        let mut buf: RingArray<u8, 128> = RingArray::new();
        write!(&mut buf, "{:#?}", BitField::<u8>::new(0b101010)).expect("Debug printing should work");
        assert_eq!("BitField {\n    bin: 0010 1010,\n    hex:    2    a,\n}", buf.to_str().unwrap());
    }

    #[test]
    fn display_fmt_works() {
        let mut buf: RingArray<u8, 32> = RingArray::new();
        write!(&mut buf, "{}", BitField::<u8>::new(0b101010)).expect("Debug printing should work");
        assert_eq!("00101010", buf.to_str().unwrap());
    }

    #[test]
    fn octal_fmt_works() {
        let mut buf: RingArray<u8, 32> = RingArray::new();
        write!(&mut buf, "{:o}", BitField::<u8>::new(0b101010)).expect("Debug printing should work");
        assert_eq!("052", buf.to_str().unwrap());
    }

    #[test]
    fn hex_fmt_works() {
        let mut buf: RingArray<u8, 32> = RingArray::new();
        write!(&mut buf, "{:x}", BitField::<u8>::new(0b101010)).expect("Debug printing should work");
        assert_eq!("2a", buf.to_str().unwrap());
    }

    #[test]
    fn nibbles_works() {
        let mut nibbles = BitField::<u8>::new(0b101010).nibbles();
        assert_eq!(Some((0b10, 4)), nibbles.next());
        assert_eq!(Some((0b1010, 4)), nibbles.next());
        assert_eq!(None, nibbles.next());
    }
}
