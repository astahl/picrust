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

#[no_mangle]
pub extern "C" fn kernel_main() {
    clear_bss();

    use peripherals::uart::Uart0;
    let core = get_core_num() as usize;
    if let Some(framebuffer) = framebuffer::Framebuffer::new() {
        for y in 300..600 {
            for x in 400..800 {
                framebuffer.set_pixel_a8r8g8b8(x, y, 0xFF00FF00);
            }
        }
        framebuffer.set_pixel_a8r8g8b8(200, 200, 0xFFFFFFFF);
        framebuffer.set_pixel_a8r8g8b8(201, 201, 0x00FFFFFF);
        framebuffer.set_pixel_a8r8g8b8(202, 202, 0xFF00FFFF);
        framebuffer.set_pixel_a8r8g8b8(203, 203, 0xFFFF00FF);
        framebuffer.set_pixel_a8r8g8b8(204, 204, 0xFFFFFF00);
        framebuffer.set_pixel_a8r8g8b8(205, 205, 0xFFFFFFFF);
    }

    Uart0::init();
    Uart0::put_uint(core as u64);
    Uart0::puts("Hallo\n");

    let mut mon = monitor::Monitor::new(|| Uart0::get_byte().unwrap_or(b'0'), Uart0::putc);
    mon.run();

    loop {
        core::hint::spin_loop();
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
    //@ Set the stack pointer to start of executable code, grow down)
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