#![no_std]
#![no_main]

mod buffer;
mod hal;
mod monitor;
mod peripherals;
mod drawing;
use core::arch::global_asm;

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
        hal::led::status_blink();
    }
}

extern "C" {
    static mut __bss_start: u8;
    static mut __bss_end: u8;
    static __rodata_start: usize;
    static __rodata_end: usize;
    static __font_start: u64;
    static __font_end: u64;
}

unsafe fn clear_bss() {
    let from = core::ptr::addr_of_mut!(__bss_start);
    let to = core::ptr::addr_of_mut!(__bss_end);
    let distance = to.offset_from(from);
    from.write_bytes(0, distance.unsigned_abs());
}

fn initialize_global() {
    unsafe {
        clear_bss();
    }
    hal::led::status_set(true);
}

#[no_mangle]
pub extern "C" fn kernel_main() {
    let core_num = get_core_num();
    match core_num {
        0 => initialize_global(),
        _ => {}
    }

    use peripherals::uart::Uart0;
    Uart0::init();
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
            b'@'..=b'_' => c as u8 - b'@',
            b'a'..=b'z' => c as u8 - b'`' | 0x80,
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
    if let Some(board_info) = hal::info::get_board_info() {
        writeln!(str_buffer, "{}", board_info.revision).unwrap();
    }
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
    let text = str_buffer.as_slices();
    fb.clear(color::BLACK);
    //fb.write_text(text.0, font, mapping);

    Uart0::puts(core::str::from_utf8(text.0).unwrap());
    // Uart0::put_uint(core as u64);
    // Uart0::puts("Hallo\n");
    //
    // let mut mon = monitor::Monitor::new(|| Uart0::get_byte().unwrap_or(b'0'), Uart0::putc);
    // mon.run();
    fb.set_pixel_a8b8g8r8(150, 100, color::WHITE);
    let mut canvas = drawing::PixelCanvas::with_slice(300, 300, fb.pitch_bytes as usize / 4, fb.as_mut_pixels()).unwrap();
    //canvas.clear(color::BLUE);
    canvas.fill_rect(color::BLUE, (298, 298), (300, 300)).unwrap();
    canvas.fill_lines(color::RED, 100..=100).unwrap();
    canvas.blit8x8(&font[0x06].to_le_bytes(), color::WHITE, color::BLACK, (100, 200)).unwrap();
    canvas.blit8x8(&font[0x07].to_le_bytes(), color::GREEN, color::RED, (108, 200)).unwrap();

    fb.set_pixel_a8b8g8r8(153, 100, color::WHITE);
    hal::led::status_set(false);
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

global_asm!(".section .font", ".incbin \"901447-10.bin\"");

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
    ".section \".text.boot\"", // Make sure the linker puts this at the start of the kernel image
    ".global _start",          // Execution starts here
    "_start:",
    // Check processor ID is zero (executing on main core), else hang
    "mrs     x1, mpidr_el1",
    "and     x1, x1, #3",
    "cbz     x1, 2f",
    // We're not on the main core, so hang in an infinite wait loop
    "1:  wfe",
    "b       1b",
    "2:", // We're on the main core!
    // Set stack to start below our code
    "ldr     x1, =_start",
    "mov     sp, x1",
    // Jump to our main() routine in C (make sure it doesn't return)
    "4:  bl      kernel_main",
    // In case it does return, halt the master core too
    "b       1b"
);

pub fn delay(mut count: usize) {
    while count > 0 {
        count -= 1;
        core::hint::spin_loop();
    }
}
