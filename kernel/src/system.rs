pub mod hal;
pub mod peripherals;
pub mod arm_core;

use core::{cell::RefCell, fmt::{Debug, Display}};

use mystd::{io::{SplitWriter, Write}, mutex::{Mutex, MutexGuard}};
use peripherals::uart;

use peripherals::uart::Uart;

extern "C" {
    static __main_stack: u8;
    static __kernel_start: u8;
    static __kernel_txt_start: u8;
    static __kernel_txt_end: u8;
    static __rodata_start: u8;
    static __font_start: u8;
    static __font_end: u8;
    static __rodata_end: u8;
    static __data_start: u8;
    static __data_end: u8;
    static __bss_start: u8;
    static __bss_end: u8;
    static __kernel_end: u8;
    static __free_memory_start: u8;
}

struct MemoryBlock(*const u8, *const u8);

impl core::fmt::Debug for MemoryBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[{:#p} - {:#p}]({})", self.0, self.1, mystd::format::ByteValue(self.byte_size()))
    }
}

impl MemoryBlock{
    fn from_symbols<T>(start: &T, end: &T) -> Self {
        Self(core::ptr::addr_of!(*start).cast(), core::ptr::addr_of!(*end).cast())
    }

    fn from_start_and_count<T>(start: &T, count: usize) -> Self {
        Self(core::ptr::addr_of!(*start).cast(), core::ptr::addr_of!(start).wrapping_add(count).cast())
    }

    fn byte_size(&self) -> usize {
        unsafe { self.0.offset_from(self.1).unsigned_abs() }
    }
}

struct MemoryMap();

impl Debug for MemoryMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe{
        let stack = core::ptr::addr_of!(__main_stack);
        let kernel_text = MemoryBlock::from_symbols(&__kernel_txt_start, &__kernel_txt_end);
        let kernel = MemoryBlock::from_symbols(&__kernel_start, &__kernel_end);
        let rodata = MemoryBlock::from_symbols(&__rodata_start, &__rodata_end);
        let font = MemoryBlock::from_symbols(&__font_start, &__font_end);
        let data = MemoryBlock::from_symbols(&__data_start, &__data_end);
        let bss = MemoryBlock::from_symbols(&__bss_start, &__bss_end);
        let ram = core::ptr::addr_of!(__free_memory_start);
        f.debug_struct("MemoryMap")
            .field("Stack Top", &format_args!("{stack:#p}"))
            .field("Kernel", &kernel)
            .field("Kernel Code", &kernel_text)
            .field("Read-Only Data Segment", &rodata)
            .field("Font", &font)
            .field("Data Segment", &data)
            .field("BSS Segment", &bss)
            .field("Heap Bottom", &format_args!("{ram:#p}"))
            .finish()
        }
    }
}



pub type CombinedWriter = mystd::io::SplitWriter<Uart, Uart>;

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
    //writer.replace_second(uart::UART_0);
    let ptr = core::ptr::addr_of!(OUT_WRITER);
    writeln!(writer, "OUT_WRITER at {:#p}-{:#p}", ptr, ptr.wrapping_add(1));
    // writer.replace_second(uart::UART_0);
}

pub fn initialize() {
    // let core_id = system::get_core_num();
    // if core_id == 0 {
    if cfg!(feature = "serial_uart") {
        init_serial_uart();
        writeln!(std_out(), "System Initialize...").unwrap();
        // print a memory map
        writeln!(std_out(), "{:#?}", MemoryMap()).unwrap();
    }
    let status_led = hal::led::Led::Status;
    status_led.on();
    if cfg!(feature = "mmu") {
        arm_core::mmu::mmu_init().unwrap();
    }
    status_led.off();
    let _a = std_out().lock();
    //writeln!(std_out(), "System Initialized").expect("second write should work");
}


