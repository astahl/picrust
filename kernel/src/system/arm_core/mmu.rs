pub mod descriptors;

use core::arch::asm;

use mystd::byte_value::ByteValue;

use crate::system::peripherals::BCM_HOST;
use crate::system::{arm_core::mmu::descriptors::AddressingMode, hal::info::MemoryMap};

use self::descriptors::{BlockDescriptor, PageDescriptor, TableDescriptor};

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

const MEMORY_ATTR_IDX_NORMAL: u64 = 0;
const MEMORY_ATTR_IDX_DEVICE: u64 = 1;
const MEMORY_ATTR_IDX_NON_CACHEABLE: u64 = 2;

#[cfg(feature = "mmu")]
pub fn mmu_init() -> Result<(), MMUInitError> {
    use crate::print_init;
    // check for 4k granule and at least 36 bits physical address bus */
    use crate::system::arm_core::mmu::descriptors::Shareability;
    use crate::system::arm_core::registers::aarch64::general_sys_ctrl;
    use crate::system::arm_core::registers::aarch64::general_sys_ctrl::id_aa64mmfr0_el1::AsidBitNum;
    use crate::system::arm_core::registers::aarch64::general_sys_ctrl::tcr_el1::GranuleSize;
    use general_sys_ctrl::id_aa64mmfr0_el1 as memory_model_features;
    use general_sys_ctrl::mair_el1 as memory_attributes;
    use general_sys_ctrl::sctlr_el1::SctlrEl1;
    use general_sys_ctrl::tcr_el1::TcrEl1;

    let mm_feats = memory_model_features::IdAa64Mmfr0El1::read_register();

    if mm_feats.pa_range().value().expect("PAR should work")
        < memory_model_features::PhysicalAddressRangeSupport::_36Bits64GB
    {
        return Err(MMUInitError::PhysicalAddressRangeAtLeast36bitNotSupported);
    }
    if mm_feats.t_gran4() == memory_model_features::Granule4KBSupport::NotSupported {
        return Err(MMUInitError::TranslationGranule4kbNotSupported);
    }
    print_init!("before table");
    let table = unsafe {
        let table_ptr = ByteValue::from_mibi(2).as_bytes() as *mut TranslationTable4KB;
        TranslationTable4KB::init(table_ptr)
        // for ia in 0..(1024 * 1024) {
        //     let ia = ia * 1024;
        //     let oa = (*table_ptr).simulate_walk(ia).expect("Table Walk should complete");
        //     println_debug!("Table Walk IA: {:?}, OA: {:?}", ia, oa);
        //     assert_eq!(ia, oa, "WALK FAILED");
        // }
        // panic!("hold it");
    };

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

    // index 0: Inner and Outer Normal Memory WriteBack Non-transient RW-Allocate (0b1111_1111)
    let write_back_non_transient_allocate = memory_attributes::NormalCacheType {
        write_policy: memory_attributes::CacheWritePolicy::WriteBack,
        transience: memory_attributes::CacheTransience::NonTransient,
        read_allocate_policy: memory_attributes::AllocatePolicy::Allocate,
        write_allocate_policy: memory_attributes::AllocatePolicy::Allocate,
    };
    let normal_memory_type = memory_attributes::NormalMemoryType {
        caching: Some(write_back_non_transient_allocate),
    };
    let device_memory_type = memory_attributes::DeviceMemoryType::NGnRE;
    let uncached_memory_type = memory_attributes::NormalMemoryType { caching: None };
    memory_attributes::MairEl1::zero()
        .set(
            MEMORY_ATTR_IDX_NORMAL,
            memory_attributes::MemoryAttributeDescriptor::normal(
                normal_memory_type,
                normal_memory_type,
            ),
        )
        .set(
            MEMORY_ATTR_IDX_DEVICE,
            memory_attributes::MemoryAttributeDescriptor::device(device_memory_type),
        )
        .set(
            MEMORY_ATTR_IDX_NON_CACHEABLE,
            memory_attributes::MemoryAttributeDescriptor::normal(
                uncached_memory_type,
                uncached_memory_type,
            ),
        )
        .write_register();
    
    let common_cacheability =
        general_sys_ctrl::tcr_el1::RegionCacheability::WriteBackReadAllocateWriteAllocate;
    let common_shareability = Shareability::InnerShareable;
    let common_granule = GranuleSize::_4KB;
    let common_tnsz = 24;
    TcrEl1::zero()
        .asid_size()
        .set_value(match mm_feats.asid().value() {
            Ok(AsidBitNum::_16Bits) => general_sys_ctrl::tcr_el1::AsidSize::_16Bit,
            Ok(AsidBitNum::_8Bits) => general_sys_ctrl::tcr_el1::AsidSize::_8Bit,
            Err(_) => panic!("ASID should've been reported correctly by id reg"),
        })
        .ips()
        .set_value(mm_feats.pa_range().value().unwrap()) // IPS= "autodetect" using the reported supported features flag
        .tbi1()
        .clear() // no tagging, use top bit for address
        .tg1()
        .set_value(common_granule) // TG1=4k
        .sh1()
        .set_value(common_shareability) // SH1= inner shareable
        .orgn1()
        .set_value(common_cacheability) // outer write back
        .irgn1()
        .set_value(common_cacheability) // inner write back
        .epd1()
        .clear() // ENABLE upper address half TTBR1
        .t1sz()
        .set_value(common_tnsz) // T1SZ=25, 3 levels (512G)
        .tbi0()
        .clear() // no tagging, use top bit for address
        .tg0()
        .set_value(common_granule) // TG0=4k
        .sh0()
        .set_value(common_shareability) // SH0= inner shareable
        .orgn0()
        .set_value(common_cacheability) // outer write back
        .irgn0()
        .set_value(common_cacheability) // inner write back
        .epd0()
        .clear() // ENABLE lower address half TTBR0
        .t0sz()
        .set_value(common_tnsz) // T0SZ=25, 3 levels (512G)
        .write_register();
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
    // tell the MMU where our translation tables are. TTBR_ENABLE bit not documented, but required
    // lower half, user space
    unsafe {
        asm!("msr ttbr0_el1, {}", in(reg) table.base_address_rg0());
        asm!("msr ttbr1_el1, {}", in(reg) table.base_address_rg1());
    }
    //println_debug!("TTBR0 is set {:#x}", ttbr0_address);
    // upper half, kernel space
    // let ttbr1_address = page_table_ptr as usize + PAGESIZE | TTBR_ENABLE;
    // unsafe {
    //     asm!("msr ttbr1_el1, {}", in(reg) ttbr1_address);
    // }
    // println_debug!("TTBR1 is set {:#x}", ttbr1_address);

    let sctlr = SctlrEl1::read_register_ordered_ish();

   let sctlr = sctlr
        .ee().clear()   // little endian translation tables
        .e0e().clear()  // little endian translation tables
        .wxn().clear()
    //    .i().clear()    // no i-cache
        .i().set()
        .sa0().clear()  // no stack pointer alignment check at EL0
    //    .sa0().set()
        .sa().clear()   // no stack pointer alignment check at EL1
    //    .sa().set()
    //    .c().clear()    // no d-cache
        .c().set()
        .a().clear()    // no data alignment check
    //    .a().set()
        .m().set()      // enable MMU
        ;

    //unsafe { asm!("BRK #1") }
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
    print_init!("End of MMU init");
    Ok(())
}

