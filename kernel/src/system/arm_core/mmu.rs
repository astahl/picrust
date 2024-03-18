pub mod descriptors;

use core::arch::asm;

use crate::system::hal::info::MemoryMap;
use crate::system::peripherals::BCM_HOST;

use self::descriptors::{PageDescriptor, TableDescriptor};

#[derive(Debug)]
pub enum MMUInitError {
    PhysicalAddressRangeAtLeast36bitNotSupported,
    TranslationGranule4kbNotSupported,
    #[cfg(not(feature = "mmu"))]
    NotImplementedError,
}

#[cfg(not(feature = "mmu"))]
pub fn mmu_init() -> Result<(), MMUInitError> {
    Err(MMUInitError::NotImplementedError)
}


const NORMAL_MEMORY_ATTR_IDX: u64 = 0;
const DEVICE_MEMORY_ATTR_IDX: u64 = 1;

#[cfg(feature = "mmu")]
pub fn mmu_init() -> Result<(), MMUInitError> {
    // check for 4k granule and at least 36 bits physical address bus */

    use crate::println_debug;
    use crate::system::arm_core::registers::aarch64::general_sys_ctrl;
    use general_sys_ctrl::sctlr_el1::SctlrEl1;
    use general_sys_ctrl::tcr_el1::TcrEl1;
    use general_sys_ctrl::mair_el1 as memory_attributes;
    use general_sys_ctrl::id_aa64mmfr0_el1 as memory_model_features;

    let mm_feats = memory_model_features::read();
    
    if mm_feats.pa_range().value().expect("PAR should work") < memory_model_features::PhysicalAddressRangeSupport::_36Bits64GB {
        return Err(MMUInitError::PhysicalAddressRangeAtLeast36bitNotSupported);
    }
    if mm_feats.t_gran4() == memory_model_features::Granule4KBSupport::NotSupported {
        return Err(MMUInitError::TranslationGranule4kbNotSupported);
    }

    println_debug!("Checked support, everything should work! going ahead...");
    let table_ptr = (0x20_0000 as *mut TranslationTable4KB).wrapping_sub(1);
    unsafe {
        table_ptr.as_mut().unwrap_unchecked().init_lower_region();
    }

    println_debug!("Created table at {:p}", table_ptr);
    // const PAGE_ENTRY_COUNT: usize = PAGESIZE / core::mem::size_of::<usize>();
    // // granularity
    // const PT_PAGE: usize = 0b11; // 4k granule
    // const PT_BLOCK: usize = 0b01; // 2M granule
    //                               // accessibility
    // const PT_KERNEL: usize = 0 << 6; // privileged, supervisor EL1 access only
    // const PT_USER: usize = 1 << 6; // unprivileged, EL0 access allowed
    // const PT_RW: usize = 0 << 7; // read-write
    // const PT_RO: usize = 1 << 7; // read-only
    // const PT_AF: usize = 1 << 10; // accessed flag
    // const PT_NX: usize = 1 << 54; // no execute
    //                               // shareability
    // const PT_OSH: usize = 2 << 8; // outter shareable
    // const PT_ISH: usize = 3 << 8; // inner shareable
    //                               // defined in MAIR register
    // const PT_MEM: usize = 0 << 2; // normal memory
    // const PT_DEV: usize = 1 << 2; // device MMIO
    // const PT_NC: usize = 2 << 2; // non-cachable

    // const TTBR_ENABLE: usize = 1;

    // const MMIO_BASE: usize = peripherals::BCM_HOST.peripheral_address;

    // let data_page_index = unsafe { core::ptr::addr_of!(crate::__data_start) } as usize / PAGESIZE;
    // let page_table_ptr = unsafe { core::ptr::addr_of!(crate::__kernel_end) };

    // let page_table_entry_ptr = page_table_ptr.cast::<usize>().cast_mut();

    // /* create MMU translation tables at __kernel_end */

    // // TTBR0 Translation Table Base Register
    // // TTBR0, identity L1
    // let physical_address = page_table_ptr.wrapping_add(2 * PAGESIZE) as usize;

    // let flags = PT_PAGE |     // it has the "Present" flag, which must be set, and we have area in it mapped by pages
    // PT_AF |       // accessed flag. Without this we're going to have a Data Abort exception
    // PT_USER |     // non-privileged
    // PT_ISH |      // inner shareable
    // PT_MEM; // normal memory
    // unsafe {
    //     page_table_entry_ptr.write_volatile(physical_address | flags);
    // }
    // println_debug!("Level 1 Entry at {:p} = {:x} flags {:b}", page_table_entry_ptr, physical_address, flags);

    // // identity L2, first 2M block
    // let physical_address = page_table_ptr.wrapping_add(3 * PAGESIZE) as usize;
    // let flags = PT_PAGE |     // we have area in it mapped by pages
    //             PT_AF |       // accessed flag
    //             PT_USER |     // non-privileged
    //             PT_ISH |      // inner shareable
    //             PT_MEM; // normal memory
    // unsafe {
    //     page_table_entry_ptr
    //         .wrapping_add(2 * PAGE_ENTRY_COUNT)
    //         .write_volatile(physical_address | flags);
    // }
    // println_debug!("Level 2 Page Entry[0] at {:p} = {:x} flags {:b}", page_table_entry_ptr.wrapping_add(2 * PAGE_ENTRY_COUNT), physical_address, flags);

    // // identity L2 2M blocks
    // // TODO (astahl): this is 2032 on Pi 4's SoC, do we need to extend the block count up to 2048?
    // let device_memory_block = MMIO_BASE >> 21; // 2 megabyte blocks
    //                                            // skip 0th, as we're about to map it by L3
    // for block in 1..PAGE_ENTRY_COUNT {
    //     let physical_address = block << 21;
    //     let flags = PT_BLOCK |    // map 2M block
    //                 PT_AF |       // accessed flag
    //                 PT_NX |       // no execute
    //                 PT_USER |     // non-privileged
    //                 if block >= device_memory_block {
    //                     // different attributes for device memory
    //                     // TODO (astahl) this is never reached atm
    //                     PT_OSH | PT_DEV
    //                 } else {
    //                     PT_ISH | PT_MEM
    //                 };
    //     unsafe {
    //         page_table_entry_ptr
    //             .wrapping_add(2 * PAGE_ENTRY_COUNT + block)
    //             .write_volatile(physical_address | flags);
    //     }
    //     println_debug!("Level 2 Block Entry[{block}] at {:p} = {:x} flags {:b}", page_table_entry_ptr.wrapping_add(2 * PAGE_ENTRY_COUNT + block), physical_address, flags);
    // }

    // // identity L3
    // for page in 0..PAGE_ENTRY_COUNT {
    //     let physical_address = page * PAGESIZE;
    //     let flags = PT_PAGE |     // map 4k
    //                 PT_AF |       // accessed flag
    //                 PT_USER |     // non-privileged
    //                 PT_ISH |      // inner shareable
    //                 if page < 0x80 || page > data_page_index {
    //                     PT_RW | PT_NX
    //                 } else {
    //                     PT_RO
    //                 }; // different for code and data
    //     unsafe {
    //         page_table_entry_ptr
    //             .wrapping_add(3 * PAGE_ENTRY_COUNT + page)
    //             .write_volatile(physical_address | flags);
    //     }

    //     println_debug!("Level 3 Page Entry[{page}] at {:p} = {:x} flags {:b}", page_table_entry_ptr.wrapping_add(3 * PAGE_ENTRY_COUNT + page), physical_address, flags);
    // }

    // // TTBR1, kernel L1
    // let physical_address = page_table_ptr.wrapping_add(4 * PAGESIZE) as usize;
    // let flags = PT_PAGE |     // we have area in it mapped by pages
    //             PT_AF |       // accessed flag
    //             PT_KERNEL |   // privileged
    //             PT_ISH |      // inner shareable
    //             PT_MEM; // normal memory
    // unsafe {
    //     page_table_entry_ptr
    //         .wrapping_add(2 * PAGE_ENTRY_COUNT - 1)
    //         .write_volatile(physical_address | flags);
    // }
    // // kernel L2
    // let physical_address = page_table_ptr.wrapping_add(5 * PAGESIZE) as usize;
    // let flags = PT_PAGE |     // we have area in it mapped by pages
    //             PT_AF |       // accessed flag
    //             PT_KERNEL |   // privileged
    //             PT_ISH |      // inner shareable
    //             PT_MEM; // normal memory
    // unsafe {
    //     page_table_entry_ptr
    //         .wrapping_add(5 * PAGE_ENTRY_COUNT - 1)
    //         .write_volatile(physical_address | flags);
    // }
    // // kernel L3
    // // TODO (astahl): this is the UART0 address??
    // let physical_address = MMIO_BASE + 0x00201000;
    // let flags = PT_PAGE |     // map 4k
    //             PT_AF |       // accessed flag
    //             PT_NX |       // no execute
    //             PT_KERNEL |   // privileged
    //             PT_OSH |      // outter shareable
    //             PT_DEV; // device memory
    // unsafe {
    //     page_table_entry_ptr
    //         .wrapping_add(5 * PAGE_ENTRY_COUNT)
    //         .write_volatile(physical_address | flags);
    // }
    /* okay, now we have to set system registers to enable MMU */

    // A72 5.5 p. 5-280: You must set CPUECTLR.SMPEN to 1 before the caches and MMU are enabled, 
    // or any instruction cache or TLB maintenance operations are performed.

    // // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
    // let r: usize = (0xFF << 0) | // AttrIdx=0: normal, IWBWA, OWBWA, NTR
    //                (0x04 << 8) | // AttrIdx=1: device, nGnRE (must be OSH too)
    //                (0x44 <<16); // AttrIdx=2: non cacheable
    // unsafe {
    //     asm!("msr mair_el1, {}", in(reg) r);
    // }

    let mut memory_attributes_array = memory_attributes::MairEl1::zero();

    // index 0: Inner and Outer Normal Memory WriteBack Non-transient RW-Allocate (0b1111_1111)
    let write_back_non_transient_allocate = memory_attributes::NormalCacheType {
        write_policy: memory_attributes::CacheWritePolicy::WriteBack,
        transistence: memory_attributes::CacheTransistence::NonTransient,
        read_allocate_policy: memory_attributes::AllocatePolicy::Allocate,
        write_allocate_policy: memory_attributes::AllocatePolicy::Allocate
    };
    let memory_type = memory_attributes::NormalMemoryType { caching: Some(write_back_non_transient_allocate) };
    memory_attributes_array.set_attr_n(NORMAL_MEMORY_ATTR_IDX as usize, memory_attributes::MemoryAttributeDescriptor::Normal { outer: memory_type , inner: memory_type });

    // index 1: Device nGnRE
    let memory_type = memory_attributes::DeviceMemoryType::NGnRE;
    memory_attributes_array.set_attr_n(DEVICE_MEMORY_ATTR_IDX as usize, memory_attributes::MemoryAttributeDescriptor::Device { memory_type });

    // index 2: Inner and Outer Normal non cacheable
    // let memory_type = memory_attributes::NormalMemoryType { caching: None };
    // memory_attributes_array.set_attr_n(2, memory_attributes::MemoryAttributeDescriptor::Normal { outer: memory_type , inner: memory_type });

    memory_attributes_array.write_register();
    println_debug!("So far so good");

    let translate_control = TcrEl1::zero()
        .ips().set_value(mm_feats.pa_range().untyped().value()) // IPS= "autodetect" using the reported supported features flag
        .tbi1().clear() // no tagging, use top bit for address
        .tg1().set_value(0b10) // TG1=4k
        .sh1().set_value(0b11) // SH1= inner shareable
        .orgn1().set_value(0b01) // outer write back
        .irgn1().set_value(0b01) // outer write back
        .epd1().set() // DISABLE upper address half TTBR1
        .t1sz().set_value(25) // T1SZ=25, 3 levels (512G)
        .tbi0().clear() // no tagging, use top bit for address
        .tg0().set_value(0b10) // TG0=4k
        .sh0().set_value(0b11) // SH0= inner shareable
        .orgn0().set_value(0b01) // outer write back
        .irgn0().set_value(0b01) // outer write back
        .epd0().clear() // ENABLE lower address half TTBR0
        .t0sz().set_value(25) // T0SZ=25, 3 levels (512G)
        ;

    translate_control.write_register();
    unsafe {
        asm!("isb");
    }
    // // next, specify mapping characteristics in translate control register
    // let r: usize = (0 << 37) | 
    //                (mm_feats.pa_range().untyped().value() << 32) | // IPS=autodetected
    //                (0b10 << 30) | // TG1=4k
    //                (0b11 << 28) | // SH1=3 inner
    //                (0b01 << 26) | // ORGN1=1 write back
    //                (0b01 << 24) | // IRGN1=1 write back
    //                (0b0  << 23) | // EPD1 enable higher half
    //                (25   << 16) | // T1SZ=25, 3 levels (512G)
    //                (0b00 << 14) | // TG0=4k
    //                (0b11 << 12) | // SH0=3 inner
    //                (0b01 << 10) | // ORGN0=1 write back
    //                (0b01 << 8) |  // IRGN0=1 write back
    //                (0b0  << 7) |  // EPD0 enable lower half
    //                (25   << 0); // T0SZ=25, 3 levels (512G)
    // unsafe {
    //     asm!("msr tcr_el1, {}; isb", in(reg)r);
    // }
    println_debug!("TCR is set {}", translate_control);
    // tell the MMU where our translation tables are. TTBR_ENABLE bit not documented, but required
    // lower half, user space
    let ttbr0_address = table_ptr as u64 | 1;
    unsafe {
        asm!("msr ttbr0_el1, {}", in(reg) ttbr0_address);
    }
    println_debug!("TTBR0 is set {:#x}", ttbr0_address);
    // upper half, kernel space
    // let ttbr1_address = page_table_ptr as usize + PAGESIZE | TTBR_ENABLE;
    // unsafe {
    //     asm!("msr ttbr1_el1, {}", in(reg) ttbr1_address);
    // }
    // println_debug!("TTBR1 is set {:#x}", ttbr1_address);

    let sctlr = SctlrEl1::load_register()
        .span().set()   // No FEAT_PAN -> RES1
        .tscxt().set()  // No FEAT_CSV2_2  -> RES1
        .lsmaoe().set() // No FEAT_LSMAOC -> RES1
        .n_tlsmd().set()// No FEAT_LSMAOC -> RES1
        .eos().set()    // No FEAT_ExS -> RES1
        .eis().set()    // No FEAT_ExS -> RES1
        .ee().clear()   // little endian translation tables
        .e0e().clear()  // little endian translation tables
        .wxn().clear()
    //    .i().clear()    // no i-cache
        .i().set()
    //    .sa0().clear()  // no stack pointer alignment check at EL0
        .sa0().set()
    //    .sa().clear()   // no stack pointer alignment check at EL1
        .sa().set()
    //    .c().clear()    // no d-cache
        .c().set()
    //    .a().clear()    // no data alignment check
        .a().set()
        .m().set()      // enable MMU
        ;

    //println_debug!("About to set SCTLR_EL1 {:#?}", sctlr);
    sctlr.write_register();    
    // finally, toggle some bits in system control register to enable page translation
    // let mut r: usize = 0;
   
    // println_debug!("SCTLR is set {:#x}", r);
    // r |= 0xC00800; // set mandatory reserved bits
    // r &= !((1 << 25) |   // clear EE, little endian translation tables
    //        (1 << 24) |   // clear E0E
    //        (1 << 19) |   // clear WXN
    //        (1 << 12) |   // clear I, no instruction cache
    //        (1 << 4) |    // clear SA0, no Stack Pointer Alignment check at EL0
    //        (1 << 3) |    // clear SA, no Stack Pointer Alignment check at EL1
    //        (1 << 2) |    // clear C, no cache at all
    //        (1 << 1)); // clear A, no aligment check
    // r |= 1 << 0; // set M, enable MMU
    println_debug!("And now SCTLR is set {}", sctlr);
    Ok(())
}


