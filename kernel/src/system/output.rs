
use core::cell::RefCell;

use mystd::{
    io::SplitWriter,
    sync::mutex::{Mutex, MutexGuard},
};
use super::{hal::console, peripherals::uart};

use super::peripherals::uart::Uart;
use super::hal::console::Console;

pub type CombinedWriter = mystd::io::SplitWriter<Uart, Console<'static>>;

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

pub fn init_serial_uart() {
    uart::UART_0.init();
    let locked_out = unsafe { OUT_WRITER.lock() };
    let mut writer = locked_out.borrow_mut();
    writer.replace_first(uart::UART_0);
}

pub fn init_fb_console(base_ptr: *mut u8) {
    let console_writer = console::Console::new(base_ptr);
    let locked_out = unsafe { OUT_WRITER.lock() };
    let mut writer = locked_out.borrow_mut();
    writer.replace_second(console_writer);
}


#[macro_export]
macro_rules! println_log {
    ($($param:tt)*) => {
        if core::cfg!(any(feature = "serial_uart", feature = "framebuffer"))
        {
            use mystd::io::Write;
            let mut locked_out = $crate::system::output::std_out().lock();
            write!(&mut locked_out, "[{}] LOG | ", $crate::system::hal::thread::id()).expect("write should always work!");
            writeln!(&mut locked_out, $($param)*).expect("write to stdout should always work!");
        }
    };
}

#[macro_export]
macro_rules! print_log {
    ($($param:tt)*) => {
        if core::cfg!(any(feature = "serial_uart", feature = "framebuffer"))
        {
            use mystd::io::Write;
            let mut locked_out = $crate::system::output::std_out().lock();
            write!(&mut locked_out, $($param)*).expect("write to stdout should always work!");
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
            let mut locked_out = $crate::system::output::std_out().lock();
            write!(&mut locked_out, "[{}] DEBUG {}:{} | {:#.3?} | ", 
                $crate::system::hal::thread::id(), 
                file!(), 
                line!(),
                $crate::system::hal::counter::uptime()
            ).expect("debug write should always work!");
            writeln!(&mut locked_out, $($param)*).expect("debug write should always work!");
        }
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! println_debug {
    ($($param:tt)*) => {};
}