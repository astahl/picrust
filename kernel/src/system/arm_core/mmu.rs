use core::{arch::asm, mem::transmute_copy};
use mystd::bit_field_type_definition;
use mystd::{bit_field, bitfield2::BitFieldError};

use crate::system::peripherals;

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

    use crate::system::arm_core::registers::aarch64::general_sys_ctrl;
    use general_sys_ctrl::id_aa64mmfr0_el1 as memory_model_features;

    let mm_feats = memory_model_features::read();
    
    if mm_feats.pa_range().value().expect("PAR should work") < memory_model_features::PhysicalAddressRangeSupport::_36Bits64GB {
        return Err(MMUInitError::PhysicalAddressRangeAtLeast36bitNotSupported);
    }
    if mm_feats.t_gran4() == memory_model_features::Granule4KBSupport::NotSupported {
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

    let data_page_index = unsafe { core::ptr::addr_of!(crate::__data_start) } as usize / PAGESIZE;
    let page_table_ptr = unsafe { core::ptr::addr_of!(crate::__kernel_end) };

    let page_table_entry_ptr = page_table_ptr.cast::<usize>().cast_mut();

    /* create MMU translation tables at __kernel_end */

    // TTBR0 Translation Table Base Register
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
                   (mm_feats.pa_range().untyped().value() << 32) | // IPS=autodetected
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
           (1 << 12) |   // clear I, no instruction cache
           (1 << 4) |    // clear SA0, no Stack Pointer Alignment check at EL0
           (1 << 3) |    // clear SA, no Stack Pointer Alignment check at EL1
           (1 << 2) |    // clear C, no cache at all
           (1 << 1)); // clear A, no aligment check
    r |= 1 << 0; // set M, enable MMU
    unsafe {
        asm!("msr sctlr_el1, {}; isb", in(reg) r);
    }
    Ok(())
}





bit_field!(TableDescriptor(u64) {
    /// # NSTable \[63]
    /// 
    /// Table descriptor bit\[63] is one of the following:
    /// 
    /// * For stage 1 translations in Secure state, the __NSTable__ field which determines the IPA or PA space used for translation tables in subsequent lookup levels. For more information, see Hierarchical control of Secure or Non-secure memory accesses on page D8-5867.
    /// * For stage 1 translations in Non-secure state, this bit is RES0.
    /// * For stage 1 translations in Realm state, this bit is RES0.
    /// * For stage 1 translations in Root state, this bit is RES0.
    /// * For stage 2 translations, this bit is RES0.
    /// 
    /// For stage 1 translations in the EL3 translation regime, the removal of NSTable in Root state is a change from the behavior of EL3 in Secure state.
    63 => ns,

    /// # APTable \[62:61]
    /// 
    /// Table descriptor bits\[62:61] are one of the following:
    /// 
    /// * For stage 1 translations, the __APTable__\[1:0] field which determines the access permissions limit for subsequent lookup levels.
    /// * For stage 2 translations, these bits are RES0.
    /// 
    /// For more information, see Hierarchical control of data access permissions on page D8-5869.
    /// 
    /// For EL1&0 stage 1 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then APTable\[0] is treated as 0 regardless of the actual value.
    /// 
    /// For more information, see Additional behavior when HCR_EL2.NV is 1 and HCR_EL2.NV1 is 1 on page D8-5909.
    62:61 => ap,
    /// # XNTable / UXNTable [60]
    /// 
    /// Table descriptor bit\[60] is one of the following:
    /// 
    /// * For stage 1 translations that support __one__ privilege level, the __XNTable__ field which determines the execute-never limit for subsequent lookup levels.
    /// * For stage 1 translations that support __two__ privilege levels, the __UXNTable__ field which determines the unprivileged execute-never limit for subsequent lookup levels at EL0.
    /// * For EL1&0 stage 1 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then the PXNTable field. For more information, see Additional behavior when HCR_EL2.NV is 1 and HCR_EL2.NV1 is 1 on page D8-5909.
    /// * For stage 2 translations, this bit is RES0.
    /// 
    /// For more information, see Hierarchical control of instruction execution permissions on page D8-5873.
    60 => xn_uxn,

    /// # PXNTable\[59]
    /// 
    /// Table descriptor bit\[59] is one of the following:
    /// 
    /// * For stage 1 translations that support one privilege level, RES0.
    /// * For stage 1 translations that support two privilege levels, the PXNTable field which determines the privileged execute-never limit for subsequent lookup levels.
    /// * For EL1&0 stage 1 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then RES0. For more information, see Additional behavior when HCR_EL2.NV is 1 and HCR_EL2.NV1 is 1 on page D8-5909.
    /// * For stage 2 translations, this bit is RES0.
    /// 
    /// For more information, see Hierarchical control of instruction execution permissions on page D8-5873.
    59 => pxn,

    // 58:51 => ignore,
    // 50 => RES0,
    
    // Next level table address
    // 
    // this field is too messy to expose directly, use the accessor methods
    // 49:2 => _next_level_table_address,
    
    /// 
    1:0 => format: enum DescriptorFormat {
        Invalid00 = 0b00,
        Invalid10 = 0b10,
        Block = 0b01,
        Table = 0b11,
    },
});

