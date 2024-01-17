#![no_std]
#![no_main]

mod buffer;
mod drawing;
mod hal;
mod monitor;
mod peripherals;
mod system;
use core::{arch::global_asm, usize};

use crate::hal::display::Resolution;

#[panic_handler]
fn on_panic(info: &core::panic::PanicInfo) -> ! {
    use peripherals::uart::Uart0;
    if let Some(msg) = info.payload().downcast_ref::<&str>() {
        Uart0::puts("PANIC: ");
        Uart0::puts(msg);
    } else {
        Uart0::puts("PANIC!");
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
pub extern "C" fn kernel_main() {
    use peripherals::uart::Uart0;
    let core_id = system::get_core_num();
    if core_id == 0 {
        hal::led::status_set(true);
        Uart0::init();
    }
    Uart0::put_uint(system::current_exception_level() as u64);
    Uart0::puts("start");
    let mut str_buffer = buffer::Ring::<u8>::new();

    use hal::framebuffer::color;
    let resolution = hal::display::Resolution::preferred().unwrap_or_default();

    let fb = hal::framebuffer::Framebuffer::new(
        resolution.horizontal as u32,
        resolution.vertical as u32,
    )
    .unwrap();

    fb.clear(color::BLACK);

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
    // let mut mon = monitor::Monitor::new(|| Uart0::get_byte().unwrap_or(b'0'), Uart0::putc);
    // mon.run();
    fb.set_pixel_a8b8g8r8(150, 100, color::WHITE);
    let mut canvas = drawing::PixelCanvas::with_slice(
        fb.width_px as usize,
        fb.height_px as usize,
        fb.pitch_bytes as usize / 4,
        fb.as_mut_pixels(),
    )
    .unwrap();
    //canvas.clear(color::BLUE);
    canvas
        .fill_rect(color::BLUE, (298, 298), (300, 300))
        .unwrap();
    canvas.fill_lines(color::RED, 100..=100).unwrap();
    let pixelscale = (5, 6);
    let cols = canvas.width / (pixelscale.0 * 8);
    let rows = canvas.height / (pixelscale.1 * 8);
    let mut row_buffer = [0_u64; 256];
    let mut v_scroll: usize = 0;
    hal::led::status_set(false);
    loop {
        let line_iterator = text
            .split(|b| *b == b'\n')
            .flat_map(|l| l.chunks(cols))
            .cycle();
        canvas.fill_rect(0, (0, 0), (cols * 8, rows * 8)).unwrap();
        for (row_nr, text_line) in line_iterator.skip(v_scroll as usize).take(rows).enumerate() {
            let mut pre = 0;
            let mut len = 0;
            for (dst, src) in row_buffer.iter_mut().zip(text_line) {
                let val = font[mapping(*src) as usize];
                if len == 0 && val == 0 {
                    pre += 1;
                    continue;
                }
                *dst = val;
                len += 1;
            }
            canvas
                .blit8x8_line(
                    &row_buffer[pre..len + pre],
                    color::WHITE,
                    color::BLACK,
                    (pre * 8, row_nr * 8),
                )
                .unwrap();
        }
        canvas.scale_in_place(pixelscale.0, pixelscale.1);
        v_scroll += 1;

        system::wait_msec(500);
    }
}

#[derive(Debug)]
#[repr(C)]
pub enum ExceptionType {
    Synchronous = 0_isize,
    IRQ,
    FIQ,
    SError,
}

#[derive(Debug)]
pub enum ExceptionClass {
    Unknown = 0b000000,
    TrappedWFxInstructionExecution = 0b000001,
    Reserved0x02,
    TrappedMCROrMRCAccessCoproc0xF = 0b000011,
    TrappedMCRROrMRRCAccess = 0b000100,
    TrappedMCROrMRCAccessCoproc0xE = 0b000101,
    TrappedLDCOrSTCAccess = 0b000110,
    TrappedFpSMEAdvancedSIMDOrSVE = 0b000111,
    Reserved0x08,
    Reserved0x09,
    TrappedLD64bOrST64bInstruction = 0b001010,
    Reserved0x0c,
    TrappedMRRCAcessCoproc0xE = 0b001100,
    BranchTargetException = 0b001101,
    IllegalExecutionState = 0b001110,
    Reserved0x11,
    TrappedSVCInstructionAArch32 = 0b010001,
    Reserved0x12,
    Reserved0x13,
    Reserved0x14,
    TrappedSVCInstructionAArch64 = 0b010101,
    Reserved0x16,
    Reserved0x17,
    TrappedMSROrMRSOrSystemInstruction = 0b011000,
    TrappedSVEAccess = 0b011001,
    Reserved0x1a,
    ExceptionFromTSTARTInstruction = 0b011011,
    PointerAuthenticationFailure = 0b011100,
    TrappedSMEAccess = 0b011101,
    Reserved0x1e,
    Reserved0x1f,
    InstructionAbortFromLowerEL = 0b100000,
    InstructionAbortFromSameEL = 0b100001,
    ProgramCounterAlignmentFault = 0b100010,
    Reserved0x23,
    DataAbortFromLowerEL = 0b100100,
    DataAbortFromSameEL = 0b100101,
    StackPointerAlignmentFault = 0b100110,
    MemoryOperationException = 0b100111,
    TrappedFloatingPointAArch32 = 0b101000,
    Reserved0x29,
    Reserved0x2a,
    Reserved0x2b,
    TrappedFloatingPointAArch64 = 0b101100,
    Reserved0x2d,
    Reserved0x2e,
    SError = 0b101111,
    BreakpointFromLowerEL = 0b110000,
    BreakpointFromSameEL = 0b110001,
    SoftwareStepFromLowerEL = 0b110010,
    SoftwareStepFromSameEL = 0b110011,
    WatchpointFromLowerEL = 0b110100,
    WatchpointFromSameEL = 0b110101,
    Reserved0x36,
    Reserved0x37,
    BKPTInstructionAArch32 = 0b111000,
    Reserved0x39,
    Reserved0x3a,
    Reserved0x3b,
    BRKInstructionAArch64 = 0b111100,
    Reserved0x3d,
    Reserved0x3e,
    Reserved0x3f,
}

pub enum InstructionLength {
    Trapped16bitInstruction,
    Trapped32bitInstruction,
}

#[repr(C)]
pub struct ExceptionSyndrome(usize);

impl ExceptionSyndrome {
    pub fn exception_class(&self) -> ExceptionClass {
        unsafe { core::mem::transmute((self.0 >> 26 & 0x3F) as u8) }
    }

    pub fn instruction_length(&self) -> InstructionLength {
        unsafe { core::mem::transmute((self.0 >> 25 & 1) as u8) }
    }

    pub fn instruction_specific_syndrome(&self) -> u32 {
        self.0 as u32 & 0x1fff
    }
}

#[no_mangle]
pub extern "C" fn exc_handler(
    exception_type: ExceptionType,
    syndrome: ExceptionSyndrome,
    elr: usize,
    spsr: usize,
    far: usize,
) -> ! {
    loop {
        core::hint::spin_loop();
    }
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
    "1:", // We're not on the main core, so hang in an infinite wait loop
    // wait for event and loop
    "wfe",
    "b       1b",
    "2:", // We're on the main core!
    // Set stack to start below our code
    "ldr     x1, =__kernel_start",
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
    "b 3b",
    "4:",
    // Jump to our kernel_main() routine in rust (make sure it doesn't return)
    "bl kernel_main",
    // In case it does return, halt the master core too
    "b       1b",
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
