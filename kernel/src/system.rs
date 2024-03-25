pub mod arm_core;
pub mod hal;
pub mod peripherals;
pub mod screen;

use core::{
    cell::RefCell,
    fmt::{Debug, Display},
};

use mystd::{
    io::SplitWriter,
    mutex::{Mutex, MutexGuard},
};
use peripherals::uart;

use peripherals::uart::Uart;

pub type CombinedWriter = mystd::io::SplitWriter<Uart, Uart>;

pub struct Stdout {
    inner: &'static Mutex<RefCell<CombinedWriter>>,
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
    inner: MutexGuard<'a, RefCell<CombinedWriter>>,
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
        StdoutLock {
            inner: unsafe { self.inner.lock() },
        }
    }
}

static OUT_WRITER: Mutex<RefCell<CombinedWriter>> = Mutex::new(RefCell::new(SplitWriter::empty()));

pub fn std_out() -> Stdout {
    Stdout { inner: &OUT_WRITER }
}

fn init_serial_uart() {
    let locked_out = unsafe { OUT_WRITER.lock() };
    let mut writer = locked_out.borrow_mut();
    uart::UART_0.init();
    writer.replace_first(uart::UART_0);
    
}

#[macro_export]
macro_rules! println_log {
    ($($param:tt)*) => {
        if core::cfg!(any(feature = "serial_uart", feature = "framebuffer"))
        {
            use mystd::io::Write;
            writeln!($crate::system::std_out(), $($param)*).expect("write to stdout should always work!");
        }
    };
}

#[macro_export]
macro_rules! print_log {
    ($($param:tt)*) => {
        if core::cfg!(any(feature = "serial_uart", feature = "framebuffer"))
        {
            use mystd::io::Write;
            write!($crate::system::std_out(), $($param)*).expect("write to stdout should always work!");
        }
    };
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! println_debug {
    ($($param:tt)*) => {
        if core::cfg!(any(feature = "serial_uart", feature = "framebuffer"))
        {
            use mystd::io::Write;
            writeln!($crate::system::std_out(), $($param)*).expect("debug write should always work!");
        }
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! println_debug {
    ($($param:tt)*) => {};
}

pub fn initialize() {
    if cfg!(feature = "serial_uart") {
        init_serial_uart();
        crate::println_debug!(
            "OUT_WRITER at {:#?}",
            hal::info::MemoryBlock::from(&OUT_WRITER)
        );
        print_log!("System Initialize...");
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