// Models the "Effective Value of TCR_ELx.DS". In reality it's a bit more complicated, 
// but until we support chips that support FEAT_LPA2 totally irrelevant
const TCR_ELX_DS: bool = crate::system::arm_core::features::FEAT_LPA2;
use crate::system::arm_core::features::FEAT_LPA;

impl TableDescriptor {
    pub const fn next_level_table_address_4kb_granule(self) -> u64 {
        if TCR_ELX_DS {
            const MASK_LSB: u64 = TableDescriptor::_field_mask(12,49);
            const MASK_MSB: u64 = TableDescriptor::_field_mask(8,9);
            const MSB_SHIFT: usize = 50 - 8;
            self.0 & MASK_LSB | ((self.0 & MASK_MSB) << MSB_SHIFT)
        } else {
            const MASK: u64 = TableDescriptor::_field_mask(12,47);
            self.0 & MASK
        }
    }

    pub const fn next_level_table_address_16kb_granule(self) -> u64 {
        if TCR_ELX_DS {
            const MASK_LSB: u64 = TableDescriptor::_field_mask(14,49);
            const MASK_MSB: u64 = TableDescriptor::_field_mask(8,9);
            const MSB_SHIFT: usize = 50 - 8;
            self.0 & MASK_LSB | ((self.0 & MASK_MSB) << MSB_SHIFT)
        } else {
            const MASK: u64 = TableDescriptor::_field_mask(14,47);
            self.0 & MASK
        }
    }

    pub const fn next_level_table_address_64kb_granule(self) -> u64 {
        if FEAT_LPA {
            const MASK_LSB: u64 = TableDescriptor::_field_mask(16,47);
            const MASK_MSB: u64 = TableDescriptor::_field_mask(12,15);
            const MSB_SHIFT: usize = 48 - 12;
            self.0 & MASK_LSB | ((self.0 & MASK_MSB) << MSB_SHIFT)
        } else {
            const MASK: u64 = TableDescriptor::_field_mask(16,47);
            self.0 & MASK
        }
    }

    pub const fn with_next_level_table_address_4kb_granule(self, address: u64) -> Self {
        if TCR_ELX_DS {
            const MASK_LSB: u64 = TableDescriptor::_field_mask(12,49);
            const MASK_MSB: u64 = TableDescriptor::_field_mask(50,51);
            const MSB_SHIFT: usize = 50 - 8;
            const MASK: u64 = MASK_MSB | TableDescriptor::_field_mask(8,9);
            let shifted_address = ((address & MASK_MSB) >> MSB_SHIFT) | (address & MASK_LSB);
            let value = (!MASK & self.0) | shifted_address;
            Self::new(value)
        } else {
            const MASK: u64 = TableDescriptor::_field_mask(12,47);
            let value = (!MASK & self.0) | (MASK & address);
            Self::new(value)
        }
    }

