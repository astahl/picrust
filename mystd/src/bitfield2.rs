
#[derive(Clone, Copy)]
pub struct BitMask<const N: usize, T: ?Sized + Copy>(T);

impl<const N: usize, T: ?Sized + Copy> BitMask<N, T> {
    pub const POSITION: usize = N;

    pub const fn position(self) -> usize {
        Self::POSITION
    }
}

#[derive(Clone, Copy)]
        pub struct FieldMask<const FROM: usize, const TO: usize, T: ?Sized + Copy>(T);

        impl<const FROM: usize, const TO: usize, T: ?Sized + Copy> FieldMask<FROM, TO, T> {
            const MIN_MAX: (usize, usize) = if FROM < TO { (FROM,TO) } else { (TO,FROM)};
            pub const LSB: usize = Self::MIN_MAX.0;
            pub const MSB: usize = Self::MIN_MAX.1; 

            pub const fn msb(self) -> usize {
                Self::MSB
            }

            pub const fn lsb(self) -> usize {
                Self::LSB
            }
        }

macro_rules! ensure_bit_fits {
    ($type:tt $bit:literal) => {
        #[deny(arithmetic_overflow)]
        let _ = (1 as $type << $bit > 0);
    }
}

macro_rules! bit_field_method {
    ($(#[$meta:meta])* $type_name:ident $underlying_type:ty, $field_name:ident $field_from:literal $field_to:literal) => {
        
        $(#[$meta])*
        pub const fn $field_name(self) -> FieldMask<$field_from, $field_to, $type_name> {
            ensure_bit_fits!($underlying_type $field_from);
            ensure_bit_fits!($underlying_type $field_to);
            FieldMask(self)
        }
    
    };
    ($(#[$meta:meta])* $type_name:ident $underlying_type:ty, $bit_name:ident $bit_position:literal) => {

        $(#[$meta])*
        pub const fn $bit_name(self) -> BitMask<$bit_position, $type_name> {
            ensure_bit_fits!($underlying_type $bit_position);
            BitMask(self)
        }

    };
}

#[macro_export]
macro_rules! bit_field {
    ($v:vis $type_name:ident ($underlying_type:ty) $($(#[$bit_meta:meta])* $bit_from:literal $(:$bit_to:literal)? => $bit_name:ident),*) => {
        #[repr(transparent)]
        #[derive(Copy, Clone)]
        $v struct $type_name($underlying_type);

        impl $type_name {
            pub const ZERO: Self = Self::zero();

            pub const fn zero() -> Self {
                $(
                    assert!($bit_from < <$underlying_type>::BITS);
                )*
                Self(0)
            }

            pub const fn new(value: $underlying_type) -> Self {
                Self(value)
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
                bit_field_method!($(#[$bit_meta])* $type_name $underlying_type, $bit_name $bit_from $($bit_to)?);
            )*

        }

        impl<const N: usize> BitMask<N, $type_name> {
            #[must_use]
            pub const fn set(self) -> $type_name {
                self.0.with_bit_set(Self::POSITION)
            }

            #[must_use]
            pub const fn clear(self) -> $type_name {
                self.0.with_bit_cleared(Self::POSITION)
            }

            pub const fn is_set(self) -> bool {
                self.0.is_bit_set(Self::POSITION)
            }
        }

        impl<const FROM: usize, const TO: usize> FieldMask<FROM, TO, $type_name> {

            pub const fn value(self) -> $underlying_type {
                self.0.field_value(Self::LSB, Self::MSB)
            }

            #[must_use]
            pub const fn set_value(self, value: $underlying_type) -> $type_name {
                self.0.with_field_value(Self::LSB, Self::MSB, value)
            }

            #[must_use]
            pub const fn all_clear(self) -> $type_name {
                self.0.with_field_all_cleared(Self::LSB, Self::MSB)
            }

            #[must_use]
            pub const fn all_set(self) -> $type_name {
                self.0.with_field_all_set(Self::LSB, Self::MSB)
            }
        }
    
    };
}

bit_field!(
    pub MyReg(u8) 
    2 => a, 
    /// probably fine
    3 => b,
    /// #The best field
    /// 
    /// A field so good it shows
    0:7 => my_field    
);

bit_field!(pub X(u32) 3 => x);



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let x = MyReg::new(0b1100100);
        let z = MyReg::zero();
        assert!(x.a().is_set());
        assert!(!z.a().is_set());
        assert_eq!(0b1100100, x.my_field().value());
    }
}