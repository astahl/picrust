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
    unsafe { clear_bss(); }
    peripherals::led_on();
}

const FONT: [u64; 6] = [
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000, // space
    0b00010000_00010000_00010000_00010000_00010000_00000000_00010000_00000000, // !
    0b01010000_01010000_00000000_00000000_00000000_00000000_00000000_00000000, // "
    0b00101000_01111100_00101000_01010000_11111000_01010000_10100000_00000000, // #
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000, // $
    0b00001111_00011110_00111100_01111000_11110000_01111000_00111100_00011110,
    ];

#[no_mangle]
pub extern "C" fn kernel_main() {
    let core_num = get_core_num();
    match core_num {
        0 => initialize_global(),
        _ => {}
    }

    let font = unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(__font_start), core::ptr::addr_of!(__font_end).offset_from(core::ptr::addr_of!(__font_start)).unsigned_abs()) };

    let text = b" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
    let mapping = |c: u8| -> u8 {
        match c {
            0 => b' ',
            b' '..=b'?' => c,
            b'@'..=b'_' => c as u8 - b'@', 
            b'a'..=b'z' => c as u8 - b'`' | 0x80,
            _ => 255
        }
    };

    if let Some(framebuffer) = framebuffer::Framebuffer::new(1280, 720) {
        let repeat = (5,6);
        let offset = (40,48);
        let size = (framebuffer.width_px - 2 * offset.0, framebuffer.height_px - 2 * offset.1);
        let columns = size.0 as usize / (repeat.0 * 8);
        for y in 0..size.1 {
            let yy = y as usize / repeat.1; 
            for x in 0..size.0 {
                let xx = x as usize / repeat.0;
                let char_index = (xx / 8, yy / 8);
                let linear_index = char_index.1 * columns + char_index.0;
                let ch = text.get(linear_index).copied().unwrap_or_default();
                let char = font[mapping(ch) as usize % font.len()];
                let char_subpixel = (xx % 8, yy % 8);
                if (char << ((7 - char_subpixel.1) * 8 + char_subpixel.0)) & (1_u64 << 63) == 0 {
                    framebuffer.set_pixel_a8b8g8r8(x + offset.0, y + offset.1, 0xFF0000AA);
                } else {
                    framebuffer.set_pixel_a8b8g8r8(x + offset.0, y + offset.1, 0xFFFFFFFF);
                }
            }
        }
        for y in 400..800 {
            for x in 400..800 {
                let x = x + (core_num * 400) as u32;
                framebuffer.set_pixel_a8b8g8r8(x, y, 0xFF00AA00);
               // crate::peripherals::delay(100000);
            }
        }
    }
    
    use peripherals::uart::Uart0;
    Uart0::init();
    unsafe { 
        Uart0::put_hex_bytes(&(FONT.as_ptr() as usize).to_be_bytes());
        Uart0::put_hex_bytes(&((core::ptr::addr_of!(__rodata_start) as usize).to_be_bytes()));
        Uart0::put_hex_bytes(&((core::ptr::addr_of!(__rodata_end) as usize).to_be_bytes()));
       // Uart0::put_hex_bytes(&((__rodata_end - __rodata_start).to_be_bytes()));
    }

    Uart0::puts("ARM Memory\n");
    if let Some(arm_memory) = peripherals::Hardware::get_arm_memory() {
        Uart0::put_hex_bytes(&(arm_memory.base_address).to_be_bytes());
        Uart0::put_hex_bytes(&(arm_memory.size).to_be_bytes());
    }

    Uart0::puts("VC Memory\n");
    if let Some(vc_memory) = peripherals::Hardware::get_vc_memory() {
        Uart0::put_hex_bytes(&(vc_memory.base_address).to_be_bytes());
        Uart0::put_hex_bytes(&(vc_memory.size).to_be_bytes());
    }

    Uart0::puts("Board Info");
    if let Some(board_info) = peripherals::Hardware::get_board_info() {
        Uart0::puts("\nModel ");
        Uart0::put_uint(board_info.model as u64);
        Uart0::puts("\nRev ");
        Uart0::put_uint(board_info.revision as u64);
        Uart0::puts("\nSerial ");
        Uart0::put_uint(board_info.serial);
    }

    Uart0::puts("\nMAC ");
    if let Some(mac) = peripherals::Hardware::get_mac_address() {
        Uart0::put_hex_bytes(&mac);
    }

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

global_asm!(
    ".section .font",
    ".incbin \"901447-10.bin\""
);

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