
pub fn get_core_num() -> usize {
    let mut core_num: usize;
    unsafe {
        #[cfg(target_arch = "arm")]
        core::arch::asm!(
            "mrc p15, #0, {0}, c0, c0, #5",
            out(reg) core_num
        );
        #[cfg(target_arch = "aarch64")]
        core::arch::asm!(
            "mrs {0}, mpidr_el1",
            out(reg) core_num
        );
    }
    core_num & 0b11
}

pub fn current_exception_level() -> usize {
    let mut el: usize;
    unsafe {
        core::arch::asm!(
            "mrs {0}, CurrentEL",
            out(reg) el
        );
    }
    return (el >> 2) & 0b11;
}

pub fn wait_msec(msec: usize) {
    let mut frequency: usize;
    let mut current_counter: usize;
    unsafe {
        core::arch::asm!(
            "mrs {0}, cntfrq_el0",
            "mrs {1}, cntpct_el0",
            out(reg) frequency, out(reg) current_counter
        );
    }
    let expire_at = current_counter + ((frequency / 1000) * msec);
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
