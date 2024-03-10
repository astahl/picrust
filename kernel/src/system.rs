pub mod hal;
pub mod peripherals;
pub mod arm_core;

use peripherals::uart;

extern "C" {
    static __kernel_end: u8;
    static __data_start: u8;
}


pub fn initialize() {
    // let core_id = system::get_core_num();
    // if core_id == 0 {
    if cfg!(feature = "serial_uart") {
        use core::fmt::Write;
        let mut uart = uart::UART_0;
        uart.init();
        writeln!(&mut uart, "System Initialize...").unwrap();
    }
    let status_led = hal::led::Led::Status;
    status_led.on();
    if cfg!(feature = "mmu") {
        arm_core::mmu::mmu_init().unwrap();
    }
    status_led.off();
}


