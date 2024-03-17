use core::arch::asm;

use mystd::bit_field;

/// # C5.2.2 CurrentEL, Current Exception Level
/// Holds the current Exception level.
pub fn current_el() -> CurrentEl {
    let value: usize;
    unsafe { asm!("mrs {0}, CurrentEL", out(reg) value) };
    value.into()
}

bit_field!(
    /// Holds the current Exception level.
    pub CurrentEl(usize){

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