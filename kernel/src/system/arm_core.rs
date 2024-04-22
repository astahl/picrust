pub mod features;
pub mod mmu;
pub mod registers;

use core::arch::asm;

use registers::aarch64::general_sys_ctrl;
use registers::aarch64::special_purpose;

use crate::print_init;

use self::registers::aarch64::general_sys_ctrl::rmr_elx::RmrElx;
pub use self::registers::aarch64::special_purpose::ExceptionLevel as ExceptionLevel;
pub use self::registers::aarch64::general_sys_ctrl::mpidr_el1::CoreId as CoreId;

pub fn get_core_num() -> CoreId {
    unsafe { general_sys_ctrl::mpidr_el1::MpidrEl1::read_register().cpu_id().value().unwrap_unchecked() }
}

pub fn current_exception_level() -> ExceptionLevel {
    unsafe { special_purpose::current_el().el().value().unwrap_unchecked() }
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
        ExceptionLevel::EL0 => panic!("Invalid EL to reset"),
        ExceptionLevel::EL1 => reset_req.write_register_el1(),
        ExceptionLevel::EL2 => reset_req.write_register_el2(),
        ExceptionLevel::EL3 => reset_req.write_register_el3(),
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

/// tries to wake up the secondary cores if they were not yet released by the firmware
pub fn wake_up_secondary_cores() {
    print_init!("Waking up secondary cores...");
    // try to wake up all other cores
    let start_fn = crate::_start as *const ();
    for core_i in 1..4 {
        if cfg!(feature = "raspi3b") {
            // https://forums.raspberrypi.com/viewtopic.php?t=209190
            let mbox_old_ptr = (0x4000008C + core_i * 0x10) as *mut u32;
            unsafe { mbox_old_ptr.write_volatile(start_fn as u32) };
            print_init!("Write {:#p} to {:#p} to wake core {}", start_fn, mbox_old_ptr, core_i);
        }

        // let mbox_ptr = (0x4c000008c_usize + core_i * 0x10) as *mut u32;
        // unsafe { mbox_ptr.write_volatile(start_fn as u32) };

        if cfg!(feature = "raspi4") {
            // https://forums.raspberrypi.com/viewtopic.php?t=273010
            let jmp_address_ptr = (0xe0 + (core_i - 1) * 0x8) as *mut u64;
            unsafe { jmp_address_ptr.write_volatile(start_fn as u64) };
            print_init!("Write {:#p} to {:#p} to wake core {}", start_fn, jmp_address_ptr, core_i);
        }
    }
    // sync and send event
    print_init!("Send event...");
    unsafe { core::arch::asm!("dsb ish", "isb", "SEV") };
    print_init!("Done");
}
