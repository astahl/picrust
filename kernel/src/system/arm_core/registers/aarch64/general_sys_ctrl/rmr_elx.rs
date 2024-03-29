use core::arch::asm;

use mystd::bit_field;

impl RmrElx {
    pub fn write_register_el1(&self) {
        unsafe { asm!("msr rmr_el1, {}", in(reg) self.0) }
    }

    pub fn write_register_el2(&self) {
        unsafe { asm!("msr rmr_el2, {}", in(reg) self.0) }
    }

    pub fn write_register_el3(&self) {
        unsafe { asm!("msr rmr_el3, {}", in(reg) self.0) }
    }
}


bit_field!(pub RmrElx(u64) {
    /// Request Reset
    1 => rr,
    /// Execution Level Architecture selection after reset
    0 => aa64: enum ArchSelect {
        AArch32,
        AArch64
    },
});