/// An IA (Input Address) larger than 48 bits requires that the Effective value of TCR_ELx.DS is 1.
/// 
/// For the 4KB translation granule, when a stage 1 translation table walk is started, the initial lookup level is determined by the value of the TCR_ELx.TnSZ field as shown in the following table.
/// 
/// | Initial lookup level | TnSZ minimum value | Maximum IA bits resolved | TnSZ maximum value | Minimum IA bits resolved | Additional Requirements            |
/// | :------------------- | :------------------| :------------------------| :------------------| :------------------------| :--------------------------------- |
/// | -1                   | 12                 | IA\[51:12]               | 15                 | IA\[48:12]               | Effective value of TCR_ELx.DS is 1 |
/// | 0                    | 16                 | IA\[47:12]               | 24                 | IA\[39:12]               | -                                  |
/// | 1                    | 25                 | IA\[38:12]               | 33                 | IA\[30:12]               | -                                  |
/// | 2                    | 34                 | IA\[29:12]               | 39                 | IA\[24:12]               | -                                  |
/// | 2                    | 40                 | IA\[23:12]               | 42                 | IA\[21:12]               | FEAT_TTST is implemented           |
/// | 3                    | 43                 | IA\[20:12]               | 48                 | IA\[15:12]               | FEAT_TTST is implemented           |
/// 
#[repr(align(4096))]
struct TranslationTable4KB {
    /// # Level 0
    /// 
    /// Table descriptors and Block descriptors (512 GB addressed by Blocks, supported only for 52 bit IA).
    /// 
    /// Indexed by IA\[47:39], maximum 512 entries. (top bit (bit 48) selects address Range (I think))
    /// 
    /// Block attributes:
    /// * If the Effective value of TCR_ELx.DS is 0, then Block descriptors are not supported on this level.
    /// * If the Effective value of TCR_ELx.DS is 1:
    ///     - The Size of memory region addressed by Block descriptor is 512 GByte.
    ///     - The Bit range that is direct mapped is IA\[38:0] to OA\[38:0].
    ///     - The Block descriptor is selected by IA\[51:39].
    level0: [TableDescriptor;512],
    
