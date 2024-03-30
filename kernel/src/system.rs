use crate::{println_debug, println_log};

pub mod arm_core;
pub mod hal;
pub mod peripherals;
pub mod screen;
pub mod output;


pub fn initialize() {
    if cfg!(feature = "serial_uart") {
        output::init_serial_uart();
        println_log!("System Initialize...");
        // print a memory map
        println_debug!("{:#?}", hal::info::MemoryMap());
    }
    let status_led = hal::led::Led::Status;
    status_led.on();
    if cfg!(feature = "mmu") {
        arm_core::mmu::mmu_init().unwrap();
    }
    status_led.off();
    //let _a = std_out().lock();
    //writeln!(std_out(), "System Initialized").expect("second write should work");
}
