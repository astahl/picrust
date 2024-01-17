#![no_std]
#![no_main]

mod buffer;
mod hal;
mod monitor;
mod peripherals;
mod drawing;
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
    let mut canvas = drawing::PixelCanvas::with_slice(fb.width_px as usize, fb.height_px as usize, fb.pitch_bytes as usize / 4, fb.as_mut_pixels()).unwrap();
    //canvas.clear(color::BLUE);
    canvas.fill_rect(color::BLUE, (298, 298), (300, 300)).unwrap();
    canvas.fill_lines(color::RED, 100..=100).unwrap();
    let pixelscale = (5, 6);
    let cols = canvas.width / (pixelscale.0*8);
    let rows = canvas.height / (pixelscale.1*8);
    let mut row_buffer = [0_u64; 256];
    let mut v_scroll: usize = 0;
    hal::led::status_set(false);
    loop {
        let line_iterator = text.split(|b| *b == b'\n').flat_map(|l| l.chunks(cols)).cycle();
        canvas.fill_rect(0, (0,0), (cols * 8, rows * 8)).unwrap();
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
            canvas.blit8x8_line(&row_buffer[pre..len+pre], color::WHITE, color::BLACK, (pre * 8, row_nr * 8)).unwrap();   
        }
        canvas.scale_in_place(pixelscale.0,pixelscale.1);
        v_scroll += 1;

        system::wait_msec(500);
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
    "b       1b"
);

