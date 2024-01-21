#![no_std]
#![no_main]

mod buffer;
mod drawing;
mod exception;
mod hal;
mod monitor;
mod peripherals;
mod system;
use core::{arch::global_asm, str, usize};

use crate::{hal::display::Resolution, peripherals::uart::Uart0Formatter, system::wait_msec};

#[panic_handler]
fn on_panic(info: &core::panic::PanicInfo) -> ! {
    use peripherals::uart::Uart0;
    Uart0::puts("PANIC!");
    if let Some(msg) = info.payload().downcast_ref::<&str>() {
        Uart0::puts(msg);
    }
    if let Some(loc) = info.location() {
        Uart0::puts(loc.file());
        Uart0::put_uint(loc.line() as u64);
    }
    loop {
        hal::led::status_blink_twice(100);
    }
}

extern "C" {
    static __rodata_start: usize;
    static __rodata_end: usize;
    static __font_start: u64;
    static __font_end: u64;
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    use peripherals::uart::Uart0;
    // let core_id = system::get_core_num();
    // if core_id == 0 {
    Uart0::init();
    Uart0::puts("start");
    hal::led::status_set(true);
    hal::led::status_blink_twice(500);
    if cfg!(not(feature = "bcm2711")) {
        system::mmu_init().unwrap();
    }

    Uart0::put_uint(system::current_exception_level() as u64);
    // Uart0::puts("start");

    let mut str_buffer = buffer::Ring::<u8>::new();

    use hal::framebuffer::color;
    let resolution = hal::display::Resolution::preferred().unwrap_or_default();

    let fb = hal::framebuffer::Framebuffer::new(
        resolution.horizontal as u32,
        resolution.vertical as u32,
    )
    .unwrap();

    // fb.clear(color::BLACK);

    let font = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(__font_start),
            core::ptr::addr_of!(__font_end)
                .offset_from(core::ptr::addr_of!(__font_start))
                .unsigned_abs(),
        )
    };

    let text = b" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
    let mapping = |c: u8| -> u8 {
        match c {
            0 => b' ',
            b' '..=b'?' => c,
            b'@'..=b'^' => c as u8 - b'@',
            b'a'..=b'z' => c as u8 - b'`' | 0x80,
            b'{' => b'<',
            b'}' => b'>',
            b'_' => 82,
            _ => 255,
        }
    };
    fb.clear(color::BLUE);
    fb.write_text(text, font, mapping);
    wait_msec(1000);
    fb.clear(color::RED);

    use core::fmt::Write;
    let mut supported_resolutions = [Resolution::default(); 128];
    let count = hal::display::Resolution::supported(supported_resolutions.as_mut_slice(), 0);
    writeln!(
        str_buffer,
        "Supported {:?}",
        supported_resolutions.get(0..count)
    )
    .unwrap();
    writeln!(str_buffer, "Requested Resolution {:?}", resolution).unwrap();
    writeln!(
        str_buffer,
        "Framebuffer: {} {} {}",
        fb.width_px, fb.height_px, fb.bits_per_pixel
    )
    .unwrap();
    if let Some(arm_memory) = hal::info::get_arm_memory() {
        writeln!(str_buffer, "ARM {}", arm_memory).unwrap();
    }
    if let Some(vc_memory) = hal::info::get_vc_memory() {
        writeln!(str_buffer, "VC {}", vc_memory).unwrap();
    }
    // if let Some(board_info) = hal::info::get_board_info() {
    //     writeln!(str_buffer, "{}", board_info.revision).unwrap();
    // }
    // if let Some(mac) = hal::info::get_mac_address() {
    //     writeln!(str_buffer, "MAC {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]).unwrap();
    // }

    for edid in hal::display::EdidIterator::new() {
        writeln!(str_buffer, "EDID BLOCK {:#?}", edid).unwrap();
        // for byte in edid.bytes() {
        //     write!(str_buffer, "{:02X} ", byte).unwrap();
        // }
    }
    writeln!(str_buffer, "Bye!").unwrap();
    let (text, _) = str_buffer.as_slices();
    fb.clear(color::BLACK);
    fb.write_text(text, font, mapping);

    Uart0::puts(core::str::from_utf8(text).unwrap());
    // Uart0::put_uint(core as u64);
    // Uart0::puts("Hallo\n");
    //
    let mut mon = monitor::Monitor::new(|| Uart0::get_byte().unwrap_or(b'0'), Uart0::putc);
    mon.run()
    // fb.set_pixel_a8b8g8r8(150, 100, color::WHITE);
    // let mut canvas = drawing::PixelCanvas::with_slice(
    //     fb.width_px as usize,
    //     fb.height_px as usize,
    //     fb.pitch_bytes as usize / 4,
    //     fb.as_mut_pixels(),
    // )
    // .unwrap();
    // //canvas.clear(color::BLUE);
    // canvas
    //     .fill_rect(color::BLUE, (298, 298), (300, 300))
    //     .unwrap();
    // canvas.fill_lines(color::RED, 100..=100).unwrap();
    // let pixelscale = (2, 2);
    // let cols = canvas.width / (pixelscale.0 * 8);
    // let rows = canvas.height / (pixelscale.1 * 8);
    // let mut row_buffer = [0_u64; 256];
    // let mut v_scroll: usize = 0;
    // hal::led::status_set(false);
    // loop {
    //     let line_iterator = text
    //         .split(|b| *b == b'\n')
    //         .flat_map(|l| l.chunks(cols))
    //         .cycle();
    //     canvas.fill_rect(0, (0, 0), (cols * 8, rows * 8)).unwrap();
    //     for (row_nr, text_line) in line_iterator.skip(v_scroll as usize).take(rows).enumerate() {
    //         let mut pre = 0;
    //         let mut len = 0;
    //         for (dst, src) in row_buffer.iter_mut().zip(text_line) {
    //             let val = font[mapping(*src) as usize];
    //             if len == 0 && val == 0 {
    //                 pre += 1;
    //                 continue;
    //             }
    //             *dst = val;
    //             len += 1;
    //         }
    //         canvas
    //             .blit8x8_line(
    //                 &row_buffer[pre..len + pre],
    //                 color::WHITE,
    //                 color::BLACK,
    //                 (pre * 8, row_nr * 8),
    //             )
    //             .unwrap();
    //     }
    //     canvas.scale_in_place(pixelscale.0, pixelscale.1);
    //     v_scroll += 1;

    //     system::wait_msec(100);
    // }
}