#[derive(Clone, Copy)]
union BlockOrTableDescriptor {
    _tag: u64,
    block: BlockDescriptor,
    table: TableDescriptor,
}

impl From<TableDescriptor> for BlockOrTableDescriptor {
    fn from(value: TableDescriptor) -> Self {
        Self { table: value }
    }
}

impl From<BlockDescriptor> for BlockOrTableDescriptor {
    fn from(value: BlockDescriptor) -> Self {
        Self { block: value }
    }
}

impl BlockOrTableDescriptor {
    const TAG_TABLE: u8 = 0b11;
    const TAG_BLOCK: u8 = 0b01;

    fn valid_bit(self) -> u8 {
        unsafe { (self._tag & 0b1) as u8 }
    }

    fn masked_tag(self) -> u8 {
        unsafe { (self._tag & 0b11) as u8 }
    }

    pub fn invalid() -> Self {
        Self { _tag: 0 }
    }

    pub fn is_invalid(self) -> bool {
        self.valid_bit() == 0
    }

    pub fn table_descriptor(self) -> Option<TableDescriptor> {
        match self.masked_tag() {
            Self::TAG_TABLE => Some(unsafe { self.table }),
            _ => None,
        }
    }

    pub fn block_descriptor(self) -> Option<BlockDescriptor> {
        match self.masked_tag() {
            Self::TAG_BLOCK => Some(unsafe { self.block }),
            _ => None,
        }
    }
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
    range_0_level_0: [TableDescriptor; 512],
    range_1_level_0: [TableDescriptor; 512],

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
    range_0_level_1: [BlockOrTableDescriptor; 512],
    range_1_level_1: [BlockOrTableDescriptor; 512],

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
    /// 
    /// there are 8 arrays allocated to support the maximum amount of 8 GB addressable SDRAM
    range_0_level_2: [[BlockOrTableDescriptor; 512]; 8],
    range_1_level_2: [[BlockOrTableDescriptor; 512]; 8],

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
    range_0_level_3: [[PageDescriptor; 512];1],
    range_1_level_3: [[PageDescriptor; 512];1],
}

