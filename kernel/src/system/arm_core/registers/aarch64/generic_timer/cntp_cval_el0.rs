use core::arch::asm;

use mystd::bit_field;

use crate::system_register_impl;

system_register_impl!(cntp_cval_el0 CntPCValEl0 (r,w));

bit_field!(
    /// # CNTP_CVAL_EL0, Counter-timer Physical Timer CompareValue register
    /// Holds the compare value for the EL1 physical timer.
    pub CntPCValEl0(u64) {

    /// Holds the EL1 physical timer CompareValue.
    /// 
    /// When CNTP_CTL_EL0.ENABLE is 1, the timer condition is met when (CNTPCT_EL0 - CompareValue) is greater than or equal to zero. This means that CompareValue acts like a 64-bit upcounter timer. When the timer condition is met:
    /// 
    /// * CNTP_CTL_EL0.ISTATUS is set to 1.
    /// * If CNTP_CTL_EL0.IMASK is 0, an interrupt is generated.
    /// 
    /// When CNTP_CTL_EL0.ENABLE is 0, the timer condition is not met, but CNTPCT_EL0 continues to count.
    /// 
    /// If the Generic counter is implemented at a size less than 64 bits, then this field is permitted to be implemented at the same width as the counter, and the upper bits are RES0.
    /// The value of this field is treated as zero-extended in all counter calculations.
    63:0 => compare_value
});