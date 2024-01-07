#![no_std]
#![no_main]

mod peripherals;
mod hal;
mod monitor;
use core::arch::global_asm;

#[panic_handler]
fn on_panic(_info: &core::panic::PanicInfo) -> ! {
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
    unsafe { clear_bss(); }
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

    let test = hal::display::EdidBlock::test_block();


    let mut str_buffer = StringBuffer::<4096>::new();
    Uart0::puts(core::str::from_utf8(&test.manufacturer_id()).unwrap());
    Uart0::put_uint(test.manufacturer_product_code() as u64);
    writeln!(str_buffer, "Checksum Ok? {:?}", test.checksum_ok()).unwrap();
    writeln!(str_buffer, "Video Input Parameter: {:?}", test.video_input_parameter()).unwrap();
    writeln!(str_buffer, "Screen Geometry: {:?}", test.screen_geometry()).unwrap();
    writeln!(str_buffer, "Supported Features: {:?}", test.supported_features()).unwrap();
    writeln!(str_buffer, "{:?}", test.chromaticity_coordinates()).unwrap();
    writeln!(str_buffer, "{:?}", test.common_timing_support()).unwrap();
    writeln!(str_buffer, "{:?}", test.standard_timing_information()).unwrap();
    writeln!(str_buffer, "{:?}", test.descriptors()).unwrap();
    
    Uart0::puts(str_buffer.str());


    str_buffer.reset();
    let test = hal::display::CtaExtensionBlock::test_block();
    writeln!(str_buffer, "Checksum Ok? {:?}", test.checksum_ok()).unwrap();
    writeln!(str_buffer, "Support underscan? {:?}", test.support_underscan()).unwrap();
    writeln!(str_buffer, "Support basic_audio? {:?}", test.support_basic_audio()).unwrap();
    writeln!(str_buffer, "Support ycbcr_444? {:?}", test.support_ycbcr_444()).unwrap();
    writeln!(str_buffer, "Support ycbcr_422? {:?}", test.support_ycbcr_422()).unwrap();
    writeln!(str_buffer, "Native format count: {:?}", test.native_format_count()).unwrap();
    for db in test.data_blocks() {
        match db {
            hal::display::CtaDataBlock::None => {},
            hal::display::CtaDataBlock::Audio { audio_blocks } => for block in audio_blocks {
                writeln!(str_buffer, "{:?}", block).unwrap();
            },
            hal::display::CtaDataBlock::Video { video_blocks } => for block in video_blocks {
                writeln!(str_buffer, "{:?}", block).unwrap();
            },
            a => writeln!(str_buffer, "{:?}", a).unwrap(),
            hal::display::CtaDataBlock::VesaDisplayTransferCharacteristic => {},
            hal::display::CtaDataBlock::VideoFormat => {},
            hal::display::CtaDataBlock::Extended => {},
        }
        
    }
    Uart0::puts(str_buffer.str());

    use hal::framebuffer::color;
    let fb = hal::framebuffer::Framebuffer::new(1280, 720).unwrap();
    fb.clear(color::BLACK);

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
    fb.clear(color::BLUE);
    fb.write_text(text, font, mapping);
    fb.clear(color::RED);

    let mut str_buffer = StringBuffer::<1024>::new();
    use core::fmt::Write;
    writeln!(str_buffer, "Framebuffer: {} {} {}", fb.width_px, fb.height_px, fb.bits_per_pixel).unwrap();
    if let Some(arm_memory) = hal::info::get_arm_memory() {
        writeln!(str_buffer, "ARM Memory {:#X} {:#X}", arm_memory.base_address, arm_memory.size).unwrap();
    }
    if let Some(vc_memory) = hal::info::get_vc_memory() {
        writeln!(str_buffer, "VC Memory {:#X} {:#X}", vc_memory.base_address, vc_memory.size).unwrap();
    }
    // if let Some(board_info) = hal::info::get_board_info() {
    //     writeln!(str_buffer, "Board Model: {} Revision: {:x} Serial: {}", board_info.model, board_info.revision, board_info.serial).unwrap();
    // }
    // if let Some(mac) = hal::info::get_mac_address() {
    //     writeln!(str_buffer, "MAC {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]).unwrap();
    // }

    for edid in hal::display::EdidIterator::new().take(2) {
        writeln!(str_buffer, "EDID BLOCK {}", edid.0).unwrap();
        for byte in edid.1 {
            write!(str_buffer, "{:02X} ", byte).unwrap();
        }
    }
    writeln!(str_buffer, "Bye!").unwrap();
    let text = str_buffer.str().as_bytes();
    fb.clear(color::GREEN);
    fb.write_text(text, font, mapping);

    // Uart0::puts(str_buffer.str());
    // Uart0::put_uint(core as u64);
    // Uart0::puts("Hallo\n");
    // 
    // let mut mon = monitor::Monitor::new(|| Uart0::get_byte().unwrap_or(b'0'), Uart0::putc);
    // mon.run();

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

pub fn delay(mut count: usize) {
    while count > 0 {
        count -= 1;
        core::hint::spin_loop();
    }
}

struct StringBuffer<const CAPACITY: usize> {
    data: [u8; CAPACITY],
    len: usize
}

impl<const CAPACITY: usize> StringBuffer<CAPACITY> {
    pub fn new() -> Self {
        Self { data: [0; CAPACITY], len: 0 }
    }

    pub fn str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.data.split_at(self.len).0) }
    }

    pub fn reset(&mut self) {
        self.len = 0;
    }
}

impl<const CAPACITY: usize> core::fmt::Write for StringBuffer<CAPACITY> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let new_length = s.len() + self.len;
        if CAPACITY < new_length {
            Err(core::fmt::Error{})
        } else {
            unsafe {
                core::ptr::copy_nonoverlapping(s.as_ptr(), self.data.as_mut_ptr().add(self.len), s.len());
            }
            self.len = new_length;
            Ok({})
        }
    }
}   