pub mod hal;
pub mod peripherals;

use core::{arch::asm, time::Duration};

extern "C" {
    static __kernel_end: u8;
    static __data_start: u8;
}


pub fn initialize() {
    // let core_id = system::get_core_num();
    // if core_id == 0 {
    if cfg!(feature = "serial_uart") {
        peripherals::uart::Uart0::init();
        peripherals::uart::Uart0::puts("System Initialize...\n");
    }
    let status_led = hal::led::Led::Status;
    status_led.on();
    if cfg!(feature = "mmu") {
        mmu_init().unwrap();
    }
    status_led.off();
}

#[derive(Debug)]
pub enum MMUInitError {
    PhysicalAddressRangeAtLeast36bitNotSupported,
    TranslationGranule4kbNotSupported,
    NotImplementedError,
}

#[cfg(not(feature = "mmu"))]
pub fn mmu_init() -> Result<(), MMUInitError> {
    Err(MMUInitError::NotImplementedError)
}

#[cfg(feature = "mmu")]
pub fn mmu_init() -> Result<(), MMUInitError> {
    // check for 4k granule and at least 36 bits physical address bus */
    let mut memory_model_features = 0_usize;
    // see D19.2.64 for register layout
    unsafe {
        asm!("mrs {}, id_aa64mmfr0_el1", out(reg) memory_model_features);
    }
    let par_flags = memory_model_features & 0xF;
    if par_flags < 0b0001 {
        return Err(MMUInitError::PhysicalAddressRangeAtLeast36bitNotSupported);
    }
    if (memory_model_features >> 28) & 0xF == 0xF {
        return Err(MMUInitError::TranslationGranule4kbNotSupported);
    }

    const PAGESIZE: usize = 4096;
    const PAGE_ENTRY_COUNT: usize = PAGESIZE / core::mem::size_of::<usize>();
    // granularity
    const PT_PAGE: usize = 0b11; // 4k granule
    const PT_BLOCK: usize = 0b01; // 2M granule
                                  // accessibility
    const PT_KERNEL: usize = 0 << 6; // privileged, supervisor EL1 access only
    const PT_USER: usize = 1 << 6; // unprivileged, EL0 access allowed
    const PT_RW: usize = 0 << 7; // read-write
    const PT_RO: usize = 1 << 7; // read-only
    const PT_AF: usize = 1 << 10; // accessed flag
    const PT_NX: usize = 1 << 54; // no execute
                                  // shareability
    const PT_OSH: usize = 2 << 8; // outter shareable
    const PT_ISH: usize = 3 << 8; // inner shareable
                                  // defined in MAIR register
    const PT_MEM: usize = 0 << 2; // normal memory
    const PT_DEV: usize = 1 << 2; // device MMIO
    const PT_NC: usize = 2 << 2; // non-cachable

    const TTBR_ENABLE: usize = 1;

    const MMIO_BASE: usize = peripherals::BCM_HOST.peripheral_address;

    let data_page_index = unsafe { core::ptr::addr_of!(__data_start) } as usize / PAGESIZE;
    let page_table_ptr = unsafe { core::ptr::addr_of!(__kernel_end) };

    let page_table_entry_ptr = page_table_ptr.cast::<usize>().cast_mut();

    /* create MMU translation tables at __kernel_end */

    // TTBR0, identity L1
    let physical_address = page_table_ptr.wrapping_add(2 * PAGESIZE) as usize;
    let flags = PT_PAGE |     // it has the "Present" flag, which must be set, and we have area in it mapped by pages
                PT_AF |       // accessed flag. Without this we're going to have a Data Abort exception
                PT_USER |     // non-privileged
                PT_ISH |      // inner shareable
                PT_MEM; // normal memory
    unsafe {
        page_table_entry_ptr.write_volatile(physical_address | flags);
    }

    // identity L2, first 2M block
    let physical_address = page_table_ptr.wrapping_add(3 * PAGESIZE) as usize;
    let flags = PT_PAGE |     // we have area in it mapped by pages
                PT_AF |       // accessed flag
                PT_USER |     // non-privileged
                PT_ISH |      // inner shareable
                PT_MEM; // normal memory
    unsafe {
        page_table_entry_ptr
            .wrapping_add(2 * PAGE_ENTRY_COUNT)
            .write_volatile(physical_address | flags);
    }
    // identity L2 2M blocks
    // TODO (astahl): this is 2032 on Pi 4's SoC, do we need to extend the block count up to 2048?
    let device_memory_block = MMIO_BASE >> 21; // 2 megabyte blocks
                                               // skip 0th, as we're about to map it by L3
    for block in 1..PAGE_ENTRY_COUNT {
        let physical_address = block << 21;
        let flags = PT_BLOCK |    // map 2M block
                    PT_AF |       // accessed flag
                    PT_NX |       // no execute
                    PT_USER |     // non-privileged
                    if block >= device_memory_block {
                        // different attributes for device memory
                        // TODO (astahl) this is never reached atm
                        PT_OSH | PT_DEV
                    } else {
                        PT_ISH | PT_MEM
                    };
        unsafe {
            page_table_entry_ptr
                .wrapping_add(2 * PAGE_ENTRY_COUNT + block)
                .write_volatile(physical_address | flags);
        }
    }

    // identity L3
    for page in 0..PAGE_ENTRY_COUNT {
        let physical_address = page * PAGESIZE;
        let flags = PT_PAGE |     // map 4k
                    PT_AF |       // accessed flag
                    PT_USER |     // non-privileged
                    PT_ISH |      // inner shareable
                    if page < 0x80 || page > data_page_index {
                        PT_RW | PT_NX
                    } else {
                        PT_RO
                    }; // different for code and data
        unsafe {
            page_table_entry_ptr
                .wrapping_add(3 * PAGE_ENTRY_COUNT + page)
                .write_volatile(physical_address | flags);
        }
    }

    // TTBR1, kernel L1
    let physical_address = page_table_ptr.wrapping_add(4 * PAGESIZE) as usize;
    let flags = PT_PAGE |     // we have area in it mapped by pages
                PT_AF |       // accessed flag
                PT_KERNEL |   // privileged
                PT_ISH |      // inner shareable
                PT_MEM; // normal memory
    unsafe {
        page_table_entry_ptr
            .wrapping_add(2 * PAGE_ENTRY_COUNT - 1)
            .write_volatile(physical_address | flags);
    }
    // kernel L2
    let physical_address = page_table_ptr.wrapping_add(5 * PAGESIZE) as usize;
    let flags = PT_PAGE |     // we have area in it mapped by pages
                PT_AF |       // accessed flag
                PT_KERNEL |   // privileged
                PT_ISH |      // inner shareable
                PT_MEM; // normal memory
    unsafe {
        page_table_entry_ptr
            .wrapping_add(5 * PAGE_ENTRY_COUNT - 1)
            .write_volatile(physical_address | flags);
    }
    // kernel L3
    // TODO (astahl): this is the UART0 address??
    let physical_address = MMIO_BASE + 0x00201000;
    let flags = PT_PAGE |     // map 4k
                PT_AF |       // accessed flag
                PT_NX |       // no execute
                PT_KERNEL |   // privileged
                PT_OSH |      // outter shareable
                PT_DEV; // device memory
    unsafe {
        page_table_entry_ptr
            .wrapping_add(5 * PAGE_ENTRY_COUNT)
            .write_volatile(physical_address | flags);
    }
    /* okay, now we have to set system registers to enable MMU */

    // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
    let r: usize = (0xFF << 0) | // AttrIdx=0: normal, IWBWA, OWBWA, NTR
                   (0x04 << 8) | // AttrIdx=1: device, nGnRE (must be OSH too)
                   (0x44 <<16); // AttrIdx=2: non cacheable
    unsafe {
        asm!("msr mair_el1, {}", in(reg) r);
    }

    // next, specify mapping characteristics in translate control register
    let r: usize = (0 << 37) | // TBI=0, no tagging
                   (par_flags << 32) | // IPS=autodetected
                   (0b10 << 30) | // TG1=4k
                   (0b11 << 28) | // SH1=3 inner
                   (0b01 << 26) | // ORGN1=1 write back
                   (0b01 << 24) | // IRGN1=1 write back
                   (0b0  << 23) | // EPD1 enable higher half
                   (25   << 16) | // T1SZ=25, 3 levels (512G)
                   (0b00 << 14) | // TG0=4k
                   (0b11 << 12) | // SH0=3 inner
                   (0b01 << 10) | // ORGN0=1 write back
                   (0b01 << 8) |  // IRGN0=1 write back
                   (0b0  << 7) |  // EPD0 enable lower half
                   (25   << 0); // T0SZ=25, 3 levels (512G)
    unsafe {
        asm!("msr tcr_el1, {}; isb", in(reg)r);
    }

    // tell the MMU where our translation tables are. TTBR_ENABLE bit not documented, but required
    // lower half, user space
    let ttbr0_address = page_table_ptr as usize | TTBR_ENABLE;
    unsafe {
        asm!("msr ttbr0_el1, {}", in(reg) ttbr0_address);
    }
    // upper half, kernel space
    let ttbr1_address = page_table_ptr as usize + PAGESIZE | TTBR_ENABLE;
    unsafe {
        asm!("msr ttbr1_el1, {}", in(reg) ttbr1_address);
    }

    // finally, toggle some bits in system control register to enable page translation
    let mut r: usize = 0;
    unsafe {
        // Data synchronization barrier then instruction synchronization barrier to guarantee all preceding memory accesses have been finished and none of the following instructions have been performed yet.
        asm!("dsb ish; isb; mrs {}, sctlr_el1", out(reg) r);
    }
    r |= 0xC00800; // set mandatory reserved bits
    r &= !((1 << 25) |   // clear EE, little endian translation tables
           (1 << 24) |   // clear E0E
           (1 << 19) |   // clear WXN
           //(1 << 12) |   // clear I, no instruction cache
           (1 << 4) |    // clear SA0, no Stack Pointer Alignment check at EL0
           (1 << 3) |    // clear SA, no Stack Pointer Alignment check at EL1
           //(1 << 2) |    // clear C, no cache at all
           (1 << 1)); // clear A, no aligment check
    r |= 1 << 0; // set M, enable MMU
    unsafe {
        asm!("msr sctlr_el1, {}; isb", in(reg) r);
    }
    Ok(())
}

pub fn get_core_num() -> usize {
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

pub fn current_exception_level() -> usize {
    let mut el: usize;
    unsafe {
        core::arch::asm!(
            "mrs {0}, CurrentEL",
            out(reg) el
        );
    }
    return (el >> 2) & 0b11;
}

pub fn wait(duration: Duration) {
    let mut frequency: usize;
    let mut current_counter: usize;
    unsafe {
        core::arch::asm!(
            "mrs {0}, cntfrq_el0",
            "mrs {1}, cntpct_el0",
            out(reg) frequency, out(reg) current_counter
        );
    }
    let expire_at =
        current_counter + ((frequency as u128 * duration.as_micros()) / 1_000_000) as usize;
    while current_counter < expire_at {
        unsafe {
            core::arch::asm!(
                "mrs {0}, cntpct_el0",
                out(reg) current_counter
            );
        }
        core::hint::spin_loop();
    }
}

pub fn wait_cycles(mut count: usize) {
    while count > 0 {
        count -= 1;
        core::hint::spin_loop();
    }
}
