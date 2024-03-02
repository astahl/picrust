#![no_std]
#![no_main]

#[cfg(not(target_arch = "aarch64"))]
compile_error!("target arch not supported! Only aarch64 allowed!");

#[cfg(any(all(feature = "raspi4", feature = "raspi3b")))]
compile_error!("Can't compile for multiple Raspberry Pi Models.");

mod exception;
mod system;
use core::{arch::global_asm, str, usize};
use mystd::buffer;
use system::hal;
use system::hal::display::Resolution;
use system::peripherals;

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
        // hal::led::status_blink_twice(100);
    }
}

extern "C" {
    static __rodata_start: usize;
    static __rodata_end: usize;
    static __font_start: u64;
    static __font_end: u64;
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    use peripherals::uart::Uart0;
    // let core_id = system::get_core_num();
    // if core_id == 0 {
    Uart0::init();
    Uart0::puts("start");
    let status_led = hal::led::Led::Status;
    //status_led.blink_pattern(0b10101001, core::time::Duration::from_millis(100));
    if cfg!(feature = "mmu") {
        system::mmu_init().unwrap();
    }

    Uart0::put_uint(system::current_exception_level() as u64);
    // Uart0::puts("start");

    let mut str_buffer = buffer::RingArray::<u8, 1024>::new();

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
            b'\n' => b' ', // TODO better handle newlines in the buffer writer
            b'_' => 82,
            _ => 255,
        }
    };
    fb.clear(color::BLUE);
    fb.write_text(text, font, mapping);

    hal::led::status_blink_twice(500);
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
        writeln!(str_buffer, "EDID BLOCK {:?}", edid).unwrap();
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
    status_led.off();
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

global_asm!(
    r#"
    .section ".text.boot"   // Make sure the linker puts this at the start of the kernel image
    .global _start          // Execution starts here
    _start:
        // Check processor ID is zero (executing on main core), else hang
        mrs     x1, mpidr_el1
        and     x1, x1, #3
        cbz     x1, 2f
    _stop_core: 
        // We're not on the main core, so hang in an infinite wait loop
        // wait for event and loop
        wfe
        bl      _stop_core
    2: 
        // We're on the main core!
        // We want the stack to start below our code
        ldr     x1, =__main_stack
        // Ensure we end up on Exception Level 1 (starting on EL3 or EL2)
        bl      _enter_el1
    "#
);