impl TranslationTable4KB {
    const ADDRESSING: AddressingMode = AddressingMode::Gran4KBAddr48bit;
    const L0_BLOCK_SIZE: u64 = ByteValue::from_gibi(512).as_bytes();
    const L1_BLOCK_SIZE: u64 = ByteValue::from_gibi(1).as_bytes();
    const L2_BLOCK_SIZE: u64 = ByteValue::from_mibi(2).as_bytes();
    const PAGE_SIZE: u64 = ByteValue::from_kibi(4).as_bytes();

    const ENTRY_COUNT: u64 = 512;

    const PERIPHERAL_BLOCKS_RANGE_INCLUSIVE: (u64, u64) = (
        BCM_HOST.peripheral_range_inclusive.0 as u64 / Self::L2_BLOCK_SIZE,
        BCM_HOST.peripheral_range_inclusive.1 as u64 / Self::L2_BLOCK_SIZE,
    );
   
    pub fn base_address_rg0(&self) -> u64 {
        self.range_0_level_0.as_ptr() as u64
    }

    pub fn base_address_rg1(&self) -> u64 {
        self.range_1_level_0.as_ptr() as u64
    }

    /// Initialize a table to use with TTBR0 mapping virtual address range [0x0 ... 0x0000_FFFF_FFFF_FFFF] (T0SZ = 16).
    ///
    /// Sets pages in the stack range to no-execute.
    ///
    /// Sets blocks in Peripheral Range to device memory.
    pub unsafe fn init<'a>(ptr: *mut TranslationTable4KB) -> &'a TranslationTable4KB {
        (*ptr).initialize_level_0();
        (*ptr).initialize_level_1(8);
        (*ptr).initialize_level_2(8);
        (*ptr).initialize_level_3();

        ptr.as_ref()
            .expect("Surely you wouldn't pass in an unaligned or null pointer...")

        // // LEVEL 1
        // // Map first 1 GB to the next table
        // self.level1[0].table = TableDescriptor::default()
        //     .with_next_level_table_at(self.level2.as_ptr() as u64, ADDRESSING);
        // // // reject addresses over 1 GB
        // self.level1[1..].fill(BlockOrTableDescriptor::invalid());

        // for i in 0..512 {
        //     let output_address = i * 1024 * 1024 * 1024;
        //     self.level1[i].block = BlockDescriptor::default()
        //         .with_output_address(output_address as u64, ADDRESSING, BlockLevel::Level1)
        //         .af().set()
        //         //.ap_s2ap().set_value(1)
        //         //.dbm().set()
        //         //.stage_1_ns_secure().set()
        //         //.contiguous().set()
        //         .sh().set_value(Shareability::OuterShareable)
        //         .stage_1_mem_attr_indx().set_value(MEMORY_ATTR_IDX_NON_CACHEABLE);
        // }
        // // for i in 0..512 {
        // //     let output_address = i * 1024 * 1024 * 1024;
        // //     self.level1[i].block = BlockDescriptor::default()
        // //         .with_output_address(output_address as u64, ADDRESSING, BlockLevel::Level1)
        // //         .af().set()
        // //         //.dbm().set()
        // //         //.stage_1_ns_secure().set()
        // //         //.contiguous().set()
        // //         .sh().set_value(Shareability::OuterShareable)
        // //         .stage_1_mem_attr_indx().set_value(DEVICE_MEMORY_ATTR_IDX);
        // // }
        // return;

        // // LEVEL 2 First 1 GB
        // // Here we map each block of 2MB of IA\[29:12] directly to each OA via blocks
        // // Map first 2 MB to the next table, so we can set no execute for the stack
        // self.level2[0].table = TableDescriptor::default()
        //     .with_next_level_table_at(self.level3_0.as_ptr() as u64, ADDRESSING);
        // // Map the rest of 2 MB Blocks to output addresses
        // const BLOCK_SIZE: usize = TranslationTable4KB::L2_BLOCK_SIZE;
        // const PERIPHERAL_BLOCKS_BEGIN: usize = BCM_HOST.peripheral_address / BLOCK_SIZE;
        // const PERIPHERAL_BLOCKS_END: usize = (BCM_HOST.peripheral_address + BCM_HOST.peripheral_size + (BLOCK_SIZE - 1)) / BLOCK_SIZE;

        // assert!(PERIPHERAL_BLOCKS_BEGIN < 512, "Not yet implemented peripheral handling beyond 1 GB");
        // assert!(PERIPHERAL_BLOCKS_END <= 512, "Not yet implemented peripheral handling beyonf 1 GB");

        // for i in 1..PERIPHERAL_BLOCKS_BEGIN {
        //     let output_address = i * BLOCK_SIZE;
        //     self.level2[i].block = BlockDescriptor::default()
        //         .with_output_address(output_address as u64, ADDRESSING, BlockLevel::Level2)
        //         .af().set()
        //         //.contiguous().set()
        //         .sh().set_value(Shareability::InnerShareable)
        //         .stage_1_mem_attr_indx().set_value(MEMORY_ATTR_IDX_NORMAL);
        // }
        // for i in PERIPHERAL_BLOCKS_BEGIN..PERIPHERAL_BLOCKS_END {
        //     let output_address = i * BLOCK_SIZE;
        //     self.level2[i].block = BlockDescriptor::default()
        //         .with_output_address(output_address as u64, ADDRESSING, BlockLevel::Level2)
        //         .af().set()
        //         //.contiguous().set()
        //         .sh().set_value(Shareability::OuterShareable)
        //         .stage_1_mem_attr_indx().set_value(MEMORY_ATTR_IDX_DEVICE);
        // }

        // // LEVEL 3 First 2 MB
        // // Here we map each page of 4KB of IA\[20:12] directly to each OA \[20:12]
        // let main_stack_range = MemoryMap::main_stack();
        // let stack_pages_begin = main_stack_range.bottom() as usize / Self::PAGE_SIZE;
        // let stack_pages_end = main_stack_range.top() as usize / Self::PAGE_SIZE;

        // for i in 0..512 {
        //     let output_address = i * Self::PAGE_SIZE;
        //     self.level3_0[i] = PageDescriptor::default()
        //         .with_output_address(output_address as u64, ADDRESSING)
        //         .sh().set_value(Shareability::OuterShareable)
        //         .af().set()
        //         //.contiguous().set();
        // }

        // for i in stack_pages_begin..stack_pages_end {
        //     self.level3_0[i] = self.level3_0[i].xn_uxn_pxn().set_value(0b10);
        // }
    }

