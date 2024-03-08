#![no_std]
#![no_main]

#[cfg(not(target_arch = "aarch64"))]
compile_error!("target arch not supported! Only aarch64 allowed!");

#[cfg(any(all(feature = "raspi4", feature = "raspi3b")))]
compile_error!("Can't compile for multiple Raspberry Pi Models.");

mod exception;
mod system;
mod tests;
use core::{arch::global_asm, str};
use system::hal;
use system::peripherals;
use system::peripherals::uart;

use crate::system::peripherals::uart::UART_0;

#[panic_handler]
fn on_panic(info: &core::panic::PanicInfo) -> ! {
    use core::fmt::Write;
    let mut uart = uart::UART_0;
    writeln!(&mut uart, "PANIC!");
    if let Some(msg) = info.payload().downcast_ref::<&str>() {
        writeln!(&mut uart, "{}", *msg);
    }
    if let Some(loc) = info.location() {
        writeln!(&mut uart, "source: {}", loc);
    }
    loop {
        // hal::led::status_blink_twice(100);
    }
}

extern "C" {
    static __font_start: u64;
    static __font_end: u64;
    static mut __kernel_end: u64;
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    system::initialize();
    tests::run();
    tests::test_dma();

    monitor::Monitor::new(|| UART_0.get_byte().unwrap_or(b'0'), |c| UART_0.put_byte(c)).run()
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