global_asm!(
    r#"
    // move execution level to EL1
    _enter_el1:
        // we make no assumptions if we're at EL3, EL2 or EL1
        // the current EL is coded numerically in CurrentEL bits 3 and 2
        mrs     x0, CurrentEL
        ubfx    x0, x0, #2, #2
        // are we running at EL3?
        cmp     x0, #3
        bne     5f
        // we are on EL3
        mov     x2, #0x5b1
        msr     scr_el3, x2
        mov     x2, #0x3c9
        msr     spsr_el3, x2
        // leave execution level 3 and continue execution at the label below
        adr     x2, 5f
        msr     elr_el3, x2
        eret
    5:
        // are we already running at EL1?
        cmp     x0, #1
        // yes, then jump to start_main
        beq     _start_main
        // no, we are still on EL2
        // set the EL1 stack pointer to __main_stack
        msr     sp_el1, x1
        // enable CNTP for EL1
        mrs     x2, cnthctl_el2
        orr     x2, x2, #3
        msr     cnthctl_el2, x2
        msr     cntvoff_el2, xzr
        // enable SIMD/FP in EL1 https://stackoverflow.com/questions/46194098/armv8-changing-from-el3-to-el1-secure#46219711
        mov     x2, #(3 << 20)
        msr     cpacr_el1, x2
        // enable AArch64 in EL1
        mov     x2, #(1 << 31)    // AArch64
        orr     x2, x2, #(1 << 1) // SWIO hardwired on Pi3
        msr     hcr_el2, x2
        mrs     x2, hcr_el2 // todo what does this read do??
        // System Control Register EL1, reset value on Cortex A72 is 0x00C50838
        // 0x30d01804 = 0b0011_0000_1101_0000_0001_1000_0000_0100
        //                  ^^      ^^ ^         ^ ^          ^
        //                   |      || |         | |          C, bit [2] = Stage 1 Cacheability control, for data access
        //                   |      || |         | EOS, bit [11] = When FEAT_ExS is implemented, else RES1
        //                   |      || |         |                 Exception Exit is Context Synchronizing.
        //                   |      || |         |                 0b0 An exception return from EL1 is not a context synchronizing event
        //                   |      || |         |                 0b1 An exception return from EL1 is a context synchronizing event
        //                   |      || |         I, bit [12] = Stage 1 instruction access Cacheability control, for accesses at EL0 and EL1
        //                   |      || TSCXT, bit [20] = When FEAT_CSV2_2 is implemented or FEAT_CSV2_1p2 is implemented, else RES1
        //                   |      ||                   Trap EL0 Access to the SCXTNUM_EL0 register, when EL0 is using AArch64.
        //                   |      ||                   0b0 EL0 access to SCXTNUM_EL0 is not disabled by this mechanism.
        //                   |      ||                   0b1 EL0 access to SCXTNUM_EL0 is disabled, causing an exception to EL1, or to EL2
        //                   |      |EIS, bit [22] = When FEAT_ExS is implemented, else RES1
        //                   |      |                Exception Entry is Context Synchronizing.
        //                   |      |                0b0 The taking of an exception to EL1 is not a context synchronizing event.
        //                   |      |                0b1 The taking of an exception to EL1 is a context synchronizing event.
        //                   |      SPAN, bit [23] = When FEAT_PAN is implemented, else RES1
        //                   |                       Set Privileged Access Never, on taking an exception to EL1.
        //                   |                       0b0 PSTATE.PAN is set to 1 on taking an exception to EL1.
        //                   |                       0b1 The value of PSTATE.PAN is left unchanged on taking an exception to EL1.
        //                   nTLSMD, bit [28] = When FEAT_LSMAOC is implemented, else RES1
        //                                      No Trap Load Multiple and Store Multiple to Device-nGRE/Device-nGnRE/Device-nGnRnE memory.
        //                                      0b0 All memory accesses by A32 and T32 Load Multiple and Store Multiple at EL0 that are marked at stage 1 as Device-nGRE/Device-nGnRE/Device-nGnRnE memory are trapped and generate a stage 1 Alignment fault.
        //                                      0b1 All memory accesses by A32 and T32 Load Multiple and Store Multiple at EL0 that are marked at stage 1 as Device-nGRE/Device-nGnRE/Device-nGnRnE memory are not trapped.
        movz    x2, #0x1804
        movk    x2, #0x30d0, lsl #16
        msr     sctlr_el1, x2
        // set up exception handlers
        ldr     x2, =_vectors
        msr     vbar_el1, x2
        mov     x2, #0x3c4
        msr     spsr_el2, x2
        // leave execution level 2 to start main
        adr     x2, _start_main
        msr     elr_el2, x2
        eret
    "#
);

global_asm!(
    r#"
    _start_main:
        // set stack pointer to __main_stack
        mov     sp, x1
        // clear bss section
        ldr     x1, =__bss_start
        // initialize w2 to the remaining size (size is mult of 8 bytes, bss is 64-bit aligned)
        ldr     w2, =__bss_size
    3:
        // if the w2 is zero, we're done
        cbz     w2, 4f
        // store the zero register to x1 and increment x1 by 8 bytes
        str     xzr, [x1], #8
        // decrement the remaining size by 1 and loop
        sub     w2, w2, #1
        cbnz    w2, 3b
    4:
        // Jump to our kernel_main() routine in rust (make sure it doesn't return)
        bl      main
        // In case it does return, halt the master core too
        bl      _stop_core
    "#
);

global_asm!(
    r#"
    // important, code has to be properly aligned
    .align 11
    _vectors:
    // synchronous
    .align  7
        mov     x0, #0
        mrs     x1, esr_el1
        mrs     x2, elr_el1
        mrs     x3, spsr_el1
        mrs     x4, far_el1
        b       exc_handler
    // IRQ
    .align  7
        mov     x0, #1
        mrs     x1, esr_el1
        mrs     x2, elr_el1
        mrs     x3, spsr_el1
        mrs     x4, far_el1
        b       exc_handler
    // FIQ
    .align  7
        mov     x0, #2
        mrs     x1, esr_el1
        mrs     x2, elr_el1
        mrs     x3, spsr_el1
        mrs     x4, far_el1
        b       exc_handler
    // SError
    .align  7
        mov     x0, #3
        mrs     x1, esr_el1
        mrs     x2, elr_el1
        mrs     x3, spsr_el1
        mrs     x4, far_el1
        b       exc_handler
    "#
);