    pub const fn with_next_level_table_address_16kb_granule(self, address: u64) -> Self {
        if TCR_ELX_DS {
            const MASK_LSB: u64 = TableDescriptor::_field_mask(14,49);
            const MASK_MSB: u64 = TableDescriptor::_field_mask(50,51);
            const MSB_SHIFT: usize = 50 - 8;
            const MASK: u64 = MASK_MSB | TableDescriptor::_field_mask(8,9);
            let shifted_address = ((address & MASK_MSB) >> MSB_SHIFT) | (address & MASK_LSB);
            let value = (!MASK & self.0) | shifted_address;
            Self::new(value)
        } else {
            const MASK: u64 = TableDescriptor::_field_mask(14,47);
            let value = (!MASK & self.0) | (MASK & address);
            Self::new(value)
        }
    }

    pub const fn with_next_level_table_address_64kb_granule(self, address: u64) -> Self {
        if FEAT_LPA {
            const MASK_LSB: u64 = TableDescriptor::_field_mask(16,47);
            const MASK_MSB: u64 = TableDescriptor::_field_mask(48,51);
            const MSB_SHIFT: usize = 48 - 12;
            const MASK: u64 = MASK_MSB | TableDescriptor::_field_mask(12,15);
            let shifted_address = ((address & MASK_MSB) >> MSB_SHIFT) | (address & MASK_LSB);
            let value = (!MASK & self.0) | shifted_address;
            Self::new(value)
        } else {
            const MASK: u64 = TableDescriptor::_field_mask(14,47);
            let value = (!MASK & self.0) | (MASK & address);
            Self::new(value)
        }
    }
}

bit_field_type_definition!(0;1;enum DeviceAttribute<u64>{
    NGnRnE = 0b00,
    NGnRE = 0b01,
    NGRE = 0b10,
    Gre = 0b11,
});

bit_field_type_definition!(0;1;enum Cacheability<u64> {
    NotApplicable = 0b00,
    NonCacheable = 0b01,
    WriteThroughCacheable = 0b10,
    WriteBackCacheable = 0b11,
});

#[derive(Debug, Clone, Copy)]
enum Stage2MemoryAttr {
    Device(DeviceAttribute),
    Normal{
        outer: Cacheability,
        inner: Cacheability,
    }
}

impl Into<u64> for Stage2MemoryAttr {
    fn into(self) -> u64 {
        match self {
            Stage2MemoryAttr::Device(device_flags) => 
                device_flags as u64,
            Stage2MemoryAttr::Normal{outer, inner} =>
                ((outer as u64) << 2) | inner as u64,
        }
    }
}

impl TryFrom<u64> for Stage2MemoryAttr {
    type Error = BitFieldError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > 0b1111 {
            Err(BitFieldError::ValueTooLargeForField)
        } else {
            match (value >> 2, value & 0b11) {
                (0b00, device_flags) => Ok(Self::Device(device_flags.try_into().unwrap())),
                (normal_outer, normal_inner) => Ok(Self::Normal {
                    outer: normal_outer.try_into().unwrap(), 
                    inner: normal_inner.try_into().unwrap()
                })
            }
        }
    }
}

