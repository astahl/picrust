pub mod counter;
pub mod mmu;

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