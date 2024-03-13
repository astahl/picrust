use core::marker::PhantomData;


#[derive(Clone, Copy, Debug)]
pub enum BitFieldError {
    ValueTooLargeForField
}

pub trait BitFieldable: Copy {
    type Underlying: Copy;
    const BIT_WIDTH: usize;
    fn with_bit_cleared(self, position: usize) -> Self;

    fn with_bit_set(self, position: usize) -> Self;

    fn is_bit_set(self, position: usize) -> bool;

    fn field_value(self, lsb: usize, msb: usize) -> Self::Underlying;

    fn with_field_value(self, lsb: usize, msb: usize, value: Self::Underlying) -> Self;

    fn with_field_all_cleared(self, lsb: usize, msb: usize) -> Self;

    fn with_field_all_set(self, lsb: usize, msb: usize) -> Self;
}


#[derive(Clone, Copy)]
pub struct BitMask<const N: usize, T: BitFieldable>(T);

impl<const N: usize, T: BitFieldable> BitMask<N, T> {
    pub const POSITION: usize = N;
    
    pub const fn new(bitfield: T) -> Self {
        Self(bitfield)
    }

    pub const fn position(self) -> usize {
        Self::POSITION
    }

    #[must_use]
    pub fn set_to(self, value: bool) -> T {
        if value {
            self.0.with_bit_set(Self::POSITION)
        } else {
            self.0.with_bit_cleared(Self::POSITION)
        }
    }

    #[must_use]
    pub fn set(self) -> T {
        self.0.with_bit_set(Self::POSITION)
    }

    #[must_use]
    pub fn clear(self) -> T {
        self.0.with_bit_cleared(Self::POSITION)
    }

    pub fn is_set(self) -> bool {
        self.0.is_bit_set(Self::POSITION)
    }

    pub fn is_clear(self) -> bool {
        !self.is_set()
    }

    pub fn value(self) -> bool {
        self.is_set()
    }
}

#[derive(Clone, Copy)]
pub struct FieldMask<const FROM: usize, const TO: usize, T: BitFieldable>(T);

impl<const FROM: usize, const TO: usize, T: BitFieldable> FieldMask<FROM, TO, T> {
    const MIN_MAX: (usize, usize) = if FROM < TO { (FROM,TO) } else { (TO,FROM)};
    pub const LSB: usize = Self::MIN_MAX.0;
    pub const MSB: usize = Self::MIN_MAX.1; 
    pub const WIDTH: usize = Self::MSB - Self::LSB + 1;
    
    pub const fn new(bitfield: T) -> Self {
        Self(bitfield)
    }

    pub const fn msb(self) -> usize {
        Self::MSB
    }

    pub const fn lsb(self) -> usize {
        Self::LSB
    }

    pub fn value(self) -> T::Underlying {
        self.0.field_value(Self::LSB, Self::MSB)
    }

    pub fn into<U>(self) -> U where U: From<T::Underlying> {
        self.value().into()
    } 

    /// Accepts the underlying type, but truncates the value to the size of the field.
    #[must_use]
    pub fn set_value(self, value: T::Underlying) -> T {
        self.0.with_field_value(Self::LSB, Self::MSB, value)
    }

    #[must_use]
    pub fn all_clear(self) -> T {
        self.0.with_field_all_cleared(Self::LSB, Self::MSB)
    }

    #[must_use]
    pub fn all_set(self) -> T {
        self.0.with_field_all_set(Self::LSB, Self::MSB)
    }
}


#[derive(Clone, Copy)]
pub struct TypedFieldMask<const FROM: usize, const TO: usize, T: BitFieldable, U>(
    FieldMask<FROM, TO, T>,
    PhantomData<U>
);


#[macro_export]
macro_rules! ensure_bit_fits {
    ($type:tt $bit:literal) => {
        #[deny(arithmetic_overflow)]
        let _ = (1 as $type << $bit > 0);
    }
}