    /// # Level 1
    /// 
    /// Table descriptors and Block descriptors (1 GB Blocks).
    /// 
    /// Indexed by IA\[38:30], maximum 512 entries.
    /// 
    /// Block attributes:
    /// 
    /// * The Size of memory region addressed by Block descriptor is 1 GByte.
    /// * The Bit range that is direct mapped is IA\[29:0] to OA\[29:0].
    /// * The Block descriptor is selected by IA bit range:
    ///     - If the Effective value of TCR_ELx.DS is 0, then IA\[47:30].
    ///     - If the Effective value of TCR_ELx.DS is 1, then IA\[51:30].
    level1: [u64;512],
    
    /// # Level 2
    /// 
    /// Table descriptors and Block descriptors (2 MB Blocks).
    /// 
    /// Indexed by IA\[29:21], maximum 512 entries.
    /// 
    /// Block attributes:
    /// 
    /// * The Size of memory region addressed by Block descriptor is 2 MByte.
    /// * The Bit range that is direct mapped is IA\[20:0] to OA\[20:0].
    /// * The Block descriptor is selected by IA bit range:
    ///     - If the Effective value of TCR_ELx.DS is 0, then IA\[47:21].
    ///     - If the Effective value of TCR_ELx.DS is 1, then IA\[51:21].
    level2: [u64;512],
    