    // LEVEL 0
    fn initialize_level_0(&mut self) {
        // init Level 0 (might not be necessary, depending on T0SZ)
        // map each 512 GB to the first entry in the next table
        let next_table_range0 = TableDescriptor::default()
            .with_next_level_table_at(self.range_0_level_1.as_ptr() as u64, Self::ADDRESSING);
        // // reject addresses over 512 GB
        self.range_0_level_0.fill(next_table_range0);

        let next_table_range1 = TableDescriptor::default()
            .with_next_level_table_at(self.range_1_level_1.as_ptr() as u64, Self::ADDRESSING);
        // // reject addresses over 512 GB
        self.range_1_level_0.fill(next_table_range1);
    }

    fn initialize_level_1(&mut self, gigabytes_supported: usize) {

        // we repeatedly map the whole range to the supported ram range.
        for i in 0..512 {
            let next_level_index = i % gigabytes_supported;
            let next_table_range0 = TableDescriptor::default()
                .with_next_level_table_at(self.range_0_level_2[next_level_index].as_ptr() as u64, Self::ADDRESSING);
            self.range_0_level_1[i] = next_table_range0.into();

            let next_table_range1 = TableDescriptor::default()
                .with_next_level_table_at(self.range_1_level_2[next_level_index].as_ptr() as u64, Self::ADDRESSING);
            self.range_1_level_1[i] = next_table_range1.into();
        }
    }

