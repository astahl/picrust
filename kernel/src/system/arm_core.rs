pub mod features;
pub mod mmu;
pub mod registers;

use core::arch::asm;

use registers::aarch64::general_sys_ctrl;
use registers::aarch64::special_purpose;

use self::registers::aarch64::general_sys_ctrl::rmr_elx::RmrElx;

pub fn get_core_num() -> u64 {
    general_sys_ctrl::mpidr_el1::MpidrEl1::read_register().cpu_id().value()
}

pub fn current_exception_level() -> u64 {
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

pub fn reset() -> ! {
    let reset_req = RmrElx::zero().rr().set().aa64().set_value(general_sys_ctrl::rmr_elx::ArchSelect::AArch64);
    match current_exception_level() {
        1 => unsafe { asm!("hvc #0x1") },
        2 => unsafe { asm!("smc #0x1") },
        3 => reset_req.write_register_el3(),
        _ => panic!("Invalid EL to reset"),
    }
    loop {
        core::hint::spin_loop();
        //wait_for_event();
    }
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
            send_event()
        } else {
            loop {
                if FENCE.load(atomic::Ordering::SeqCst) == N_RESET {
                    break;
                }
                
                wait_for_event()
            } 
        }
    }
}
