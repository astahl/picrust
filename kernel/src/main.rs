#![no_std]
#![no_main]

#[cfg(not(target_arch = "aarch64"))]
compile_error!("target arch not supported! Only aarch64 allowed!");

#[cfg(any(all(feature = "raspi4", feature = "raspi3b")))]
compile_error!("Can't compile for multiple Raspberry Pi Models.");

mod exception;
mod system;
mod tests;
use core::arch::asm;
use core::arch::global_asm;
use mystd::io::Write;
use mystd::sync::signal::SleepError;
use mystd::sync::signal::SleepHandler;
use mystd::sync::signal::WakeError;
use system::arm_core;
use system::arm_core::wait_for_all_cores;
use system::hal;
use system::peripherals;
use system::peripherals::uart;

use crate::system::hal::signal::new_signal;

#[panic_handler]
fn on_panic(info: &core::panic::PanicInfo) -> ! {

    if cfg!(feature = "serial_uart") {
        let mut uart = uart::UART_0;
        let _ = writeln!(uart, "Doki Doki! {info}");
        loop {
            let _ = writeln!(uart, "press m for monitor, r to reset");
            match uart.get_byte() {
                Ok(b'm') => monitor::Monitor::new(uart, uart).run(),
                Ok(b'r') => {
                    writeln!(uart, "Resetting...").unwrap();
                    arm_core::reset();
                },
                _ => ()
            }
        }
    } else {
        loop {
            hal::led::status_blink_twice(1000);
        }
    }
}



#[no_mangle]
pub extern "C" fn main(core_id: usize) {
    static SIG: hal::signal::EventSignal = new_signal();
    match core_id {
        0 => {
            system::initialize();
            SIG.wake_all().expect("Yup");
        }
        _ => SIG.wait().expect("Should sleep"),
    }
    println_debug!("Continue.");
    match core_id {
        0 => {
            tests::test_irq();
            //tests::test_dma();
        }
        1 => {
            tests::test_screen();
        }
        _ => ()
    }
    println_debug!("{core_id} Waiting.");
    wait_for_all_cores();
    println_log!("{core_id} Done.");
    if core_id == 0 {
        panic!("Let's go monitor!")
    } else {
        loop {}
    }
    // tests::run();
    // tests::test_usb().expect("USB test should pass");
}

//global_asm!(".section .font", ".incbin \"901447-10.bin\"");

global_asm!(
    r#"
    .section ".text.boot"   // Make sure the linker puts this at the start of the kernel image
    .global _start          // Execution starts here
    _start:
        // Determine Stack Pointer for each core
        // Put processor ID into x0
        mrs     x0, mpidr_el1
        and     x0, x0, #3
        // We want the stack to start below our code
        ldr     x1, =_start
        lsr     x2, x1, #2 // scale the stack size: 1. divide by 4
        mul     x2, x2, x0 //                       2. multiply by the core id (0,1,2,3)
        sub     x1, x1, x2 // subtract the scaled Stack Size from the top of the stack
        // Ensure we end up on Exception Level 1 (starting on EL3 or EL2)
        b      _enter_el1
    "#
);

global_asm!(
    r#"
    // move execution level to EL1
    _enter_el1:
        // we make no assumptions if we're at EL3, EL2 or EL1
        // the current EL is coded numerically in CurrentEL bits 3 and 2
        mrs     x10, CurrentEL
        ubfx    x10, x10, #2, #2
        // are we running at EL3?
        cmp     x10, #3
        bne     5f
        // we are on EL3
        // stick core id into the thread id register
        msr     tpidr_el3, x0
        // set up exception handlers for EL2
        ldr     x2, =_vectors_el2
        msr     vbar_el2, x2
        // prepare to leave EL3
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
        cmp     x10, #1
        // yes, then jump to start_main
        beq     _start_main
        // no, we are still on EL2
        // stick core id into the thread id register
        msr     tpidr_el2, x0
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
        // 0x30d00800 = 0b0011_0000_1101_0000_0000_1000_0000_0000
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
        movz    x2, #0x0800
        movk    x2, #0x30d0, lsl #16
        msr     sctlr_el1, x2
        // set up exception handlers
        ldr     x2, =_vectors_el1
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
    // SUBROUTINE clear bss section
    _clear_bss:
        // w3 set to the bss size (size is mult of 8 bytes, bss is 64-bit aligned)
        ldr     w3, =__bss_size
        cbz     w3, 4f
        ldr     x2, =__bss_start
    3:
        // store the zero register to x2 and increment x2 by 8 bytes
        str     xzr, [x2], #8
        // decrement the remaining size by 1 and loop
        sub     w3, w3, #1
        cbnz    w3, 3b
    4:
        // return to _start_main
        ret
    "#
);


global_asm!(
    r#"
    _start_main:
        // stick core id into the thread id register
        msr     tpidr_el1, x0
        // set stack pointer to __main_stack
        mov     sp, x1
        bl      _clear_bss
        // Jump to our kernel main() routine in rust (make sure it doesn't return)
        bl      main
        // In case it does return, perform a reset
        //mov     x0, #3 // 0b11
        //msr     rmr_el1, x0
    5:
        wfe
        b 5b
    "#
);

