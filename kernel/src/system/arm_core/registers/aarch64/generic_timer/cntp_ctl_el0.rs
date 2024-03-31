use core::arch::asm;

use mystd::bit_field;

use crate::system_register_impl;

system_register_impl!(cntp_ctl_el0 CntPCtlEl0 (r,w));

bit_field!(
    /// # CNTP_CTL_EL0, Counter-timer Physical Timer Control register
    /// Control register for the EL1 physical timer.
    pub CntPCtlEl0(u64) {
        
        /// The status of the timer. This bit indicates whether the timer condition is met:
        /// 
        /// * 0b0 Timer condition is not met.
        /// * 0b1 Timer condition is met.
        /// 
        /// When the value of the ENABLE bit is 1, ISTATUS indicates whether the timer condition is met. 
        /// ISTATUS takes no account of the value of the IMASK bit. If the value of ISTATUS is 1 and 
        /// the value of IMASK is 0 then the timer interrupt is asserted.
        2 => istatus,

        /// Timer interrupt mask bit. Permitted values are:
        /// 
        /// * 0b0 Timer interrupt is not masked by the IMASK bit.
        /// * 0b1 Timer interrupt is masked by the IMASK bit.
        /// 
        /// For more information, see the description of the ISTATUS bit.
        1 => imask,

        /// Enables the timer. Permitted values are:
        /// 
        /// * 0b0 Timer disabled.
        /// * 0b1 Timer enabled.
        /// 
        /// Setting this bit to 0 disables the timer output signal, but the timer value accessible from CNTP_TVAL_EL0 continues to count down.
        /// > ### Note
        /// > Disabling the output signal might be a power-saving option.
        0 => enable
});