bit_field!(BlockDescriptor(u64) {
    // upper attributes

    /// # AMEC \[63]
    /// Block descriptor and Page descriptor bit\[63] is one of the following:
    /// * For stage 1 translations using the Non-secure translation regime, Secure translation regime, or Realm EL1&0 translation regime, this bit is IGNORED.
    /// * For stage 1 translations using the Realm EL2 or EL2&0 translation regimes, this bit is the AMEC field.
    /// * For stage 2 translations using the Non-secure translation regime or Secure translation regime, this bit is RES0. • For stage 2 translations using the Realm translation regime, this bit is the AMEC field.
    63 => amec,

    /// # PBHA \[62:59]
    /// If FEAT_HPDS2 is not implemented, then Block descriptor and Page descriptor bits\[62:59] are one of the following:
    /// * For stage 1 translations, the bits are IGNORED.
    /// * For stage 2 translations, all of the following:
    ///     - Bits\[62:60] are reserved for use by a System MMU.
    ///     - Bit\[59] is IGNORED.
    /// 
    /// If FEAT_HPDS2 is implemented, then Block descriptor and Page descriptor bits\[62:59] can be individually enabled as the Page-Based Hardware Attributes bits (PBHA\[3:0]).
    /// For more information, see Page Based Hardware attributes on page D8-5891.
    /// 
    /// If FEAT_HPDS2 is implemented and a PBHA bit is not enabled by TCR_ELx control bits, then the corresponding
    /// descriptor and Page descriptor bit is one of the following: 
    /// * For stage 1 translations, the bit is IGNORED.
    /// * For stage 2 translations, all of the following:
    ///     - For descriptor bits\[62:60], the bit is reserved for use by a System MMU.
    ///     - For descriptor bit\[59], the bit is IGNORED.
    62:59 => pbha,

    // 58:56 => ignore,

    /// # NS (Realm Security) \[55]
    /// Block descriptor and Page descriptor bit\[55] is one of the following: 
    /// * For stage 1 translations, IGNORED.
    /// * For stage 2 translations, one of the following:
    ///     - For the Realm Security state, the NS field.
    ///     - For all Security states other than Realm Security state, IGNORED.
    55 => ns_realm,

    /// # XN / UXN / PXN Flags \[54:53]
    /// 
    /// Block descriptor and Page descriptor bits\[54:53] are one of the following: 
    /// 
    /// * For stage 1 translations, bit\[54] is one of the following:
    ///     - If the translation regime supports only one privilege level, then the Execute-never field (XN).
    ///     - If the translation regime can support two privilege levels, then Unprivileged execute-never field (UXN).
    ///     - For EL1&0 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then the Privileged execute-never field (PXN).
    /// * For stage 1 translations, bit\[53] is one of the following:
    ///     - If the translation regime supports only one privilege level, then RES0.
    ///     - If the translation regime can support two privilege levels, then the Privileged execute-never field (PXN).
    ///     - For EL1&0 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then RES0. 
    /// * For stage 2 translations, bits\[54:53] are one of the following:
    ///     - If FEAT_XNX is not implemented, then bit\[54] is the Execute-never field (XN) and bit\[53] is RES0.
    ///     - If FEAT_XNX is implemented, then bits\[54:53] are the Execute-never field (XN\[1:0]).
    /// 
    /// For more information, see Instruction execution permissions on page D8-5870 and Additional behavior when
    /// HCR_EL2.NV is 1 and HCR_EL2.NV1 is 1 on page D8-5909.
    54:53 => xn_uxn_pxn,

    /// # Contiguous bit \[52]
    /// 
    /// Block descriptor and Page descriptor bit\[52] is the Contiguous bit.
    /// 
    /// The Contiguous bit identifies a descriptor as belonging to a group of adjacent translation table entries that point to a contiguous OA range.
    /// 
    /// For more information, see The Contiguous bit on page D8-5890.
    52 => contiguous,

    /// # Dirty Bit Modifier (DBM) \[51]
    /// 
    /// Block descriptor and Page descriptor bit\[51] is the Dirty Bit Modifier (DBM).
    /// 
    /// The dirty state is used to indicate a memory block or page has been modified. When hardware update of the dirty state is enabled, the descriptor DBM field indicates whether the descriptor is a candidate for hardware updates of the dirty state.
    /// 
    /// For more information, see Hardware management of the dirty state on page D8-5877.
    51 => dbm,

    /// # Guarded Page (GP) \[50]
    /// 
    /// Block descriptor and Page descriptor bit\[50] is one of the following:
    /// * If FEAT_BTI is not implemented, then Block descriptor and Page descriptor bit\[50] is RES0. 
    /// * If FEAT_BTI is implemented, then one of the following:
    ///     - For stage 1 translations, the Block descriptor and Page descriptor bit\[50] is Guarded Page field (GP).
    ///     - For stage 2 translations, the Block descriptor and Page descriptor bit\[50] is RES0.
    50 => gp,

    // output address
    // too messy, use accessors

    // lower attributes

    /// # nT \[16]
    /// __Block__ descriptor bit\[16] is one of the following:
    /// * If FEAT_BBM is not implemented, then Block descriptor bit\[16] is RES0.
    /// * If FEAT_BBM is not implemented, then Block descriptor bit\[16] is the nT field. For more information, see Block translation entry on page D8-5892.
    16 => n_t,

    /// # NSE / nG /FnXS \[11]
    /// 
    /// Block descriptor and Page descriptor bit\[11] is one of the following:
    /// * For stage 1 translations that support a single privilege level and are not in Root state, Block descriptor and Page descriptor bit\[11] is RES0.
    /// * For stage 1 translations that are in Root state, Block descriptor and Page descriptor bit\[11] is the NSE field.
    /// * For stage 1 translations that support two privilege levels, Block descriptor and Page descriptor bit\[11] is the not global bit (nG). 
    ///   The nG bit in a Block descriptor and Page descriptor indicates one of the following:
    ///     - If the value is 0, the translation is global and the TLB entry applies to all ASID values.
    ///     - If the value is 1, the translation is non-global and the TLB entry applies to only the current ASID value.
    ///     - For more information, see Global and process-specific translation table entries on page D8-5930.
    /// * For stage 2 translations, one of the following:
    ///     - If FEAT_XS is not implemented, then Block descriptor and Page descriptor bit\[11] is RES0.
    ///     - If FEAT_XS is implemented, then Block descriptor and Page descriptor bit\[11] is FnXS. 
    ///     For a stage 2 translation, the FnXS bit in a Block descriptor and Page descriptor has all of the following properties:
    ///         + If the FnXS bit is 0, then the XS attribute of the resultant memory translation is not modified.
    ///         + If the FnXS bit is 1, then the XS attribute of the resultant memory translation is set to 0.
    ///         + For more information, see XS attribute modifier on page D8-5892.
    11 => nse_ng_fnxs,

    /// # Access Flag (AF) \[10]
    /// The AF in a Block descriptor and Page descriptor indicates one of the following:
    /// * If the value is 0, then the memory region has not been accessed since the value of AF was last set to 0.
    /// * If the value is 1, then the memory region has been accessed since the value of AF was last set to 0.
    /// * For more information, see The Access flag on page D8-5875.
    10 => af,

    /// # Shareability (SH\[1:0]) \[9:8]
    /// 
    /// Block descriptor and Page descriptor bits\[9:8] are one of the following:
    /// 
    /// * For the 4KB and 16KB granules, one of the following:
    ///     - If the Effective value of TCR_ELx.DS is 0, then the Shareability field (SH\[1:0]).
    ///     - If the Effective value of TCR_ELx.DS is 1, then bits\[51:50] of the OA. 
    /// * For the 64KB translation granule, the Shareability field (SH\[1:0]).
    /// 
    /// For more information, see Stage 1 Shareability attributes on page D8-5884 and Stage 2 Shareability attributes on page D8-5888.
    9:8 => sh: enum Shareability {
        NonShareable = 0b00,
        Reserved = 0b01,
        OuterShareable = 0b10,
        InnerShareable = 0b11
    },

    /// # Access Permissions (AP \[2:1]) or Stage 2 Access Permissions (S2AP\[1:0]) \[7:6]
    /// 
    /// Block descriptor and Page descriptor bits\[7:6] are one of the following:
    /// * For stage 1 translations that support a single privilege level, all of the following:
    ///     - Bit\[7] is the data Access Permissions bit (AP\[2]).
    ///     - Bit\[6] is RES1.
    /// * For EL1 stage 1 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then all of the following:
    ///     - Bit\[7] is the data Access Permissions bit (AP\[2]).
    ///     - Bit\[6] is treated as 0 regardless of the actual value.
    /// * For stage 1 translations that support two privilege levels, the data Access Permissions bits (AP\[2:1]). 
    /// * For stage 2 translations, the stage 2 data Access Permissions field (S2AP\[1:0]).
    /// 
    /// For more information, see Data access permissions for stage 1 translations on page D8-5867, Data access permissions for stage 2 translations on page D8-5868, and Additional behavior when HCR_EL2.NV is 1 and HCR_EL2.NV1 is 1 on page D8-5909.
    /// 
    /// The Armv8 Block descriptor and Page descriptor format defines the data Access Permissions bits, AP\[2:1], and does not define an AP\[0] bit.
    7:6 => ap_s2ap,

    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 1 Secure State: Non-Secure Flag NS \[5]
    /// * For stage 1 translations bit\[5] is one of the following:
    ///     - When the access is from Secure state, from Realm state using the EL2 or EL2&0 translation regimes, or from Root state, the Non-secure bit (NS). For more information, see Controlling memory access Security state on page D8-5866.
    ///     - When the access is from Non-secure state, or from Realm state using the EL1&0 translation regime, the bit is RES0.
    5 => stage_1_ns_secure,

    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 1: Memory Attributes Index (AttrIndex\[2:0]) \[4:2]
    /// * For stage 1 translations, bits\[4:2] are the stage 1 memory attributes index field for the MAIR_ELx (AttrIndx\[2:0]). For more information, see Stage 1 memory type and Cacheability attributes on page D8-5883.    
    4:2 => stage_1_mem_attr_indx,

    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 2: Memory Attributes (MemAttr\[3:0]) \[5:2]
    /// * For stage 2 translations, if the Effective value of HCR_EL2.FWB is 0, then bits\[5:2] are the stage 2 memory attributes (MemAttr\[3:0]). For more information, see Stage 2 memory type and Cacheability attributes when FEAT_S2FWB is disabled on page D8-5885.
    5:2 => stage_2_mem_attr: Stage2MemoryAttr,

    /// # !!Not Implemented on A72!!
    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 2: Memory Attributes (MemAttr\[2:0]) \[4:2]
    /// * For stage 2 translations, if the Effective value of HCR_EL2.FWB is 1, then all of the following:
    ///     - Bit\[5] is RES0.
    ///     - Bits\[4:2] are the stage 2 memory attributes (MemAttr\[2:0]). For more information, see Stage 2 memory type and Cacheability attributes when FEAT_S2FWB is enabled on page D8-5887.
    4:2 => stage_2_fwb_mem_attr,

    // descriptor formats
    1:0 => format: DescriptorFormat,

    1:0 => level_3_format: enum Level3DescriptorFormat {
        Invalid00 = 0b00,
        Invalid01 = 0b01,
        Invalid10 = 0b10,
        Page = 0b11,
    }
});

