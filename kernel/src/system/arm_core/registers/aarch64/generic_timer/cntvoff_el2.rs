use mystd::bit_field;

use crate::system_register_impl;

system_register_impl!(cntvoff_el2 CntVOffEl2 (r,w));


/// # CNTVOFF_EL2, Counter-timer Virtual Offset register
/// Holds the 64-bit virtual offset. This is the offset between the physical count value visible in CNTPCT_EL0 and the virtual count value visible in CNTVCT_EL0.
pub struct CntVOffEl2(u64);

impl CntVOffEl2 {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// # Virtual offset.
    /// If the Generic counter is implemented at a size less than 64 bits, then this field is permitted to be implemented at the same width as the counter, and the upper bits are RES0.
    /// The value of this field is treated as zero-extended in all counter calculations. 
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    pub const fn value(&self) -> u64 {
        self.0
    }
}