    /// # Level 3
    /// 
    /// Page descriptors (4 KB Pages).
    /// 
    /// Indexed by IA\[20:12], maximum 512 entries.
    /// 
    /// * The page size is 4KB.
    /// * The translation can resolve a page using one of the following maximum address ranges:
    ///     - If the Effective value of TCR_ELx.DS is 0, then IA\[47:12].
    ///     - If the Effective value of TCR_ELx.DS is 1, then IA\[51:12].
    /// * The page is addressed by one of the following:
    ///     - If the Effective value of TCR_ELx.DS is 0, then OA\[47:12].
    ///     - If the Effective value of TCR_ELx.DS is 1, then OA\[51:12].
    /// 
    /// IA\[11:0] is mapped directly to OA[\11:0].
    level3_0: [PageDescriptor;512],
}

impl TranslationTable4KB {
    const PAGE_SIZE: usize = 0x1000;
    const L2_BLOCK_SIZE: usize = 0x200000;

    /// Initialize a table to use with TTBR0 mapping virtual address range [0x0 ... 0x0000_FFFF_FFFF_FFFF] (T0SZ = 16).
    /// 
    /// Sets pages in the stack range to no-execute.
    /// 
    /// Sets blocks in Peripheral Range to device memory.
    pub fn init_lower_region(&mut self) {
        use descriptors::*;
        const ADDRESSING: AddressingMode = AddressingMode::Gran4KBAddr48bit;
        // LEVEL 0 
        // init Level 0 (might not be necessary, depending on T0SZ)
        // map first 512 GB to the first entry in the next table 
        self.level0[0] = TableDescriptor::default().with_next_level_table_at(self.level1.as_ptr() as u64, ADDRESSING);
        // reject addresses over 512 GB
        self.level0[1..].fill(TableDescriptor::invalid());
        
        // LEVEL 1
        // Map first 1 GB to the next table
        self.level1[0] = TableDescriptor::default().with_next_level_table_at(self.level2.as_ptr() as u64, ADDRESSING).to_underlying();
        // reject addresses over 1 GB
        self.level1[1..].fill(TableDescriptor::invalid().to_underlying());

        // LEVEL 2 First 1 GB
        // Here we map each block of 2MB of IA\[29:12] directly to each OA via blocks
        // Map first 2 MB to the next table, so we can set no execute for the stack
        self.level2[0] = TableDescriptor::default().with_next_level_table_at(self.level3_0.as_ptr() as u64, ADDRESSING).to_underlying();
        // Map the rest of 2 MB Blocks to output addresses
        const BLOCK_SIZE: usize = TranslationTable4KB::L2_BLOCK_SIZE;
        const PERIPHERAL_BLOCKS_BEGIN: usize = BCM_HOST.peripheral_address / BLOCK_SIZE;
        const PERIPHERAL_BLOCKS_END: usize = (BCM_HOST.peripheral_address + BCM_HOST.peripheral_size + (BLOCK_SIZE - 1)) / BLOCK_SIZE;
        
        assert!(PERIPHERAL_BLOCKS_BEGIN < 512, "Not yet implemented peripheral handling beyond 1 GB");
        assert!(PERIPHERAL_BLOCKS_END <= 512, "Not yet implemented peripheral handling beyonf 1 GB");

        for i in 1..PERIPHERAL_BLOCKS_BEGIN {
            let output_address = i * BLOCK_SIZE;
            self.level2[i] = BlockDescriptor::default()
                .with_output_address(output_address as u64, ADDRESSING, BlockLevel::Level2)
                .contiguous().set()
                .sh().set_value(Shareability::InnerShareable)
                .stage_1_mem_attr_indx().set_value(NORMAL_MEMORY_ATTR_IDX)
                .to_underlying();
        }
        for i in PERIPHERAL_BLOCKS_BEGIN..PERIPHERAL_BLOCKS_END {
            let output_address = i * BLOCK_SIZE;
            self.level2[i] = BlockDescriptor::default()
                .with_output_address(output_address as u64, ADDRESSING, BlockLevel::Level2)
                .contiguous().set()
                .sh().set_value(Shareability::OuterShareable)
                .stage_1_mem_attr_indx().set_value(DEVICE_MEMORY_ATTR_IDX)
                .to_underlying();
        }

        // LEVEL 3 First 2 MB
        // Here we map each page of 4KB of IA\[20:12] directly to each OA \[20:12]
        let main_stack_range = MemoryMap::main_stack();
        let stack_pages_begin = main_stack_range.bottom() as usize / Self::PAGE_SIZE;
        let stack_pages_end = main_stack_range.top() as usize / Self::PAGE_SIZE;

        for i in 0..512 {
            let output_address = i * Self::PAGE_SIZE;
            self.level3_0[i] = PageDescriptor::default()
                .with_output_address(output_address as u64, ADDRESSING)
                .contiguous().set();
        }

        for i in stack_pages_begin..stack_pages_end {
            self.level3_0[i] = self.level3_0[i].xn_uxn_pxn().set_value(0b10);
        }
    }
}