global_asm!(".section .font", ".incbin \"901447-10.bin\"");

#[cfg(target_arch = "aarch64")]
global_asm!(
    ".section \".text.boot\"", // Make sure the linker puts this at the start of the kernel image
    ".global _start",          // Execution starts here
    "_start:",
    // Check processor ID is zero (executing on main core), else hang
    "mrs     x1, mpidr_el1",
    "and     x1, x1, #3",
    "cbz     x1, 2f",
    "stop_core:", // We're not on the main core, so hang in an infinite wait loop
    // wait for event and loop
    "wfe",
    "bl       stop_core",
    "2:", // We're on the main core!
    // Set stack to start below our code
    "ldr     x1, =__main_stack",
    // Ensure we end up on Exception Level 1 (starting on EL3)
    "bl enter_el1",
);

#[cfg(not(feature = "bcm2711"))]
global_asm!(
    "enter_el1:",
    // set up EL1
    "mrs     x0, CurrentEL",
    "and     x0, x0, #12", // clear reserved bits
    // running at EL3?
    "cmp     x0, #12",
    "bne     5f",
    // should never be executed, just for completeness
    "mov     x2, #0x5b1",
    "msr     scr_el3, x2",
    "mov     x2, #0x3c9",
    "msr     spsr_el3, x2",
    "adr     x2, 5f",
    "msr     elr_el3, x2",
    "eret",
    // running at EL2?
    "5:  cmp     x0, #4",
    "beq     start_main",
    "msr     sp_el1, x1",
    // enable CNTP for EL1
    "mrs     x0, cnthctl_el2",
    "orr     x0, x0, #3",
    "msr     cnthctl_el2, x0",
    "msr     cntvoff_el2, xzr",
    // enable SIMD/FP in EL1 https://stackoverflow.com/questions/46194098/armv8-changing-from-el3-to-el1-secure#46219711
    "mov     x0, #(3 << 20)",
    "msr     cpacr_el1, x0",
    // enable AArch64 in EL1
    "mov     x0, #(1 << 31)",    // AArch64
    "orr     x0, x0, #(1 << 1)", // SWIO hardwired on Pi3
    "msr     hcr_el2, x0",
    "mrs     x0, hcr_el2",
    // Setup SCTLR access
    "mov     x2, #0x0800",
    "movk    x2, #0x30d0, lsl #16",
    "orr    x2, x2, #(1 << 12)", // enable I-Cache
    "orr    x2, x2, #(1 << 2)",  // enable D-Cache
    "msr     sctlr_el1, x2",
    // set up exception handlers
    "ldr     x2, =_vectors",
    "msr     vbar_el1, x2",
    // change execution level to EL1
    "mov     x2, #0x3c4",
    "msr     spsr_el2, x2",
    "adr     x2, start_main",
    "msr     elr_el2, x2",
    "eret",
);

