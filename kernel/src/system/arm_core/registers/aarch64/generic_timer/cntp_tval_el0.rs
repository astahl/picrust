use core::arch::asm;

use mystd::bit_field;

use crate::system_register_impl;

system_register_impl!(cntp_tval_el0 CntPTValEl0 (r,w));

bit_field!(
    /// # CNTP_TVAL_EL0, Counter-timer Physical Timer TimerValue register
    /// Holds the timer value for the EL1 physical timer.
    pub CntPTValEl0(u64) {

    /// The TimerValue view of the EL1 physical timer.
    /// 
    /// On a read of this register:
    /// 
    /// * If CNTP_CTL_EL0.ENABLE is 0, the value returned is UNKNOWN.
    /// * If CNTP_CTL_EL0.ENABLE is 1, the value returned is (CNTP_CVAL_EL0 - CNTPCT_EL0).
    /// 
    /// On a write of this register, CNTP_CVAL_EL0 is set to (CNTPCT_EL0 + TimerValue), where TimerValue is treated as a signed 32-bit integer.
    /// 
    /// When CNTP_CTL_EL0.ENABLE is 1, the timer condition is met when (CNTPCT_EL0 - CNTP_CVAL_EL0) is greater than or equal to zero. This means that TimerValue acts like a 32-bit downcounter timer. When the timer condition is met:
    /// 
    /// * CNTP_CTL_EL0.ISTATUS is set to 1.
    /// * If CNTP_CTL_EL0.IMASK is 0, an interrupt is generated.
    /// 
    /// When CNTP_CTL_EL0.ENABLE is 0, the timer condition is not met, but CNTPCT_EL0 continues to count, so the TimerValue view appears to continue to count down.
    /// 
    /// > ### Note
    /// > The value of CNTPCT_EL0 used in these calculations is the value seen at the Exception Level that the CNTPCT_EL0 regsiter is being read or written from.
    31:0 => timer_value
});