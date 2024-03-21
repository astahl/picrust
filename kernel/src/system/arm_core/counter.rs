use core::time::Duration;

pub fn wait(duration: Duration) {
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