#[cfg(feature = "bcm2711")]
global_asm!(
    "enter_el1:",
    "mov     x0, #0x33ff",
    "msr     cptr_el3, x0", // Disable coprocessor traps to EL3
    "mov     x0, #3 << 20",
    "msr     cpacr_el1, x0", // Enable FP/SIMD at EL1
    // Now get ready to switch from EL3 down to EL1
    "mov     x0, #0x0800",          // reserved bits for sctlr
    "movk    x0, #0x30d0, lsl #16", // reserved bits for sctlr
    "orr    x0, x0, #(1 << 12)",    // enable I-Cache
    "orr    x0, x0, #(1 << 2)",     // enable D-Cache
    // keep bit0 0 to disable MMU
    "msr	sctlr_el1, x0",
    "mov     x0, #(1 << 31)", // Hypervisor RW
    "msr     hcr_el2, x0",
    "mov     x0, #(3 << 4)",      // reserved bits for SCR
    "orr     x0, x0, #(1 << 10)", // RW
    "orr     x0, x0, #(1 << 0)",  // NS
    "msr     scr_el3, x0",
    "mov     x0, #(7 << 6)",     // mask all
    "orr     x0, x0, #(1 << 0)", // EL1h
    "orr     x0, x0, #(1 << 2)", // EL1h
    "msr     spsr_el3, x0",
    // set up exception handlers
    "ldr     x2, =_vectors",
    "msr     vbar_el1, x2",
    "adr     x0, start_main",
    "msr     elr_el3, x0",
    "eret",
);

global_asm!(
    "start_main:",
    "mov     sp, x1",
    // clear bss section
    "ldr    x1, =__bss_start",
    // initialize w2 to the remaining size (size is mult of 8 bytes, bss is aligned)
    "ldr    w2, =__bss_size",
    // if the w2 is zero, we're done
    "3: cbz w2, 4f",
    // store the zero register to x1 and increment x1 by 8 bytes
    "str xzr, [x1], #8",
    // decrement the remaining size by 1 and loop
    "sub w2, w2, #1",
    "cbnz    w2, 3b",
    "4:",
    // Jump to our kernel_main() routine in rust (make sure it doesn't return)
    "bl kernel_main",
    // In case it does return, halt the master core too
    "bl stop_core",
);

global_asm!(
    // important, code has to be properly aligned
    ".align 11",
    "_vectors:",
    // synchronous
    ".align  7",
    "mov     x0, #0",
    "mrs     x1, esr_el1",
    "mrs     x2, elr_el1",
    "mrs     x3, spsr_el1",
    "mrs     x4, far_el1",
    "b       exc_handler",
    // IRQ
    ".align  7",
    "mov     x0, #1",
    "mrs     x1, esr_el1",
    "mrs     x2, elr_el1",
    "mrs     x3, spsr_el1",
    "mrs     x4, far_el1",
    "b       exc_handler",
    // FIQ
    ".align  7",
    "mov     x0, #2",
    "mrs     x1, esr_el1",
    "mrs     x2, elr_el1",
    "mrs     x3, spsr_el1",
    "mrs     x4, far_el1",
    "b       exc_handler",
    // SError
    ".align  7",
    "mov     x0, #3",
    "mrs     x1, esr_el1",
    "mrs     x2, elr_el1",
    "mrs     x3, spsr_el1",
    "mrs     x4, far_el1",
    "b       exc_handler",
);
