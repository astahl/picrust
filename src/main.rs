#![no_std]
#![no_main]

mod monitor;
mod peripherals;
mod framebuffer;

use core::arch::global_asm;

#[panic_handler]
fn on_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main() {
    use peripherals::uart;
    let core = get_core_num() as usize;
    if let Some(framebuffer) = framebuffer::Framebuffer::new() {
        for y in 300..600 {
            for x in 400..800 {
                framebuffer.set_pixel(x, y, 0xFF00FF00);
            }
        }
        framebuffer.set_pixel(200, 200, 0xFFFFFFFF);
        framebuffer.set_pixel(201, 201, 0x00FFFFFF);
        framebuffer.set_pixel(202, 202, 0xFF00FFFF);
        framebuffer.set_pixel(203, 203, 0xFFFF00FF);
        framebuffer.set_pixel(204, 204, 0xFFFFFF00);
        framebuffer.set_pixel(205, 205, 0xFFFFFFFF);
    }

    uart::init();
    uart::put_uint(core as u64);
    uart::puts("Hallo");

    let mut mon = monitor::Monitor::new(|| uart::get_byte().unwrap_or(b'0'), uart::putc);
    mon.run();
    loop {
        core::hint::spin_loop();
    }
}

fn get_core_num() -> u32 {
    let mut core_num: u32;
    unsafe {
        #[cfg(target_arch = "arm")]
        core::arch::asm!(
            "mrc p15, #0, {0}, c0, c0, #5",
            "and {0}, {0}, #3",
            out(reg) core_num
        );
    }
    core_num
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
    //@ Set the stack pointer to 0x8000 (start of executable code, grow down)
    "mov sp, #0x8000",
    //@ Clear the BSS segment (C statics) to 0
    "ldr r4, =__bss_start",
    "ldr r9, =__bss_end",
    "mov r5, #0",
    "mov r6, #0",
    "mov r7, #0",
    "mov r8, #0",
    "b       2f",
    "1: ",
    "stmia r4!, {{r5-r8}}",
    "2: ",
    "cmp r4, r9",
    "blo 1b",
    //@ Jump to kernel_main
    "ldr r3, =kernel_main",
    "blx r3",
    "halt:",
    "wfe",
    "b halt"
);
