use core::arch::asm;

use mystd::bit_field;

use crate::system_register_impl;

system_register_impl!(cntfrq_el0 CntFrqEl0 (r));

bit_field!(
    /// # CNTFRQ_EL0, Counter-timer Frequency register
    /// This register is provided so that software can discover the frequency of the system counter. It must be programmed with this value as part of system initialization. The value of the register is not interpreted by hardware.
    pub CntFrqEl0(u64) {
    /// Clock frequency. Indicates the system counter clock frequency, in Hz.
    31:0 => clock_frequency
});