    fn initialize_level_2(&mut self, gigabytes_supported: usize) {
        // first we initialise all blocks to a default value
        for j in 0..gigabytes_supported {
            for i in 0..512 {
                let output_address = (i + j * 512) as u64 * Self::L2_BLOCK_SIZE;
                let default_block = BlockDescriptor::default()
                    .with_output_address(
                        output_address,
                        Self::ADDRESSING,
                        descriptors::BlockLevel::Level2,
                    )
                    .af()
                    .set()
                    .contiguous()
                    .set()
                    .sh()
                    .set_value(descriptors::Shareability::InnerShareable)
                    .stage_1_mem_attr_indx()
                    .set_value(MEMORY_ATTR_IDX_NORMAL);
                self.range_0_level_2[j][i] = default_block.into();
                self.range_1_level_2[j][i] = default_block.into();
            }
        }

        // then we mark the first 2 MB (where the kernel and stack are) as handled by pages in the 3rd level
        self.range_0_level_2[0][0] = TableDescriptor::default()
            .with_next_level_table_at(self.range_0_level_3[0].as_ptr() as u64, Self::ADDRESSING)
            .into();
        self.range_1_level_2[0][0] = TableDescriptor::default()
            .with_next_level_table_at(self.range_1_level_3[0].as_ptr() as u64, Self::ADDRESSING)
            .into();

        // then we mark the peripheral mmio range as device memory
        for block_nr in Self::PERIPHERAL_BLOCKS_RANGE_INCLUSIVE.0..=Self::PERIPHERAL_BLOCKS_RANGE_INCLUSIVE.1 {
            let j = block_nr as usize / 512;
            let i = block_nr as usize % 512;
            let output_address = block_nr * Self::L2_BLOCK_SIZE;
            let device_block = BlockDescriptor::default()
                .with_output_address(
                    output_address,
                    Self::ADDRESSING,
                    descriptors::BlockLevel::Level2,
                )
                .af()
                .set()
                .contiguous()
                .set()
                .sh()
                .set_value(descriptors::Shareability::OuterShareable)
                .stage_1_mem_attr_indx()
                .set_value(MEMORY_ATTR_IDX_DEVICE);
            self.range_0_level_2[j][i] = device_block.into();
            self.range_1_level_2[j][i] = device_block.into();
            
        }

    }