bit_field!(PageDescriptor(u64) {
    // upper attributes

    /// # AMEC \[63]
    /// Block descriptor and Page descriptor bit\[63] is one of the following:
    /// * For stage 1 translations using the Non-secure translation regime, Secure translation regime, or Realm EL1&0 translation regime, this bit is IGNORED.
    /// * For stage 1 translations using the Realm EL2 or EL2&0 translation regimes, this bit is the AMEC field.
    /// * For stage 2 translations using the Non-secure translation regime or Secure translation regime, this bit is RES0. • For stage 2 translations using the Realm translation regime, this bit is the AMEC field.
    63 => amec,

    /// # PBHA \[62:59]
    /// If FEAT_HPDS2 is not implemented, then Block descriptor and Page descriptor bits\[62:59] are one of the following:
    /// * For stage 1 translations, the bits are IGNORED.
    /// * For stage 2 translations, all of the following:
    ///     - Bits\[62:60] are reserved for use by a System MMU.
    ///     - Bit\[59] is IGNORED.
    /// 
    /// If FEAT_HPDS2 is implemented, then Block descriptor and Page descriptor bits\[62:59] can be individually enabled as the Page-Based Hardware Attributes bits (PBHA\[3:0]).
    /// For more information, see Page Based Hardware attributes on page D8-5891.
    /// 
    /// If FEAT_HPDS2 is implemented and a PBHA bit is not enabled by TCR_ELx control bits, then the corresponding
    /// descriptor and Page descriptor bit is one of the following: 
    /// * For stage 1 translations, the bit is IGNORED.
    /// * For stage 2 translations, all of the following:
    ///     - For descriptor bits\[62:60], the bit is reserved for use by a System MMU.
    ///     - For descriptor bit\[59], the bit is IGNORED.
    62:59 => pbha,

    // 58:56 => ignore,

    /// # NS (Realm Security) \[55]
    /// Block descriptor and Page descriptor bit\[55] is one of the following: 
    /// * For stage 1 translations, IGNORED.
    /// * For stage 2 translations, one of the following:
    ///     - For the Realm Security state, the NS field.
    ///     - For all Security states other than Realm Security state, IGNORED.
    55 => ns_realm,

    /// # XN / UXN / PXN Flags \[54:53]
    /// 
    /// Block descriptor and Page descriptor bits\[54:53] are one of the following: 
    /// 
    /// * For stage 1 translations, bit\[54] is one of the following:
    ///     - If the translation regime supports only one privilege level, then the Execute-never field (XN).
    ///     - If the translation regime can support two privilege levels, then Unprivileged execute-never field (UXN).
    ///     - For EL1&0 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then the Privileged execute-never field (PXN).
    /// * For stage 1 translations, bit\[53] is one of the following:
    ///     - If the translation regime supports only one privilege level, then RES0.
    ///     - If the translation regime can support two privilege levels, then the Privileged execute-never field (PXN).
    ///     - For EL1&0 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then RES0. 
    /// * For stage 2 translations, bits\[54:53] are one of the following:
    ///     - If FEAT_XNX is not implemented, then bit\[54] is the Execute-never field (XN) and bit\[53] is RES0.
    ///     - If FEAT_XNX is implemented, then bits\[54:53] are the Execute-never field (XN\[1:0]).
    /// 
    /// For more information, see Instruction execution permissions on page D8-5870 and Additional behavior when
    /// HCR_EL2.NV is 1 and HCR_EL2.NV1 is 1 on page D8-5909.
    54:53 => xn_uxn_pxn,

    /// # Contiguous bit \[52]
    /// 
    /// Block descriptor and Page descriptor bit\[52] is the Contiguous bit.
    /// 
    /// The Contiguous bit identifies a descriptor as belonging to a group of adjacent translation table entries that point to a contiguous OA range.
    /// 
    /// For more information, see The Contiguous bit on page D8-5890.
    52 => contiguous,

    /// # Dirty Bit Modifier (DBM) \[51]
    /// 
    /// Block descriptor and Page descriptor bit\[51] is the Dirty Bit Modifier (DBM).
    /// 
    /// The dirty state is used to indicate a memory block or page has been modified. When hardware update of the dirty state is enabled, the descriptor DBM field indicates whether the descriptor is a candidate for hardware updates of the dirty state.
    /// 
    /// For more information, see Hardware management of the dirty state on page D8-5877.
    51 => dbm,

    /// # Guarded Page (GP) \[50]
    /// 
    /// Block descriptor and Page descriptor bit\[50] is one of the following:
    /// * If FEAT_BTI is not implemented, then Block descriptor and Page descriptor bit\[50] is RES0. 
    /// * If FEAT_BTI is implemented, then one of the following:
    ///     - For stage 1 translations, the Block descriptor and Page descriptor bit\[50] is Guarded Page field (GP).
    ///     - For stage 2 translations, the Block descriptor and Page descriptor bit\[50] is RES0.
    50 => gp,

    // output address
    // too messy, use accessors

    // lower attributes

    /// # NSE / nG /FnXS \[11]
    /// 
    /// Block descriptor and Page descriptor bit\[11] is one of the following:
    /// * For stage 1 translations that support a single privilege level and are not in Root state, Block descriptor and Page descriptor bit\[11] is RES0.
    /// * For stage 1 translations that are in Root state, Block descriptor and Page descriptor bit\[11] is the NSE field.
    /// * For stage 1 translations that support two privilege levels, Block descriptor and Page descriptor bit\[11] is the not global bit (nG). 
    ///   The nG bit in a Block descriptor and Page descriptor indicates one of the following:
    ///     - If the value is 0, the translation is global and the TLB entry applies to all ASID values.
    ///     - If the value is 1, the translation is non-global and the TLB entry applies to only the current ASID value.
    ///     - For more information, see Global and process-specific translation table entries on page D8-5930.
    /// * For stage 2 translations, one of the following:
    ///     - If FEAT_XS is not implemented, then Block descriptor and Page descriptor bit\[11] is RES0.
    ///     - If FEAT_XS is implemented, then Block descriptor and Page descriptor bit\[11] is FnXS. 
    ///     For a stage 2 translation, the FnXS bit in a Block descriptor and Page descriptor has all of the following properties:
    ///         + If the FnXS bit is 0, then the XS attribute of the resultant memory translation is not modified.
    ///         + If the FnXS bit is 1, then the XS attribute of the resultant memory translation is set to 0.
    ///         + For more information, see XS attribute modifier on page D8-5892.
    11 => nse_ng_fnxs,

    /// # Access Flag (AF) \[10]
    /// The AF in a Block descriptor and Page descriptor indicates one of the following:
    /// * If the value is 0, then the memory region has not been accessed since the value of AF was last set to 0.
    /// * If the value is 1, then the memory region has been accessed since the value of AF was last set to 0.
    /// * For more information, see The Access flag on page D8-5875.
    10 => af,

    /// # Shareability (SH\[1:0]) \[9:8]
    /// 
    /// Block descriptor and Page descriptor bits\[9:8] are one of the following:
    /// 
    /// * For the 4KB and 16KB granules, one of the following:
    ///     - If the Effective value of TCR_ELx.DS is 0, then the Shareability field (SH\[1:0]).
    ///     - If the Effective value of TCR_ELx.DS is 1, then bits\[51:50] of the OA. 
    /// * For the 64KB translation granule, the Shareability field (SH\[1:0]).
    /// 
    /// For more information, see Stage 1 Shareability attributes on page D8-5884 and Stage 2 Shareability attributes on page D8-5888.
    9:8 => sh: Shareability,

    /// # Access Permissions (AP \[2:1]) or Stage 2 Access Permissions (S2AP\[1:0]) \[7:6]
    /// 
    /// Block descriptor and Page descriptor bits\[7:6] are one of the following:
    /// * For stage 1 translations that support a single privilege level, all of the following:
    ///     - Bit\[7] is the data Access Permissions bit (AP\[2]).
    ///     - Bit\[6] is RES1.
    /// * For EL1 stage 1 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then all of the following:
    ///     - Bit\[7] is the data Access Permissions bit (AP\[2]).
    ///     - Bit\[6] is treated as 0 regardless of the actual value.
    /// * For stage 1 translations that support two privilege levels, the data Access Permissions bits (AP\[2:1]). 
    /// * For stage 2 translations, the stage 2 data Access Permissions field (S2AP\[1:0]).
    /// 
    /// For more information, see Data access permissions for stage 1 translations on page D8-5867, Data access permissions for stage 2 translations on page D8-5868, and Additional behavior when HCR_EL2.NV is 1 and HCR_EL2.NV1 is 1 on page D8-5909.
    /// 
    /// The Armv8 Block descriptor and Page descriptor format defines the data Access Permissions bits, AP\[2:1], and does not define an AP\[0] bit.
    7:6 => ap_s2ap,

    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 1 Secure State: Non-Secure Flag NS \[5]
    /// * For stage 1 translations bit\[5] is one of the following:
    ///     - When the access is from Secure state, from Realm state using the EL2 or EL2&0 translation regimes, or from Root state, the Non-secure bit (NS). For more information, see Controlling memory access Security state on page D8-5866.
    ///     - When the access is from Non-secure state, or from Realm state using the EL1&0 translation regime, the bit is RES0.
    5 => stage_1_ns_secure,

    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 1: Memory Attributes Index (AttrIndex\[2:0]) \[4:2]
    /// * For stage 1 translations, bits\[4:2] are the stage 1 memory attributes index field for the MAIR_ELx (AttrIndx\[2:0]). For more information, see Stage 1 memory type and Cacheability attributes on page D8-5883.    
    4:2 => stage_1_mem_attr_indx,

    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 2: Memory Attributes (MemAttr\[3:0]) \[5:2]
    /// * For stage 2 translations, if the Effective value of HCR_EL2.FWB is 0, then bits\[5:2] are the stage 2 memory attributes (MemAttr\[3:0]). For more information, see Stage 2 memory type and Cacheability attributes when FEAT_S2FWB is disabled on page D8-5885.
    5:2 => stage_2_mem_attr: Stage2MemoryAttr,

    /// # !!Not Implemented on A72!!
    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 2: Memory Attributes (MemAttr\[2:0]) \[4:2]
    /// * For stage 2 translations, if the Effective value of HCR_EL2.FWB is 1, then all of the following:
    ///     - Bit\[5] is RES0.
    ///     - Bits\[4:2] are the stage 2 memory attributes (MemAttr\[2:0]). For more information, see Stage 2 memory type and Cacheability attributes when FEAT_S2FWB is enabled on page D8-5887.
    4:2 => stage_2_fwb_mem_attr,

    // descriptor formats
    1:0 => format: DescriptorFormat,

    1:0 => level_3_format: Level3DescriptorFormat,
});

