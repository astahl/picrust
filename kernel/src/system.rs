use crate::{println_debug, println_log};

pub mod arm_core;
pub mod hal;
pub mod peripherals;
pub mod screen;
pub mod output;


pub fn initialize() {
    if cfg!(feature = "serial_uart") {
        output::init_serial_uart();
        println_log!("Serial UART Initialized...");
        // print a memory map
        println_debug!("{:#?}", hal::info::MemoryMap());
    }
    
    if cfg!(feature = "framebuffer") {
        screen::create_screen(0x50_0000 as *mut u8);
        output::init_fb_console(0x60_0000 as *mut u8);
    //    println_log!("Framebuffer Console created...");
        // print a memory map
    //    println_debug!("{:#?}", hal::info::MemoryMap());
    }

    if cfg!(feature = "mmu") {
        println_log!("MMU...");
        arm_core::mmu::mmu_init().expect("MMU should be initialised");
        println_log!("MMU initialised");
    }
    //let _a = std_out().lock();
    //writeln!(std_out(), "System Initialized").expect("second write should work");
}
