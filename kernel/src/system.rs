use crate::{print_init, println_debug, println_log};

pub mod arm_core;
pub mod hal;
pub mod peripherals;
pub mod screen;
pub mod output;


pub fn initialize() {
    if cfg!(feature = "mmu") {
        print_init!("before mmu");
        arm_core::mmu::mmu_init()
            .expect("MMU should be initialised");
        print_init!("after mmu");
    }
    if cfg!(feature = "serial_uart") {
        print_init!("before serial uart");
        output::init_serial_uart();
        println_log!("Serial UART Initialized...");
        // print a memory map
        println_log!("{:#?}", hal::info::MemoryMap());
    }
    
    if cfg!(feature = "framebuffer") {
        screen::create_screen(0x50_0000 as *mut u8);
        output::init_fb_console(0x60_0000 as *mut u8);
    //    println_log!("Framebuffer Console created...");
        // print a memory map
    //    println_debug!("{:#?}", hal::info::MemoryMap());
    }

    //let _a = std_out().lock();
    //writeln!(std_out(), "System Initialized").expect("second write should work");
}