    fn initialize_level_3(&mut self) {
        // LEVEL 3 First 2 MB
        // Here we map each page of 4KB of IA\[20:12] directly to each OA \[20:12]
        let main_stack_range = MemoryMap::main_stack();
        let stack_pages_begin = main_stack_range.bottom() as u64 / Self::PAGE_SIZE;
        let stack_pages_end = main_stack_range.top() as u64 / Self::PAGE_SIZE;

        for i in 0..512 {
            let output_address = i * Self::PAGE_SIZE;
            let normal_page = PageDescriptor::default()
                .with_output_address(output_address, Self::ADDRESSING)
                .sh()
                .set_value(descriptors::Shareability::OuterShareable)
                .af()
                .set()
                .contiguous()
                .set();

            self.range_0_level_3[0][i as usize] = normal_page;
            self.range_1_level_3[0][i as usize] = normal_page;
        }

        for i in stack_pages_begin..stack_pages_end {
            // change the stack pages to execute never
            self.range_0_level_3[0][i as usize] = self.range_0_level_3[0][i as usize]
                .xn_uxn_pxn()
                .set_value(0b10);
            self.range_1_level_3[0][i as usize] = self.range_1_level_3[0][i as usize]
                .xn_uxn_pxn()
                .set_value(0b10);
        }
    }

    // pub fn simulate_walk(&self, input_address: u64) -> Result<u64, u8> {
    //     const MASK_8: u64 = 0xff;
    //     const MASK_9: u64 = 0x1ff;
    //     const MODE: AddressingMode = AddressingMode::Gran4KBAddr48bit;
    //     const LEVEL_0_OA_MASK: u64 = (1 << 39) - 1;
    //     const LEVEL_1_OA_MASK: u64 = (1 << 30) - 1;
    //     const LEVEL_2_OA_MASK: u64 = (1 << 21) - 1;
    //     const LEVEL_3_OA_MASK: u64 = (1 << 12) - 1;
    //     let level0idx = (input_address >> 39) & MASK_8;
    //     let level1idx = (input_address >> 30) & MASK_9;
    //     let level2idx = (input_address >> 21) & MASK_9;
    //     let level3idx = (input_address >> 12) & MASK_9;

    //     let table_entry = self.level0[level0idx as usize];
    //     let next_table_address = match table_entry.format().value().expect("all values are handled") {
    //         descriptors::TableOrBlockDescriptorFormat::Invalid00 => return Err(0),
    //         descriptors::TableOrBlockDescriptorFormat::Invalid10 => return Err(0),
    //         descriptors::TableOrBlockDescriptorFormat::Block => return Err(0),
    //         descriptors::TableOrBlockDescriptorFormat::Table => table_entry.next_level_table_address(MODE),
    //     };

    //     assert_eq!(next_table_address, self.level1.as_ptr() as u64);

    //     let table_entry = self.level1[level1idx as usize];
    //     if table_entry.is_invalid() {
    //         return Err(1);
    //     } else if let Some(block) = table_entry.block_descriptor() {
    //         return Ok(input_address & LEVEL_1_OA_MASK | block.output_address(MODE, descriptors::BlockLevel::Level1));
    //     }

    //     let next_table_address = table_entry.table_descriptor().expect("this should be a table descriptor now").next_level_table_address(MODE);
    //     assert_eq!(next_table_address, self.level2.as_ptr() as u64);

    //     let table_entry = self.level2[level2idx as usize];
    //     if table_entry.is_invalid() {
    //         return Err(2);
    //     } else if let Some(block) = table_entry.block_descriptor() {
    //         return Ok(input_address & LEVEL_2_OA_MASK | block.output_address(MODE, descriptors::BlockLevel::Level2));
    //     }

    //     let next_table_address = table_entry.table_descriptor().expect("this should be a table descriptor now").next_level_table_address(MODE);
    //     assert_eq!(next_table_address, self.level3_0.as_ptr() as u64);

    //     let page = self.level3_0[level3idx as usize];
    //     match page.format().value().expect("all values are handled") {
    //         descriptors::PageDescriptorFormat::Page => {
    //             return Ok(input_address & LEVEL_3_OA_MASK | page.output_address(MODE));
    //         },
    //         _ => Err(3)
    //     }

    // }
}