#[macro_export]
macro_rules! bit_field_method {
    ($(#[$meta:meta])* $type_name:ident $underlying_type:ty, $field_name:ident $field_from:literal $field_to:literal) => {
        
        $(#[$meta])*
        pub const fn $field_name(self) -> $crate::bitfield2::FieldMask<$field_from, $field_to, $type_name> {
            $crate::ensure_bit_fits!($underlying_type $field_from);
            $crate::ensure_bit_fits!($underlying_type $field_to);
            $crate::bitfield2::FieldMask::new(self)
        }
    
    };
    ($(#[$meta:meta])* $type_name:ident $underlying_type:ty, $bit_name:ident $bit_position:literal) => {

        $(#[$meta])*
        pub const fn $bit_name(self) -> $crate::bitfield2::BitMask<$bit_position, $type_name> {
            $crate::ensure_bit_fits!($underlying_type $bit_position);
            $crate::bitfield2::BitMask::new(self)
        }

    };
}

#[macro_export]
macro_rules! bit_field {
    ($(#[$meta:meta])* $v:vis $type_name:ident ($underlying_type:ty) 
        $(
            $(#[$bit_meta:meta])* 
            $bit_from:literal $(:$bit_to:literal)? => $bit_name:ident$(:$field_type:ty)?
        ),* $(,)?
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Copy, Clone)]
        $v struct $type_name($underlying_type);

        impl $type_name {
            pub const fn zero() -> Self {
                Self(0)
            }

            pub const fn all_set() -> Self {
                Self((0 as $underlying_type).wrapping_sub(1))
            }

            pub const fn new(value: $underlying_type) -> Self {
                Self(value)
            }

            pub const fn to_underlying(self) -> $underlying_type {
                self.0
            }

            pub const fn is_all_clear(self) -> bool {
                self.0 == 0
            }

            const fn _bit_mask(position: usize) -> $underlying_type {
                1 << position
            }

            const fn _field_mask(lsb: usize, msb: usize) -> $underlying_type {
                (((1 << (msb - lsb)) - 1) << lsb) | (1 << msb)
            }

            pub const fn with_bit_cleared(self, position: usize) -> Self {
                Self::new(self.0 & !Self::_bit_mask(position))
            }

            pub const fn with_bit_set(self, position: usize) -> Self {
                Self::new(self.0 | Self::_bit_mask(position))
            }

            pub const fn is_bit_set(self, position: usize) -> bool {
                self.0 & Self::_bit_mask(position) != 0
            }

            pub const fn field_value(self, lsb: usize, msb: usize) -> $underlying_type {
                (self.0 & Self::_field_mask(lsb, msb)) >> lsb
            }

            pub const fn with_field_value(self, lsb: usize, msb: usize, value: $underlying_type) -> Self {
                let mask = Self::_field_mask(lsb, msb);
                Self::new(self.0 & !mask | ((value << lsb) & mask))
            }

            pub const fn with_field_all_cleared(self, lsb: usize, msb: usize) -> Self {
                let mask = Self::_field_mask(lsb, msb);
                Self::new(self.0 & !mask)
            }

            pub const fn with_field_all_set(self, lsb: usize, msb: usize) -> Self {
                let mask = Self::_field_mask(lsb, msb);
                Self::new(self.0 | mask)
            }

            $(
                $crate::bit_field_method!($(#[$bit_meta])* $type_name $underlying_type, $bit_name $bit_from $($bit_to)?);
            )*

        }

        impl $crate::bitfield2::BitFieldable for $type_name {
            type Underlying = $underlying_type;
            const BIT_WIDTH: usize = <$underlying_type>::BITS as usize;

            fn with_bit_cleared(self, position: usize) -> Self {
                self.with_bit_cleared(position)
            }

            fn with_bit_set(self, position: usize) -> Self {
                self.with_bit_set(position)
            }

            fn is_bit_set(self, position: usize) -> bool {
                self.is_bit_set(position)
            }

            fn field_value(self, lsb: usize, msb: usize) -> $underlying_type {
                self.field_value(lsb, msb)
            }

            fn with_field_value(self, lsb: usize, msb: usize, value: $underlying_type) -> Self {
                self.with_field_value(lsb, msb, value)
            }

            fn with_field_all_cleared(self, lsb: usize, msb: usize) -> Self {
                self.with_field_all_cleared(lsb, msb)
            }

            fn with_field_all_set(self, lsb: usize, msb: usize) -> Self {
                self.with_field_all_set(lsb, msb)
            }
        }

        impl From<$underlying_type> for $type_name {
            fn from(val: $underlying_type) -> Self {
                Self::new(val)
            }
        }

        impl core::fmt::Debug for $type_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($type_name))
                    .field("binary", &format_args!("{:#0width$b}", &self.0, width=(<$underlying_type>::BITS as usize)))
                $(
                    .field(concat!(stringify!($bit_name),"[", stringify!($bit_from) $(, ":", stringify!($bit_to))?, "]"),
                        &self.$bit_name().value())
                )*
                    .finish()
            }
        }

        impl core::ops::BitOr<$type_name> for $type_name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                Self::new(self.0 | rhs.0)
            }
        }

        impl core::ops::BitAnd<$type_name> for $type_name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self {
                Self::new(self.0 & rhs.0)
            }
        }
    
    };
}

bit_field!(
    pub MyReg(u8) 
    2 => a, 
    /// probably fine
    3 => b,
    /// # The best field
    /// 
    /// A field so good it shows
    0:7 => my_field    
);


bit_field!(pub X(u32) 3 => x);



#[cfg(test)]
mod tests {
    use crate::collections::ring::RingArray;

    use super::*;

    #[test]
    fn test_name() {
        let x = MyReg::new(0b1100100);
        let z = MyReg::zero();
        assert!(x.a().is_set());
        assert!(!z.a().is_set());
        assert_eq!(0b1100100, x.my_field().value());
    }

    #[test]
    fn fmt_debug_works() {
        use core::fmt::Write;
        let x = MyReg::new(0b1100100);
        let mut buf: RingArray<u8, 64> = RingArray::new();
        write!(&mut buf, "{:?}", x).expect("should work");
        assert_eq!("MyReg { a[2]: true, b[3]: false, my_field[0:7]: 100 }", buf.to_str().unwrap());
    }
}