use core::arch::asm;

use mystd::bit_field;

/// # C5.2.2 CurrentEL, Current Exception Level
/// Holds the current Exception level.
pub fn current_el() -> CurrentEl {
    let value: u64;
    unsafe { asm!("mrs {0}, CurrentEL", out(reg) value) };
    value.into()
}

bit_field!(
    /// Holds the current Exception level.
    pub CurrentEl(u64){

    /// Current Exception level.
    /// * 0b00 EL0.
    /// * 0b01 EL1.
    /// * 0b10 EL2.
    /// * 0b11 EL3.
    ///
    /// When the HCR_EL2.NV bit is 1, EL1 read accesses to the CurrentEL register return the value of
    /// 0b10 in this field.
    ///
    /// The reset behavior of this field is:
    /// * On a Warm reset:
    ///     - When the highest implemented Exception level is EL1, this field resets to 1.
    ///     - When the highest implemented Exception level is EL2, this field resets to 2.
    ///     - Otherwise, this field resets to 3.
    3:2 => el
});

impl Daif {
    #[inline]
    pub fn read_register() -> Self {
        let value: u64;
        unsafe { asm!("mrs {}, daif", out(reg) value); }
        value.into()
    }

    #[inline]
    pub fn write_register(&self) {
        unsafe { asm!("msr daif, {}", in(reg) self.0); }
    }
}

bit_field!(
    /// DAIF Interrupt Mask Bits
    pub Daif(u64) {
        /// Process state D mask.
        9 => debug_masked,
        /// SError interrupt mask bit.
        8 => serror_masked,
        /// IRQ mask bit.
        7 => irq_masked,
        /// FIQ mask bit.
        6 => fiq_masked,

});