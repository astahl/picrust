use core::time::Duration;

use super::registers::aarch64::generic_timer;

/// Returns the counter frequency in Hz
pub fn frequency() -> u32 {
    generic_timer::cntfrq_el0::CntFrqEl0::read_register().clock_frequency().value() as u32
}


pub fn spin_wait_for(duration: Duration) {
    let mut frequency: usize;
    let mut current_counter: usize;
    unsafe {
        core::arch::asm!(
            "mrs {0}, cntfrq_el0",
            "mrs {1}, cntpct_el0",
            out(reg) frequency, out(reg) current_counter
        );
    }
    let expire_at =
        current_counter + ((frequency as u128 * duration.as_micros()) / 1_000_000) as usize;
    while current_counter < expire_at {
        unsafe {
            core::arch::asm!(
                "mrs {0}, cntpct_el0",
                out(reg) current_counter
            );
        }
        core::hint::spin_loop();
    }
}

pub fn wait_cycles(mut count: usize) {
    while count > 0 {
        count -= 1;
        core::hint::spin_loop();
    }
}
