pub mod hal;
pub mod peripherals;
pub mod arm_core;

use mystd::io::Write;
use peripherals::uart;

use crate::system::peripherals::uart::Pl011Uart;

extern "C" {
    static __kernel_end: u8;
    static __data_start: u8;
}


pub struct Stdout {
    inner: &'static mystd::mutex::Mutex<core::cell::RefCell<Pl011Uart>>
}

pub struct StdoutLock<'a> {
    inner: mystd::mutex::MutexGuard::<'a, core::cell::RefCell<Pl011Uart>>
}

impl<'a> mystd::io::Write for StdoutLock<'a> {    
    fn write(&mut self, buf: &[u8]) -> mystd::io::Result<mystd::io::Size> {
        self.inner.borrow_mut().write(buf)
    }
    
    fn flush(&mut self) -> mystd::io::Result<()> {
        self.inner.borrow_mut().flush()
    }
}

impl Stdout {
    pub fn lock(&mut self) -> StdoutLock<'static> {
        StdoutLock { inner: self.inner.try_lock().unwrap() }
    }
}

pub fn std_out() -> Stdout {
    static UART: mystd::mutex::Mutex<core::cell::RefCell<Pl011Uart>> = mystd::mutex::Mutex::new(core::cell::RefCell::new(uart::UART_0));
    Stdout {
        inner: &UART
    }
}

pub fn initialize() {
    // let core_id = system::get_core_num();
    // if core_id == 0 {
    if cfg!(feature = "serial_uart") {
        let mut out = std_out().lock();
        writeln!(out, "System Initialize...").unwrap();
    }
    let status_led = hal::led::Led::Status;
    status_led.on();
    if cfg!(feature = "mmu") {
        arm_core::mmu::mmu_init().unwrap();
    }
    status_led.off();
}


