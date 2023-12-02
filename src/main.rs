#![no_std]
#![no_main]

mod framebuffer;
mod monitor;
mod peripherals;

use core::arch::global_asm;

#[panic_handler]
fn on_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern "C" {
    static __bss_start: usize;
    static __bss_end: usize;
}


fn clear_bss() {
    unsafe {
        for address in __bss_start..__bss_end {
            *(address as *mut u8) = 0;
        }
    }
}


fn initialize_global() {
    clear_bss();
    peripherals::led_on();
}

#[no_mangle]
pub extern "C" fn kernel_main() {
    let core_num = get_core_num();
    match core_num {
        0 => initialize_global(),
        _ => {}
    }

    if let Some(framebuffer) = framebuffer::Framebuffer::new() {
        for y in 400..800 {
            for x in 400..800 {
                let x = x + (core_num * 400) as u32;
                framebuffer.set_pixel_a8r8g8b8(x, y, 0xFF00FF00);
                crate::peripherals::delay(100000);
            }
        }
    }
    
    use peripherals::uart::Uart0;
    Uart0::init();
    // Uart0::put_uint(core as u64);
    Uart0::puts("Hallo\n");
    // peripherals::led_off();
    // let mut mon = monitor::Monitor::new(|| Uart0::get_byte().unwrap_or(b'0'), Uart0::putc);
    // mon.run();

    loop {
        core::hint::spin_loop();
        if core_num == 0 {
            peripherals::blink_led();
        }
    }
}

fn get_core_num() -> usize {
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

#[cfg(target_arch = "arm")]
global_asm!(
    ".section \".text.boot\"",
    ".global _start",
    "_start:",
    //@ Get core id and halt if not 0 (stop all but one threads)
    "mrc p15, #0, r1, c0, c0, #5",
    "and r1, r1, #3",
    "cmp r1, #0",
    "bne halt",
    // //@ Set the stack pointer to start of executable code, grow down)
    "ldr r1, =_start",
    "mov sp, r1",
    //@ Jump to kernel_main
    "ldr r3, =kernel_main",
    "blx r3",
    "halt:",
    "wfe",
    "b halt"
);


#[cfg(target_arch = "aarch64")]
global_asm!(
".section \".text.boot\"",  // Make sure the linker puts this at the start of the kernel image
".global _start",  // Execution starts here
"_start:",
    // Check processor ID is zero (executing on main core), else hang
    "mrs     x1, mpidr_el1",
    "and     x1, x1, #3",
    "cbz     x1, 2f",
    // We're not on the main core, so hang in an infinite wait loop
"1:  wfe",
    "b       1b",
"2:",  // We're on the main core!

    // Set stack to start below our code
    "ldr     x1, =_start",
    "mov     sp, x1",
    
    // Jump to our main() routine in C (make sure it doesn't return)
"4:  bl      kernel_main",
    // In case it does return, halt the master core too
    "b       1b"
);