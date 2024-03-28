pub mod counter;
pub mod features;
pub mod mmu;
pub mod registers;

use core::arch::asm;
use core::usize;

use registers::aarch64::general_sys_ctrl;
use registers::aarch64::special_purpose;

pub fn get_core_num() -> usize {
    general_sys_ctrl::mpidr_el1::read().cpu_id().value()
}

pub fn current_exception_level() -> usize {
    special_purpose::current_el().el().value()
}

pub fn wait_for_event() {
    unsafe { asm!("wfe") }
}

pub fn send_event() {
    unsafe { asm!("SEV") }
}

pub fn send_event_local() {
    unsafe { asm!("SEVL") }
}

pub fn wait_for_interrupt() {
    unsafe { asm!("wfi") }
}

pub fn stop_core() -> ! {
    loop { wait_for_event() }
}

pub fn wait_for_all_cores() {
    use core::sync::atomic;
    const N_RESET: u8 = 4;
    static mut FENCE: atomic::AtomicU8 = atomic::AtomicU8::new(N_RESET);
    unsafe {
        let Ok(i) = FENCE.fetch_update(atomic::Ordering::SeqCst, atomic::Ordering::SeqCst, |n| {
            match n {
                // last one to reach resets to N
                1 => Some(N_RESET),
                // every other decrements
                _ => Some(n - 1)
            }
        }) else {
            panic!()
        };
        if i == 1 {
            send_event_local()
        } else {
            while FENCE.load(atomic::Ordering::SeqCst) != N_RESET {
                wait_for_event()
            } 
        }
    }
}
