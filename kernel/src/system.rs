pub mod hal;
pub mod peripherals;
pub mod arm_core;

use core::cell::RefCell;

use mystd::{io::{SplitWriter, Write}, mutex::{Mutex, MutexGuard}};
use peripherals::uart;

use crate::system::peripherals::uart::Pl011Uart;

extern "C" {
    static __kernel_end: u8;
    static __data_start: u8;
}

pub type CombinedWriter = mystd::io::SplitWriter<Pl011Uart, Pl011Uart>;

pub struct Stdout {
    inner: &'static Mutex<RefCell<CombinedWriter>>
}

impl mystd::io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> mystd::io::Result<mystd::io::Size> {
        (&*self).write(buf)
    }

    fn flush(&mut self) -> mystd::io::Result<()> {
        (&*self).flush()
    }
}

impl mystd::io::Write for &Stdout {
    fn write(&mut self, buf: &[u8]) -> mystd::io::Result<mystd::io::Size> {
        self.lock().write(buf)
    }

    fn flush(&mut self) -> mystd::io::Result<()> {
        self.lock().flush()
    }
}

pub struct StdoutLock<'a> {
    inner: MutexGuard::<'a, RefCell<CombinedWriter>>
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
    pub fn lock(&self) -> StdoutLock<'static> {
        StdoutLock { inner: unsafe { self.inner.lock() } }
    }
}

static OUT_WRITER: Mutex<RefCell<CombinedWriter>> = Mutex::new(RefCell::new(SplitWriter::empty()));

pub fn std_out() -> Stdout {
    Stdout {
        inner: &OUT_WRITER
    }
}

fn init_serial_uart() {
    let locked_out = unsafe { OUT_WRITER.lock() };
    let mut writer = locked_out.borrow_mut();
    uart::UART_0.init();
    writer.replace_first(uart::UART_0);
    // writer.replace_second(uart::UART_0);
}

pub fn initialize() {
    // let core_id = system::get_core_num();
    // if core_id == 0 {
    if cfg!(feature = "serial_uart") {
        init_serial_uart();
        writeln!(std_out(), "System Initialize...").unwrap();
    }
    let status_led = hal::led::Led::Status;
    status_led.on();
    if cfg!(feature = "mmu") {
        arm_core::mmu::mmu_init().unwrap();
    }
    status_led.off();
    writeln!(std_out(), "System Initialized").expect("second write should work");
}


