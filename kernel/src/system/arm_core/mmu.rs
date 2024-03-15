use core::arch::asm;
use mystd::bit_field;

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


bit_field!(
/// ## D19.2.124 SCTLR_EL1, System Control Register (EL1)
/// The SCTLR_EL1 characteristics are:
/// 
/// ### Purpose 
/// 
/// Provides top level control of the system, including its memory system, at EL1 and EL0.
/// 
/// ### Configurations
/// 
/// AArch64 System register SCTLR_EL1 bits [31:0] are architecturally mapped to AArch32 System register SCTLR[31:0]. 
/// 
/// ### Attributes
/// 
/// SCTLR_EL1 is a 64-bit register.
SctlrEl1(u64)

    /// ## TIDCP, bit \[63]
    /// 
    /// ### When FEAT_TIDCP1 is implemented:
    /// 
    /// Trap IMPLEMENTATION DEFINED functionality. When HCR_EL2.{E2H, TGE} != {1, 1}, traps EL0 accesses to the encodings reserved for IMPLEMENTATION DEFINED functionality to EL1.
    /// 
    /// * 0b0
    ///     - No instructions accessing the System register or System instruction spaces are trapped by this mechanism.
    /// * 0b1
    ///     - Instructions accessing the following System register or System instruction spaces are trapped to EL1 by this mechanism:
    ///         + In AArch64 state, EL0 access to the encodings in the following reserved encoding spaces are trapped and reported using EC syndrome 0x18:
    ///             - IMPLEMENTATION DEFINED System instructions, which are accessed using SYS and SYSL, with CRn == {11, 15}.
    ///             - IMPLEMENTATION DEFINED System registers, which are accessed using MRS and MSR with the S3_\<op1>_\<Cn>_\<Cm>_\<op2> register name.
    ///         + In AArch32 state, EL0 MCR and MRC access to the following encodings are trapped and reported using EC syndrome 0x03:
    ///             - All coproc==p15, CRn==c9, opc1 == {0-7}, CRm == {c0-c2, c5-c8}, opc2 == {0-7}.
    ///             - All coproc==p15, CRn==c10, opc1 =={0-7}, CRm == {c0, c1, c4, c8}, opc2 == {0-7}.
    ///             - All coproc==p15, CRn==c11, opc1=={0-7}, CRm == {c0-c8, c15}, opc2 == {0-7}.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    63 => tidcp,

    /// ## SPINTMASK, bit \[62]
    /// 
    /// ### When FEAT_NMI is implemented:
    /// 
    /// SP Interrupt Mask enable. When SCTLR_EL1.NMI is 1, controls whether PSTATE.SP acts as an interrupt mask, and controls the value of PSTATE.ALLINT on taking an exception to EL1.
    /// 
    /// * 0b0
    ///     - Does not cause PSTATE.SP to mask interrupts. PSTATE.ALLINT is set to 1 on taking an exception to EL1.
    /// * 0b1
    ///     - When PSTATE.SP is 1 and execution is at EL1, an IRQ or FIQ interrupt that is targeted to EL1 is masked regardless of any denotion of Superpriority. 
    ///     - PSTATE.ALLINT is set to 0 on taking an exception to EL1.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    62 => spintmask,

    /// ## NMI, bit \[61]
    /// 
    /// ### When FEAT_NMI is implemented:
    /// 
    /// Non-maskable Interrupt enable.
    /// 
    /// * 0b0 
    ///     - This control does not affect interrupt masking behavior.
    /// * 0b1 
    ///     - This control enables all of the following:
    ///         + The use of the PSTATE.ALLINT interrupt mask.
    ///         + IRQ and FIQ interrupts to have Superpriority as an additional attribute.
    ///         + PSTATE.SP to be used as an interrupt mask.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset:
    ///     - When EL2 is not implemented and EL3 is not implemented, this field resets to 0.
    ///     - Otherwise, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    61 => nmi,

    /// ## EnTP2, bit \[60]
    /// 
    /// ### When FEAT_SME is implemented:
    /// 
    /// Traps instructions executed at EL0 that access TPIDR2_EL0 to EL1, or to EL2 when EL2 is implemented and enabled for the current Security state and HCR_EL2.TGE is 1. The exception is reported using ESR_ELx.EC value 0x18.
    /// 
    /// * 0b0
    ///     - This control causes execution of these instructions at EL0 to be trapped.
    /// * 0b1
    ///     - This control does not cause execution of any instructions to be trapped.
    /// 
    /// If FEAT_VHE is implemented, EL2 is implemented and enabled in the current Security state, and HCR_EL2.{E2H, TGE} == {1, 1}, this field has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    ///
    /// ### Otherwise:
    /// 
    /// Reserved, RES0
    60 => en_tp2,

    // 59:58 => reserved res 0

    /// ## EPAN, bit \[57]
    /// 
    /// ### When FEAT_PAN3 is implemented:
    /// 
    /// Enhanced Privileged Access Never. When PSTATE.PAN is 1, determines whether an EL1 data access to a page with stage 1 EL0 instruction access permission generates a Permission fault as a result of the Privileged Access Never mechanism.
    /// 
    /// * 0b0
    ///     - No additional Permission faults are generated by this mechanism.
    /// * 0b1
    ///     - An EL1 data access to a page with stage 1 EL0 data access permission or stage 1 EL0 instruction access permission generates a Permission fault.
    ///     - Any speculative data accesses that would generate a Permission fault as a result of PSTATE.PAN = 1 if the accesses were not speculative, will not cause an allocation into a cache.
    /// 
    /// This bit is permitted to be cached in a TLB.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    57 => epan,

    /// ## EnALS, bit \[56]
    /// 
    /// ### When FEAT_LS64 is implemented:
    /// 
    /// When HCR_EL2.{E2H, TGE} != {1, 1}, traps execution of an LD64B or ST64B instruction at EL0 to EL1.
    /// 
    /// * 0b0
    ///     - Execution of an LD64B or ST64B instruction at EL0 is trapped to EL1.
    /// * 0b1
    ///     - This control does not cause any instructions to be trapped.
    /// 
    /// A trap of an LD64B or ST64B instruction is reported using an ESR_ELx.EC value of 0x0A, with an ISS code of 0x0000002.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    56 => en_als,

    /// ## EnAS0, bit \[55]
    /// 
    /// ### When FEAT_LS64_ACCDATA is implemented:
    /// 
    /// When HCR_EL2.{E2H, TGE} != {1, 1}, traps execution of an ST64BV0 instruction at EL0 to EL1.
    /// 
    /// * 0b0
    ///     - Execution of an ST64BV0 instruction at EL0 is trapped to EL1.
    /// * 0b1
    ///     - This control does not cause any instructions to be trapped.
    /// 
    /// A trap of an ST64BV0 instruction is reported using an ESR_ELx.EC value of 0x0A, with an ISS code of 0x0000001.
    /// 
    /// The reset behavior of this field is:
    /// 
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    55 => en_as0,

    /// ## EnASR, bit \[54]
    /// 
    /// ### When FEAT_LS64_V is implemented:
    /// 
    /// When HCR_EL2.{E2H, TGE} != {1, 1}, traps execution of an ST64BV instruction at EL0 to EL1.
    /// 
    /// * 0b0
    ///     - Execution of an ST64BV instruction at EL0 is trapped to EL1.
    /// * 0b1
    ///     - This control does not cause any instructions to be trapped.
    /// 
    /// A trap of an ST64BV instruction is reported using an ESR_ELx.EC value of 0x0A, with an ISS code of 0x0000000.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    54 => en_asr,

    /// ## TME, bit \[53]
    /// 
    /// ### When FEAT_TME is implemented:
    /// 
    /// Enables the Transactional Memory Extension at EL1.
    /// 
    /// * 0b0 
    ///     - Any attempt to execute a TSTART instruction at EL1 is trapped to EL1, unless HCR_EL2.TME or SCR_EL3.TME causes TSTART instructions to be UNDEFINED at EL1.
    /// * 0b1 
    ///     - This control does not cause any TSTART instruction to be trapped.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    53 => tme,

    /// ## TME0, bit \[52]
    /// 
    /// ### When FEAT_TME is implemented:
    /// 
    /// Enables the Transactional Memory Extension at EL0.
    /// 
    /// * 0b0
    ///     - Any attempt to execute a TSTART instruction at EL0 is trapped to EL1, unless HCR_EL2.TME or SCR_EL3.TME causes TSTART instructions to be UNDEFINED at EL0.
    /// * 0b1
    ///     - This control does not cause any TSTART instruction to be trapped.
    /// 
    /// If FEAT_VHE is implemented, EL2 is implemented and enabled in the current Security state, and
    /// HCR_EL2.{E2H, TGE} == {1, 1}, this field has no effect on execution at EL0. 
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    52 => tme0,

    /// ## TMT, bit \[51]
    /// 
    /// ### When FEAT_TME is implemented:
    /// 
    /// Forces a trivial implementation of the Transactional Memory Extension at EL1.
    /// 
    /// * 0b0
    ///     - This control does not cause any TSTART instruction to fail.
    /// * 0b1
    ///     - When the TSTART instruction is executed at EL1, the transaction fails with a TRIVIAL failure cause.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    51 => tmt,

    /// ## TMT0, bit \[50]
    /// 
    /// ### When FEAT_TME is implemented:
    /// 
    /// Forces a trivial implementation of the Transactional Memory Extension at EL0.
    /// 
    /// * 0b0
    ///     - This control does not cause any TSTART instruction to fail.
    /// * 0b1
    ///     - When the TSTART instruction is executed at EL0, the transaction fails with a TRIVIAL failure cause.
    /// 
    /// If FEAT_VHE is implemented, EL2 is implemented and enabled in the current Security state, and HCR_EL2.{E2H, TGE} == {1, 1}, this field has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    50 => tmt0,

    /// ## TWEDEL, bits \[49:46]
    /// 
    /// ### When FEAT_TWED is implemented:
    /// 
    /// TWE Delay. A 4-bit unsigned number that, when SCTLR_EL1.TWEDEn is 1, encodes the minimum delay in taking a trap of WFE* caused by SCTLR_EL1.nTWE as 2(TWEDEL + 8) cycles.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    49:46 => twedel,

    /// ## TWEDEn, bit \[45]
    /// 
    /// ### When FEAT_TWED is implemented:
    /// 
    /// TWE Delay Enable. Enables a configurable delayed trap of the WFE* instruction caused by SCTLR_EL1.nTWE.
    /// 
    /// * 0b0
    ///     - The delay for taking the trap is IMPLEMENTATION DEFINED.
    /// * 0b1
    ///     - The delay for taking the trap is at least the number of cycles defined in SCTLR_EL1.TWEDEL.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    45 => twed_en,

    /// ## DSSBS, bit \[44]
    /// 
    /// ### When FEAT_SSBS is implemented:
    /// 
    /// Default PSTATE.SSBS value on Exception Entry.
    /// 
    /// * 0b0
    ///     - PSTATE.SSBS is set to 0 on an exception to EL1.
    /// * 0b1
    ///     - PSTATE.SSBS is set to 1 on an exception to EL1.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset,this field resets to an IMPLEMENTATION DEFINED value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    44 => dssbs,
    
    /// ## ATA, bit \[43]
    /// 
    /// ### When FEAT_MTE2 is implemented:
    /// 
    /// Allocation Tag Access in EL1.
    /// 
    /// When SCR_EL3.ATA == 1 and HCR_EL2.ATA == 1, controls access to Allocation Tags and Tag Check operations in EL1.
    /// 
    /// * 0b0 
    ///     - Access to Allocation Tags is prevented at EL1.
    ///     - Memory accesses at EL1 are not subject to a Tag Check operation.
    /// * 0b1 
    ///     - This control does not prevent access to Allocation Tags at EL1.
    ///     - Tag Checked memory accesses at EL1 are subject to a Tag Check operation.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// Reserved, RES0.
    43 => ata,

    /// ## ATA0, bit \[42]
    /// 
    /// ### When FEAT_MTE2 is implemented:
    /// 
    /// Allocation Tag Access in EL0.
    /// 
    /// When SCR_EL3.ATA == 1, HCR_EL2.ATA == 1, and HCR_EL2.{E2H, TGE} != {1, 1}, controls access to Allocation Tags and Tag Check operations in EL0.
    /// 
    /// * 0b0 
    ///     - Access to Allocation Tags is prevented at EL0.
    ///     - Memory accesses at EL0 are not subject to a Tag Check operation.
    /// * 0b1 
    ///     - This control does not prevent access to Allocation Tags at EL0.
    ///     - Tag Checked memory accesses at EL0 are subject to a Tag Check operation.
    /// 
    ///
    /// > ### Note
    /// > Software may change this control bit on a context switch.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// Reserved, RES0.
    42 => ata0,

    /// ## TCF, bits \[41:40]
    /// 
    /// ### When FEAT_MTE2 is implemented:
    /// 
    /// Tag Check Fault in EL1. Controls the effect of Tag Check Faults due to Loads and Stores in EL1. If FEAT_MTE3 is not implemented, the value 0b11 is reserved.
    /// * 0b00
    ///     - Tag Check Faults have no effect on the PE.
    /// * 0b01
    ///     - Tag Check Faults cause a synchronous exception.
    /// * 0b10
    ///     - Tag Check Faults are asynchronously accumulated.
    /// * 0b11
    ///     - _When FEAT_MTE3 is implemented_: Tag Check Faults cause a synchronous exception on reads, and are asynchronously accumulated on writes.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    41:40 => tcf,

    /// ## TCF0, bits \[39:38]
    /// 
    /// ### When FEAT_MTE2 is implemented:
    /// 
    /// Tag Check Fault in EL0. When HCR_EL2.{E2H,TGE} != {1,1}, controls the effect of Tag Check Faults due to Loads and Stores in EL0.
    /// 
    /// If FEAT_MTE3 is not implemented, the value 0b11 is reserved. 
    /// 
    /// > ### Note
    /// > Software may change this control bit on a context switch.
    /// 
    /// * 0b00 
    ///     - Tag Check Faults have no effect on the PE.
    /// * 0b01 
    ///     - Tag Check Faults cause a synchronous exception.
    /// * 0b10 
    ///     - Tag Check Faults are asynchronously accumulated.
    /// * 0b11 
    ///     - _When FEAT_MTE3 is implemented_: Tag Check Faults cause a synchronous exception on reads, and are asynchronously accumulated on writes.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    39:40 => tcf0,

    /// ## ITFSB, bit \[37]
    /// ### When FEAT_MTE2 is implemented:
    /// 
    /// When synchronous exceptions are not being generated by Tag Check Faults, this field controls whether on exception entry into EL1, all Tag Check Faults due to instructions executed before exception entry, that are reported asynchronously, are synchronized into TFSRE0_EL1 and TFSR_EL1 registers.
    /// 
    /// * 0b0
    ///     - Tag Check Faults are not synchronized on entry to EL1.
    /// * 0b1
    ///     - Tag Check Faults are synchronized on entry to EL1.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    37 => itfsb,

    /// ## BT1, bit \[36]
    /// 
    /// ### When FEAT_BTI is implemented:
    /// 
    /// PAC Branch Type compatibility at EL1.
    /// 
    /// * 0b0
    ///     - When the PE is executing at EL1, PACIASP and PACIBSP are compatible with PSTATE.BTYPE == 0b11.
    /// * 0b1
    ///     - When the PE is executing at EL1, PACIASP and PACIBSP are not compatible with PSTATE.BTYPE == 0b11.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    36 => bt1,

    /// ## BT0, bit \[35]
    /// 
    /// ### When FEAT_BTI is implemented:
    /// 
    /// PAC Branch Type compatibility at EL0.
    /// 
    /// * 0b0
    ///     - When the PE is executing at EL0, PACIASP and PACIBSP are compatible with PSTATE.BTYPE == 0b11.
    /// * 0b1
    ///     - When the PE is executing at EL0, PACIASP and PACIBSP are not compatible with PSTATE.BTYPE == 0b11.
    /// 
    /// When the value of HCR_EL2.{E2H, TGE} is {1, 1}, the value of SCTLR_EL1.BT0 has no effect
    /// on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    35 => bt0,

    // 34 => reserved res0,
    
    /// ## MSCEn, bit \[33]
    /// 
    /// ### When FEAT_MOPS is implemented and (HCR_EL2.E2H == 0 or HCR_EL2.TGE == 0):
    /// 
    /// Memory Copy and Memory Set instructions Enable. Enables execution of the Memory Copy and Memory Set instructions at EL0.
    /// 
    /// * 0b0
    ///     - Execution of the Memory Copy and Memory Set instructions is UNDEFINED at EL0.
    /// * 0b1
    ///     - This control does not cause any instructions to be UNDEFINED.
    /// 
    /// When FEAT_MOPS is implemented and HCR_EL2.{E2H, TGE} is {1, 1}, the Effective value of this bit is 0b1.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    33 => msc_en,

    /// ## CMOW, bit \[32]
    /// 
    /// ### When FEAT_CMOW is implemented:
    /// 
    /// Controls cache maintenance instruction permission for the following instructions executed at EL0.
    /// 
    /// * IC IVAU, DC CIVAC, DC CIGDVAC and DC CIGVAC.
    /// 
    /// * 0b0
    ///     - These instructions executed at EL0 with stage 1 read permission, but without stage 1 write permission, do not generate a stage 1 permission fault.
    /// * 0b1
    ///     - If enabled as a result of SCTLR_EL1.UCI==1, these instructions executed at EL0 with stage 1 read permission, but without stage 1 write permission, generate a stage 1 permission fault.
    /// 
    /// When AArch64.HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on execution at EL0. For this control, stage 1 has write permission if all of the following apply:
    /// 
    /// * AP\[2] is 0 or DBM is 1 in the stage 1 descriptor.
    /// * Where APTable is in use, APTable\[1] is 0 for all levels of the translation table.
    /// 
    /// This bit is permitted to be cached in a TLB.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    32 => cmow,

    /// ## EnIA, bit \[31]
    /// 
    /// ### When FEAT_PAuth is implemented:
    /// 
    /// Controls enabling of pointer authentication (using the APIAKey_EL1 key) of instruction addresses in the EL1&0 translation regime.
    /// 
    /// * 0b0
    ///     - Pointer authentication (using the APIAKey_EL1 key) of instruction addresses is not enabled.
    /// * 0b1
    ///     - Pointer authentication (using the APIAKey_EL1 key) of instruction addresses is enabled.
    /// 
    /// > ### Note
    /// > This field controls the behavior of the AddPACIA and AuthIA pseudocode functions. Specifically, when the field is 1, AddPACIA returns a copy of a pointer to which a pointer authentication code has been added, and AuthIA returns an authenticated copy of a pointer. When the field is 0, both of these functions are NOP.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    31 => en_ia,

    /// ## EnIB, bit \[30]
    /// 
    /// ### When FEAT_PAuth is implemented:
    /// 
    /// Controls enabling of pointer authentication (using the APIBKey_EL1 key) of instruction addresses in the EL1&0 translation regime.
    /// 
    /// * 0b0
    ///     - Pointer authentication (using the APIBKey_EL1 key) of instruction addresses is not enabled.
    /// * 0b1
    ///     - Pointer authentication (using the APIBKey_EL1 key) of instruction addresses is enabled.
    /// 
    /// > ### Note
    /// > This field controls the behavior of the AddPACIB and AuthIB pseudocode functions. Specifically, when the field is 1, AddPACIB returns a copy of a pointer to which a pointer authentication code has been added, and AuthIB returns an authenticated copy of a pointer. When the field is 0, both of these functions are NOP.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    30 => en_ib,

    /// ## LSMAOE, bit \[29]
    /// 
    /// ### When FEAT_LSMAOC is implemented:
    /// 
    /// Load Multiple and Store Multiple Atomicity and Ordering Enable.
    /// 
    /// * 0b0
    ///     - For all memory accesses at EL0, A32 and T32 Load Multiple and Store Multiple can have an interrupt taken during the sequence memory accesses, and the memory accesses are not required to be ordered.
    /// * 0b1
    ///     - The ordering and interrupt behavior of A32 and T32 Load Multiple and Store Multiple at EL0 is as defined for Armv8.0.
    /// 
    /// This bit is permitted to be cached in a TLB.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1,1}, this bit has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES1.
    29 => lsmaoe,

    /// ## nTLSMD, bit \[28]
    /// 
    /// ### When FEAT_LSMAOC is implemented:
    /// 
    /// No Trap Load Multiple and Store Multiple to Device-nGRE/Device-nGnRE/Device-nGnRnE memory.
    /// 
    /// * 0b0
    ///     - All memory accesses by A32 and T32 Load Multiple and Store Multiple at EL0 that are marked at stage 1 as Device-nGRE/Device-nGnRE/Device-nGnRnE memory are trapped and generate a stage 1 Alignment fault.
    /// * 0b1
    ///     - All memory accesses by A32 and T32 Load Multiple and Store Multiple at EL0 that are marked at stage 1 as Device-nGRE/Device-nGnRE/Device-nGnRnE memory are not trapped.
    /// 
    /// This bit is permitted to be cached in a TLB.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1,1}, this bit has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES1.
    28 => n_tlsmd,

    /// ## EnDA, bit \[27]
    /// 
    /// ### When FEAT_PAuth is implemented:
    /// 
    /// Controls enabling of pointer authentication (using the APDAKey_EL1 key) of instruction addresses in the EL1&0 translation regime.
    /// 
    /// * 0b0
    ///     - Pointer authentication (using the APDAKey_EL1 key) of data addresses is not enabled. 
    /// * 0b1
    ///     - Pointer authentication (using the APDAKey_EL1 key) of data addresses is enabled.
    /// 
    /// > ### Note
    /// > This field controls the behavior of the AddPACDA and AuthDA pseudocode functions. Specifically, when the field is 1, AddPACDA returns a copy of a pointer to which a pointer authentication code has been added, and AuthDA returns an authenticated copy of a pointer. When the field is 0, both of these functions are NOP.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    27 => en_da,

    /// ## UCI, bit \[26]
    /// 
    /// Traps EL0 execution of cache maintenance instructions, to EL1, or to EL2 when it is implemented and enabled for the current Security state and HCR_EL2.TGE is 1, from AArch64 state only, reported using an ESR_ELx.EC value of 0x18.
    /// 
    /// This applies to DC CVAU, DC CIVAC, DC CVAC, DC CVAP, and IC IVAU.
    /// 
    /// If FEAT_DPB2 is implemented, this trap also applies to DC CVADP.
    /// 
    /// If FEAT_MTE is implemented, this trap also applies to DC CIGVAC, DC CIGDVAC, DC CGVAC,
    /// DC CGDVAC, DC CGVAP, and DC CGDVAP.
    /// 
    /// If FEAT_DPB2 and FEAT_MTE are implemented, this trap also applies to DC CGVADP and DC
    /// CGDVADP.
    /// 
    /// * 0b0
    ///     - Execution of the specified instructions at EL0 using AArch64 is trapped.
    /// * 0b1
    ///     - This control does not cause any instructions to be trapped.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on execution at EL0.
    /// 
    /// If the Point of Coherency is before any level of data cache, it is IMPLEMENTATION DEFINED whether the execution of any data or unified cache clean, or clean and invalidate instruction that operates by VA to the point of coherency can be trapped when the value of this control is 1.
    /// 
    /// If the Point of Unification is before any level of data cache, it is IMPLEMENTATION DEFINED whether the execution of any data or unified cache clean by VA to the Point of Unification instruction can be trapped when the value of this control is 1.
    /// 
    /// If the Point of Unification is before any level of instruction cache, it is IMPLEMENTATION DEFINED whether the execution of any instruction cache invalidate by VA to the Point of Unification instruction can be trapped when the value of this control is 1.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    26 => uci,

    /// ## EE, bit \[25]
    /// 
    /// Endianness of data accesses at EL1, and stage 1 translation table walks in the EL1&0 translation regime.
    /// 
    /// * 0b0
    ///     - Explicit data accesses at EL1, and stage 1 translation table walks in the EL1&0 translation regime are little-endian.
    /// * 0b1
    ///     - Explicit data accesses at EL1, and stage 1 translation table walks in the EL1&0 translation regime are big-endian.
    /// 
    /// If an implementation does not provide Big-endian support at Exception levels higher than EL0, this bit is RES0.
    /// 
    /// If an implementation does not provide Little-endian support at Exception levels higher than EL0, this bit is RES1.
    /// 
    /// The EE bit is permitted to be cached in a TLB.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on the PE.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset,this field resets to an IMPLEMENTATION DEFINED value.
    25 => ee,

    /// ## E0E, bit \[24]
    /// 
    /// Endianness of data accesses at EL0.
    /// 
    /// * 0b0
    ///     - Explicit data accesses at EL0 are little-endian.
    /// * 0b1
    ///     - Explicit data accesses at EL0 are big-endian.
    /// 
    /// If an implementation only supports Little-endian accesses at EL0, then this bit is RES0. This option is not permitted when SCTLR_EL1.EE is RES1.
    /// 
    /// If an implementation only supports Big-endian accesses at EL0, then this bit is RES1. This option is not permitted when SCTLR_EL1.EE is RES0.
    /// 
    /// This bit has no effect on the endianness of LDTR, LDTRH, LDTRSH, LDTRSW, STTR, and STTRH instructions executed at EL1.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    24 => e0e,

    /// ## SPAN, bit \[23]
    /// 
    /// ### When FEAT_PAN is implemented:
    /// 
    /// Set Privileged Access Never, on taking an exception to EL1.
    /// 
    /// * 0b0
    ///     - PSTATE.PAN is set to 1 on taking an exception to EL1.
    /// * 0b1
    ///     - The value of PSTATE.PAN is left unchanged on taking an exception to EL1.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES1.
    23 => span,

    /// ## EIS, bit \[22]
    /// 
    /// ### When FEAT_ExS is implemented:
    /// 
    /// Exception Entry is Context Synchronizing.
    /// 
    /// * 0b0
    ///     - The taking of an exception to EL1 is not a context synchronizing event.
    /// * 0b1
    ///     - The taking of an exception to EL1 is a context synchronizing event.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1,1}, this bit has no effect on execution at EL0.
    /// 
    /// If SCTLR_EL1.EIS is set to 0b0:
    /// 
    /// * Indirect writes to ESR_EL1, FAR_EL1, SPSR_EL1, ELR_EL1 are synchronized on exception entry to EL1, so that a direct read of the register after exception entry sees the indirectly written value caused by the exception entry.
    /// * Memory transactions, including instruction fetches, from an Exception level always use the translation resources associated with that translation regime.
    /// * Exception Catch debug events are synchronous debug events.
    /// * DCPS* and DRPS instructions are context synchronization events.
    /// 
    /// The following are not affected by the value of SCTLR_EL1.EIS:
    /// 
    /// * Changes to the PSTATE information on entry to EL1.
    /// * Behavior of accessing the banked copies of the stack pointer using the SP register name for loads, stores and data processing instructions.
    /// * Exit from Debug state. The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES1.
    22 => eis,

    /// ## IESB, bit \[21]
    /// 
    /// ### When FEAT_IESB is implemented:
    /// 
    /// Implicit Error Synchronization event enable. Possible values are:
    /// 
    /// * 0b0
    ///     - Disabled.
    /// * 0b1
    ///     - An implicit error synchronization event is added:
    ///         + At each exception taken to EL1.
    ///         + Before the operational pseudocode of each ERET instruction executed at EL1.
    /// When the PE is in Debug state, the effect of this field is CONSTRAINED UNPREDICTABLE, and its Effective value might be 0 or 1 regardless of the value of the field. If the Effective value of the field is 1, then an implicit error synchronization event is added after each DCPSx instruction taken to EL1 and before each DRPS instruction executed at EL1, in addition to the other cases where it is added.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    21 => iesb,

    /// ## TSCXT, bit \[20]
    /// 
    /// ### When FEAT_CSV2_2 is implemented or FEAT_CSV2_1p2 is implemented:
    /// 
    /// Trap EL0 Access to the SCXTNUM_EL0 register, when EL0 is using AArch64.
    /// 
    /// * 0b0
    ///     - EL0 access to SCXTNUM_EL0 is not disabled by this mechanism.
    /// * 0b1
    ///     - EL0 access to SCXTNUM_EL0 is disabled, causing an exception to EL1, or to EL2 when it is implemented and enabled for the current Security state and HCR_EL2.TGE is 1.
    ///     - The value of SCXTNUM_EL0 is treated as 0.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1,1}, this bit has
    /// no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES1.
    20 => tscxt,

    /// ## WXN, bit \[19]
    /// 
    /// Write permission implies XN (Execute-never). For the EL1&0 translation regime, this bit can force all memory regions that are writable to be treated as XN.
    /// 
    /// * 0b0
    ///     - This control has no effect on memory access permissions.
    /// * 0b1
    ///     - Any region that is writable in the EL1&0 translation regime is forced to XN for accesses from software executing at EL1 or EL0.
    /// 
    /// This bit applies only when SCTLR_EL1.M bit is set.
    /// 
    /// The WXN bit is permitted to be cached in a TLB.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on the PE.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    19 => wxn,

    /// ## nTWE, bit \[18]
    /// 
    /// Traps EL0 execution of WFE instructions to EL1, or to EL2 when it is implemented and enabled for the current Security state and HCR_EL2.TGE is 1, from both Execution states, reported using an ESR_ELx.EC value of 0x01.
    /// 
    /// When FEAT_WFxT is implemented, this trap also applies to the WFET instruction.
    /// 
    /// * 0b0
    ///     - Any attempt to execute a WFE instruction at EL0 is trapped, if the instruction would otherwise have caused the PE to enter a low-power state.
    /// * 0b1
    ///     - This control does not cause any instructions to be trapped.
    /// 
    /// In AArch32 state, the attempted execution of a conditional WFE instruction is only trapped if the instruction passes its condition code check.
    /// 
    /// > ### Note
    /// > Since a WFE or WFI can complete at any time, even without a Wakeup event, the traps on WFE of WFI are not guaranteed to be taken, even if the WFE or WFI is executed when there is no Wakeup event. The only guarantee is that if the instruction does not complete in finite time in the absence of a Wakeup event, the trap will be taken.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    18 => n_twe,

    /// ## nTWI, bit \[16]
    /// 
    /// Traps EL0 execution of WFI instructions to EL1, or to EL2 when it is implemented and enabled for the current Security state and HCR_EL2.TGE is 1, from both Execution states, reported using an ESR_ELx.EC value of 0x01.
    /// 
    /// When FEAT_WFxT is implemented, this trap also applies to the WFIT instruction.
    /// 
    /// * 0b0
    ///     - Any attempt to execute a WFI instruction at EL0 is trapped, if the instruction would otherwise have caused the PE to enter a low-power state.
    /// * 0b1
    ///     - This control does not cause any instructions to be trapped.
    /// 
    /// In AArch32 state, the attempted execution of a conditional WFI instruction is only trapped if the instruction passes its condition code check.
    /// 
    /// > ### Note
    /// > Since a WFE or WFI can complete at any time, even without a Wakeup event, the traps on WFE of WFI are not guaranteed to be taken, even if the WFE or WFI is executed when there is no Wakeup event. The only guarantee is that if the instruction does not complete in finite time in the absence of a Wakeup event, the trap will be taken.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    16 => n_twi,

    /// ## UCT, bit \[15]
    /// 
    /// Traps EL0 accesses to the CTR_EL0 to EL1, or to EL2 when it is implemented and enabled for the current Security state and HCR_EL2.TGE is 1, from AArch64 state only, reported using an ESR_ELx.EC value of 0x18.
    /// 
    /// * 0b0
    ///     - Accesses to the CTR_EL0 from EL0 using AArch64 are trapped.
    /// * 0b1
    ///     - This control does not cause any instructions to be trapped.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    15 => uct,

    /// ## DZE, bit \[14]
    /// 
    /// Traps EL0 execution of DC ZVA instructions to EL1, or to EL2 when it is implemented and enabled for the current Security state and HCR_EL2.TGE is 1, from AArch64 state only, reported using an ESR_ELx.EC value of 0x18.
    /// 
    /// If FEAT_MTE is implemented, this trap also applies to DC GVA and DC GZVA.
    /// 
    /// * 0b0 
    ///     - Any attempt to execute an instruction that this trap applies to at EL0 using AArch64 is trapped.
    ///     - Reading DCZID_EL0.DZP from EL0 returns 1, indicating that the instructions this trap applies to are not supported.
    /// * 0b1 
    ///     - This control does not cause any instructions to be trapped.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has
    /// no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    14 => dze,

    /// ## EnDB, bit \[13]
    /// 
    /// ### When FEAT_PAuth is implemented:
    /// 
    /// Controls enabling of pointer authentication (using the APDBKey_EL1 key) of instruction addresses in the EL1&0 translation regime.
    /// 
    /// * 0b0
    ///     - Pointer authentication (using the APDBKey_EL1 key) of data addresses is not enabled. 
    /// * 0b1
    ///     - Pointer authentication (using the APDBKey_EL1 key) of data addresses is enabled.
    /// 
    /// > ### Note
    /// > This field controls the behavior of the AddPACDB and AuthDB pseudocode functions. Specifically, when the field is 1, AddPACDB returns a copy of a pointer to which a pointer authentication code has been added, and AuthDB returns an authenticated copy of a pointer. When the field is 0, both of these functions are NOP.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    13 => en_db,

    /// ## I, bit \[12]
    /// 
    /// Stage 1 instruction access Cacheability control, for accesses at EL0 and EL1:
    /// 
    /// * 0b0 
    ///     - All instruction access to Stage 1 Normal memory from EL0 and EL1 are Stage 1 Non-cacheable.
    ///     - If the value of SCTLR_EL1.M is 0, instruction accesses from stage 1 of the EL1&0 translation regime are to Normal, Outer Shareable, Inner Non-cacheable, Outer Non-cacheable memory.
    /// * 0b1 
    ///     - This control has no effect on the Stage 1 Cacheability of instruction access to Stage 1 Normal memory from EL0 and EL1.
    ///     - If the value of SCTLR_EL1.M is 0, instruction accesses from stage 1 of the EL1&0 translation regime are to Normal, Outer Shareable, Inner Write-Through, Outer Write-Through memory.
    /// 
    /// When the value of the HCR_EL2.DC bit is 1, then instruction access to Normal memory from EL0 and EL1 are Cacheable regardless of the value of the SCTLR_EL1.I bit.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on the PE.
    /// 
    /// The reset behavior of this field is: 
    /// * On a Warm reset:
    ///     - When EL2 is not implemented and EL3 is not implemented, this field resets to 0.
    ///     - Otherwise, this field resets to an architecturally UNKNOWN value.
    12 => i,

    /// ## EOS, bit \[11]
    /// 
    /// ### When FEAT_ExS is implemented:
    /// 
    /// Exception Exit is Context Synchronizing.
    /// * 0b0
    ///     - An exception return from EL1 is not a context synchronizing event
    /// * 0b1
    ///     - An exception return from EL1 is a context synchronizing event
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1,1}, this bit has no effect on execution at EL0.
    /// 
    /// If SCTLR_EL1.EOS is set to 0b0:
    /// 
    /// * Memory transactions, including instruction fetches, from an Exception level always use the translation resources associated with that translation regime.
    /// * Exception Catch debug events are synchronous debug events.
    /// * DCPS* and DRPS instructions are context synchronization events.
    /// 
    /// The following are not affected by the value of SCTLR_EL1.EOS:
    /// 
    /// * The indirect write of the PSTATE and PC values from SPSR_EL1 and ELR_EL1 on exception return is synchronized.
    /// * Behavior of accessing the banked copies of the stack pointer using the SP register name for loads, stores and data processing instructions.
    /// * Exit from Debug state. The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES1.
    11 => eos,

    /// ## EnRCTX, bit \[10]
    /// 
    /// ### When FEAT_SPECRES is implemented:
    /// 
    /// Enable EL0 access to the following System instructions:
    /// * CFPRCTX, DVPRCTX and CPPRCTX instructions.
    /// * CFP RCTX, DVP RCTX and CPP RCTX instructions.
    /// 
    /// * 0b0
    ///     - EL0 access to these instructions is disabled, and these instructions are trapped to EL1, or to EL2 when it is implemented and enabled for the current Security state and HCR_EL2.TGE is 1.
    /// * 0b1
    ///     - EL0 access to these instructions is enabled.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1,1}, this bit has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    10 => en_rctx,

    /// ## UMA, bit \[9]
    /// 
    /// User Mask Access. Traps EL0 execution of MSR and MRS instructions that access the PSTATE.{D, A, I, F} masks to EL1, or to EL2 when it is implemented and enabled for the current Security state and HCR_EL2.TGE is 1, from AArch64 state only, reported using an ESR_ELx.EC value of 0x18.
    /// 
    /// * 0b0
    ///     - Any attempt at EL0 using AArch64 to execute an MRS, MSR(register), or MSR(immediate) instruction that accesses the DAIF is trapped.
    /// * 0b1
    ///     - This control does not cause any instructions to be trapped.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    9 => uma,

    /// ## SED, bit \[8]
    /// 
    /// ### When EL0 is capable of using AArch32:
    /// 
    /// SETEND instruction disable. Disables SETEND instructions at EL0 using AArch32.
    /// 
    /// * 0b0
    ///     - SETEND instruction execution is enabled at EL0 using AArch32.
    /// * 0b1
    ///     - SETEND instructions are UNDEFINED at EL0 using AArch32 and any attempt at EL0 to access a SETEND instruction generates an exception to EL1, or to EL2 when it is implemented and enabled for the current Security state and HCR_EL2.TGE is 1, reported using an ESR_ELx.EC value of 0x00.
    /// 
    /// If the implementation does not support mixed-endian operation at any Exception level, this bit is RES1.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES1.
    8 => sed,

    /// ## ITD, bit \[7]
    /// 
    /// ### When EL0 is capable of using AArch32:
    /// 
    /// IT Disable. Disables some uses of IT instructions at EL0 using AArch32.
    /// 
    /// * 0b0
    ///     - All IT instruction functionality is enabled at EL0 using AArch32.
    /// * 0b1
    ///     - Any attempt at EL0 using AArch32 to execute any of the following is UNDEFINED and generates an exception, reported using an ESR_ELx.EC value of 0x00, to EL1 or to EL2 when it is implemented and enabled for the current Security state and HCR_EL2.TGE is 1:
    ///         + All encodings of the IT instruction with hw1\[3:0]!=1000.
    ///         + All encodings of the subsequent instruction with the following values for hw1:
    ///             - `0b11xxxxxxxxxxxxxx`: All 32-bit instructions, and the 16-bit instructions B, UDF, SVC, LDM, and STM.
    ///             - `0b1011xxxxxxxxxxxx`: All instructions in Miscellaneous 16-bit instructions on page F3-8135.
    ///             - `0b10100xxxxxxxxxxx`: ADD Rd, PC, #imm
    ///             - `0b01001xxxxxxxxxxx`: LDR Rd, \[PC, #imm]
    ///             - `0b0100x1xxx1111xxx`: ADD Rdn, PC; CMP Rn, PC; MOV Rd, PC; BX PC; BLX PC.
    ///             - `0b010001xx1xxxx111`: ADD PC, Rm; CMP PC, Rm; MOV PC, Rm. This pattern also covers unpredictable cases with BLX Rn.
    /// 
    /// These instructions are always UNDEFINED, regardless of whether they would pass or fail the condition code check that applies to them as a result of being in an IT block.
    /// 
    /// It is IMPLEMENTATION DEFINED whether the IT instruction is treated as:
    /// 
    /// * A 16-bit instruction, that can only be followed by another 16-bit instruction.
    /// * The first half of a 32-bit instruction.
    /// 
    /// This means that, for the situations that are UNDEFINED, either the second 16-bit instruction or the 32-bit instruction is UNDEFINED.
    /// 
    /// An implementation might vary dynamically as to whether IT is treated as a 16-bit instruction or the first half of a 32-bit instruction.
    /// 
    /// If an instruction in an active IT block that would be disabled by this field sets this field to 1 then behavior is CONSTRAINED UNPREDICTABLE. For more information, see Changes to an ITD control by an instruction in an IT block on page E1-7978.
    /// 
    /// ITD is optional, but if it is implemented in the SCTLR_EL1 then it must also be implemented in the SCTLR_EL2, HSCTLR, and SCTLR.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value. When an implementation does not implement ITD, access to this field is RAZ/WI.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES1.
    7 => itd,

    /// ## nAA, bit \[6]
    /// 
    /// ### When FEAT_LSE2 is implemented:
    /// 
    /// Non-aligned access. This bit controls generation of Alignment faults at EL1 and EL0 under certain conditions.
    /// 
    /// The following instructions generate an Alignment fault if all bytes being accessed are not within a single 16-byte quantity, aligned to 16 bytes for access:
    /// 
    /// * LDAPR, LDAPRH, LDAPUR, LDAPURH, LDAPURSH, LDAPURSW, LDAR, LDARH, LDLAR, LDLARH.
    /// * STLLR, STLLRH, STLR, STLRH, STLUR, and STLURH.
    /// 
    /// * 0b0 
    ///     - Unaligned accesses by the specified instructions generate an Alignment fault. 
    /// * 0b1 
    ///     - This control does not generate Alignment faults.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    6 => n_aa,

    /// ## CP15BEN, bit \[5]
    /// 
    /// ### When EL0 is capable of using AArch32:
    /// 
    /// System instruction memory barrier enable. Enables accesses to the DMB, DSB, and ISB System instructions in the (coproc==0b1111) encoding space from EL0:
    /// 
    /// * 0b0
    ///     - EL0 using AArch32: EL0 execution of the CP15DMB, CP15DSB, and CP15ISB instructions is UNDEFINED and generates an exception to EL1, or to EL2 when it is implemented and enabled for the current Security state and HCR_EL2.TGE is 1. The exception is reported using an ESR_ELx.EC value of 0x00.
    /// * 0b1
    ///     - EL0 using AArch32: EL0 execution of the CP15DMB, CP15DSB, and CP15ISB instructions is enabled.
    /// 
    /// CP15BEN is optional, but if it is implemented in the SCTLR_EL1 then it must also be implemented in the SCTLR_EL2, HSCTLR, and SCTLR.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// When an implementation does not implement CP15BEN, access to this field is RAO/WI.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    5 => cp15b_en,

    /// ## SA0, bit \[4]
    /// 
    /// SP Alignment check enable for EL0. When set to 1, if a load or store instruction executed at EL0 uses the SP as the base address and the SP is not aligned to a 16-byte boundary, then an SP alignment fault exception is generated. For more information, see SP alignment checking on page D1-5387.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    4 => sa0,

    /// ## SA, bit \[3]
    /// 
    /// SP Alignment check enable. When set to 1, if a load or store instruction executed at EL1 uses the SP as the base address and the SP is not aligned to a 16-byte boundary, then an SP alignment fault exception is generated. For more information, see SP alignment checking on page D1-5387.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on the PE.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    3 => sa,
    
    /// ## C, bit \[2]
    /// 
    /// Stage 1 Cacheability control, for data accesses.
    /// 
    /// * 0b0
    ///     - All data access to Stage 1 Normal memory from EL0 and EL1, and all Normal memory accesses from unified cache to the EL1&0 Stage 1 translation tables, are treated as Stage 1 Non-cacheable.
    /// * 0b1
    ///     - This control has no effect on the Stage 1 Cacheability of:
    /// 
    /// * Data access to Normal memory from EL0 and EL1.
    /// * Normal memory accesses to the EL1&0 Stage 1 translation tables.
    /// 
    /// When the Effective value of the HCR_EL2.DC bit in the current Security state is 1, the PE ignores SCTLR_EL1.C. This means that EL0 and EL1 data accesses to Normal memory are Cacheable.
    /// 
    /// When FEAT_VHE is implemented, and the Effective value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on the PE.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset:
    ///     - When EL2 is not implemented and EL3 is not implemented, this field resets to 0.
    ///     - Otherwise, this field resets to an architecturally UNKNOWN value.
    2 => c,

    /// ## A, bit \[1]
    /// 
    /// Alignment check enable. This is the enable bit for Alignment fault checking at EL1 and EL0.
    /// 
    /// 0b0 
    ///     - Alignment fault checking disabled when executing at EL1 or EL0.
    ///     - Instructions that load or store one or more registers, other than load/store exclusive and load-acquire/store-release, do not check that the address being accessed is aligned to the size of the data element(s) being accessed.
    /// 0b1 
    ///     - Alignment fault checking enabled when executing at EL1 or EL0.
    ///     - All instructions that load or store one or more registers have an alignment check that the address being accessed is aligned to the size of the data element(s) being accessed. If this check fails it causes an Alignment fault, which is taken as a Data Abort exception.
    /// 
    /// Load/store exclusive and load-acquire/store-release instructions have an alignment check regardless of the value of the A bit.
    /// 
    /// If FEAT_MOPS is implemented, SETG* instructions have an alignment check regardless of the value of the A bit.
    /// 
    /// When FEAT_VHE is implemented, and the value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on execution at EL0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    1 => a,

    /// ## M, bit \[0]
    /// 
    /// MMU enable for EL1&0 stage 1 address translation.
    /// 
    /// * 0b0 
    ///     - EL1&0 stage 1 address translation disabled.
    ///     - See the SCTLR_EL1.I field for the behavior of instruction accesses to Normal memory.
    /// * 0b1 
    ///     - EL1&0 stage 1 address translation enabled.
    /// 
    /// If the Effective value of HCR_EL2.{DC, TGE} in the current Security state is not {0, 0} then the PE behaves as if the value of the SCTLR_EL1.M field is 0 for all purposes other than returning the value of a direct read of the field.
    /// 
    /// When FEAT_VHE is implemented, and the Effective value of HCR_EL2.{E2H, TGE} is {1, 1}, this bit has no effect on the PE.
    /// 
    /// The reset behavior of this field is: 
    /// * On a Warm reset:
    ///     - When EL2 is not implemented and EL3 is not implemented, this field resets to 0.
    ///     - Otherwise, this field resets to an architecturally UNKNOWN value.
    0 => m
);

bit_field!(
/// ## D19.2.139 TCR_EL1, Translation Control Register (EL1)
/// The TCR_EL1 characteristics are:
/// 
/// ### Purpose 
/// * The control register for stage 1 of the EL1&0 translation regime.
/// 
/// ### Configurations
/// * AArch64 System register TCR_EL1 bits 31:0 are architecturally mapped to AArch32 System
/// register TTBCR 31:0.
/// * AArch64 System register TCR_EL1 bits 63:32 are architecturally mapped to AArch32 System
/// register TTBCR2 31:0.
///  
/// ### Attributes
/// * TCR_EL1 is a 64-bit register.
/// 
/// Any of the bits in TCR_EL1, other than the A1 bit and the EPDx bits when they have the value 1, are permitted to be cached in a TLB.
/// 
/// ### Accessing TCR_EL1
/// 
/// When HCR_EL2.E2H is 1, without explicit synchronization, access from EL3 using the mnemonic TCR_EL1 or TCR_EL12 are not guaranteed to be ordered with respect to accesses using the other mnemonic.
TcrEl1(u64)
    // 63:60 => reserved 0

    /// ## DS, Bit 59
    /// 
    /// ### When FEAT_LPA2 is implemented:
    /// 
    /// This field affects whether a 52-bit output address can be described by the translation tables of the 4KB or 16KB translation granules.
    /// 
    /// * `0b0` 
    ///     - Bits 49:48 of translation descriptors are RES0.
    ///     - Bits 9:8 in Block and Page descriptors encode shareability information in the SH 1:0 field. Bits 9:8 in Table descriptors are ignored by hardware.
    ///     - The minimum value of the TCR_EL1.{T0SZ, T1SZ} fields is 16. Any memory access using a smaller value generates a stage 1 level 0 translation table fault.
    ///     - Output address 51:48 is 0b0000.
    /// * `0b1`
    ///     - Bits 49:48 of translation descriptors hold output address 49:48.
    ///     - Bits 9:8 of Translation table descriptors hold output address 51:50.
    ///     - The shareability information of Block and Page descriptors for cacheable locations is determined by:
    ///         - TCR_EL1.SH0 if the VA is translated using tables pointed to by TTBR0_EL1.
    ///         - TCR_EL1.SH1 if the VA is translated using tables pointed to by TTBR1_EL1.
    ///     - The minimum value of the TCR_EL1.{T0SZ, T1SZ} fields is 12. Any memory access using a smaller value generates a stage 1 level 0 translation table fault.
    ///     - All calculations of the stage 1 base address are modified for tables of fewer than 8 entries so that the table is aligned to 64 bytes.
    ///     - Bits 5:2 of TTBR0_EL1 or TTBR1_EL1 are used to hold bits 51:48 of the output address in all cases.
    ///         
    ///         > ### Note
    ///         > As FEAT_LVA must be implemented if TCR_EL1.DS == 1, the minimum value of the TCR_EL1.{T0SZ, T1SZ} fields is 12, as determined by that extension.
    ///     
    ///     - For the TLBI Range instructions affecting VA, the format of the argument is changed so that bits 36:0 hold BaseADDR 52:16. For the 4KB translation granule, bits 15:12 of BaseADDR are treated as 0b0000. For the 16KB translation granule, bits 15:14 of BaseADDR are treated as 0b00.
    ///         
    ///         > ### Note
    ///         > This forces alignment of the ranges used by the TLBI range instructions.
    /// 
    /// This field is RES0 for a 64KB translation granule. 
    /// 
    /// The reset behavior of this field is:
    ///     - On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// Reserved, RES0, and the Effective value of this bit is 0b0.
    59 => ds,

    /// ## TCMA1, bit 58
    /// ### When FEAT_MTE2 is implemented:
    /// 
    /// Controls the generation of Unchecked accesses at EL1, and at EL0 if HCR_EL2.{E2H,TGE}!={1,1}, when address 59:55 = 0b11111.
    /// 
    /// * 0b0 
    ///     - This control has no effect on the generation of Unchecked accesses at EL1 or EL0. 
    /// * 0b1 
    ///     - All accesses at EL1 and EL0 are Unchecked.
    ///     
    /// > ### Note
    /// > Software may change this control bit on a context switch.
    /// 
    /// The reset behavior of this field is:
    ///     - On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    58 => tcma1,

    /// ## TCMA0, bit 57
    /// ### When FEAT_MTE2 is implemented:
    /// 
    /// Controls the generation of Unchecked accesses at EL1, and at EL0 if HCR_EL2.{E2H,TGE}!={1,1}, when address 59:55 = 0b00000.
    /// 
    /// * 0b0 
    ///     - This control has no effect on the generation of Unchecked accesses at EL1 or EL0. 
    /// * 0b1 
    ///     - All accesses at EL1 and EL0 are Unchecked.
    ///     
    /// > ### Note
    /// > Software may change this control bit on a context switch.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    57 => tcma0,

    /// ## E0PD1, bit 56
    /// 
    /// ### When FEAT_E0PD is implemented:
    /// 
    /// Faulting control for Unprivileged access to any address translated by TTBR1_EL1.
    /// 
    /// * 0b0 
    ///     - Unprivileged access to any address translated by TTBR1_EL1 will not generate a fault
    ///     by this mechanism.
    /// * 0b1 
    ///     - Unprivileged access to any address translated by TTBR1_EL1 will generate a level 0 Translation fault.
    /// 
    /// Level 0 Translation faults generated as a result of this field are not counted as TLB misses for performance monitoring. The fault should take the same time to generate, whether the address is present in the TLB or not, to mitigate attacks that use fault timing.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// Reserved, RES0.
    56 => e0pd1,

    /// ## E0PD0, bit 55
    /// 
    /// ### When FEAT_E0PD is implemented:
    /// 
    /// Faulting control for Unprivileged access to any address translated by TTBR0_EL1.
    /// 
    /// * 0b0 
    ///     - Unprivileged access to any address translated by TTBR0_EL1 will not generate a fault
    ///     by this mechanism.
    /// * 0b1 
    ///     - Unprivileged access to any address translated by TTBR0_EL1 will generate a level 0 Translation fault.
    /// 
    /// Level 0 Translation faults generated as a result of this field are not counted as TLB misses for performance monitoring. The fault should take the same time to generate, whether the address is present in the TLB or not, to mitigate attacks that use fault timing.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// Reserved, RES0.
    55 => e0pd0,

    /// ## NFD1, bit 54
    /// 
    /// ### When FEAT_SVE is implemented or FEAT_TME is implemented:
    /// 
    /// Non-fault translation timing disable for stage 1 translations using TTBR1_EL1.
    /// 
    /// This bit controls how a TLB miss is reported in response to a non-fault unprivileged access for a
    /// virtual address that is translated using TTBR1_EL1.
    /// 
    /// If SVE is implemented, the affected access types include:
    /// 
    /// * All accesses due to an SVE non-fault contiguous load instruction.
    /// * Accesses due to an SVE first-fault gather load instruction that are not for the First active element. Accesses due to an SVE first-fault contiguous load instruction are not affected.
    /// * Accesses due to prefetch instructions might be affected, but the effect is not architecturally visible.
    /// 
    /// If FEAT_TME is implemented, the affected access types include all accesses generated by a load or store instruction in Transactional state.
    /// 
    /// * 0b0 
    ///     - Does not affect the handling of a TLB miss on accesses translated using TTBR1_EL1.
    /// * 0b1 
    ///     - A TLB miss on a virtual address that is translated using TTBR1_EL1 due to the specified access types causes the access to fail without taking an exception. The failure should take the same amount of time to be handled as a Permission fault on a TLB entry that is present in the TLB, to mitigate attacks that use fault timing.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// Reserved, RES0.
    54 => nfd1,

    /// ## NFD0, bit 53
    /// 
    /// ### When FEAT_SVE is implemented or FEAT_TME is implemented:
    /// 
    /// Non-fault translation timing disable for stage 1 translations using TTBR0_EL1.
    /// 
    /// This bit controls how a TLB miss is reported in response to a non-fault unprivileged access for a
    /// virtual address that is translated using TTBR0_EL1.
    /// 
    /// If SVE is implemented, the affected access types include:
    /// 
    /// * All accesses due to an SVE non-fault contiguous load instruction.
    /// * Accesses due to an SVE first-fault gather load instruction that are not for the First active element. Accesses due to an SVE first-fault contiguous load instruction are not affected.
    /// * Accesses due to prefetch instructions might be affected, but the effect is not architecturally visible.
    /// 
    /// If FEAT_TME is implemented, the affected access types include all accesses generated by a load or store instruction in Transactional state.
    /// 
    /// * 0b0 
    ///     - Does not affect the handling of a TLB miss on accesses translated using TTBR0_EL1.
    /// * 0b1 
    ///     - A TLB miss on a virtual address that is translated using TTBR0_EL1 due to the specified access types causes the access to fail without taking an exception. The failure should take the same amount of time to be handled as a Permission fault on a TLB entry that is present in the TLB, to mitigate attacks that use fault timing.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// Reserved, RES0.
    53 => nfd0,

    /// ## TBID1, bit 52
    /// 
    /// ### When FEAT_PAuth is implemented:
    /// 
    /// Controls the use of the top byte of instruction addresses for address matching.
    /// 
    /// For the purpose of this field, all cache maintenance and address translation instructions that perform address translation are treated as data accesses.
    /// 
    /// For more information, see Address tagging on page D8-5894.
    /// 
    /// * 0b0 
    ///     - TCR_EL1.TBI1 applies to Instruction and Data accesses.
    /// * 0b1 
    ///     - TCR_EL1.TBI1 applies to Data accesses only.
    /// 
    /// This affects addresses where the address would be translated by tables pointed to by TTBR1_EL1. The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    52 => tbid1,

    /// ## TBID0, bit 51
    /// 
    /// ### When FEAT_PAuth is implemented:
    /// 
    /// Controls the use of the top byte of instruction addresses for address matching.
    /// 
    /// For the purpose of this field, all cache maintenance and address translation instructions that perform address translation are treated as data accesses.
    /// 
    /// For more information, see Address tagging on page D8-5894.
    /// 
    /// * 0b0 
    ///     - TCR_EL1.TBI0 applies to Instruction and Data accesses.
    /// * 0b1 
    ///     - TCR_EL1.TBI0 applies to Data accesses only.
    /// 
    /// This affects addresses where the address would be translated by tables pointed to by TTBR0_EL1. 
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    51 => tbid0,

    /// ## HWU162, bit 50
    /// 
    /// ### When FEAT_HPDS2 is implemented:
    /// 
    /// Hardware Use. Indicates IMPLEMENTATION DEFINED hardware use of bit 62 of the stage 1 translation table Block or Page entry for translations using TTBR1_EL1.
    /// 
    /// * 0b0 
    ///     - For translations using TTBR1_EL1, bit 62 of each stage 1 translation table Block or Page entry cannot be used by hardware for an IMPLEMENTATION DEFINED purpose.
    /// * 0b1 
    ///     - For translations using TTBR1_EL1, bit 62 of each stage 1 translation table Block or Page entry can be used by hardware for an IMPLEMENTATION DEFINED purpose if the value of TCR_EL1.HPD1 is 1.
    /// 
    /// The Effective value of this field is 0 if the value of TCR_EL1.HPD1 is 0.
    /// 
    /// The reset behavior of this field is:
    /// *  On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    50 => hwu162,

    /// ## HWU161, bit 49
    /// 
    /// ### When FEAT_HPDS2 is implemented:
    /// 
    /// Hardware Use. Indicates IMPLEMENTATION DEFINED hardware use of bit 61 of the stage 1 translation table Block or Page entry for translations using TTBR1_EL1.
    /// 
    /// * 0b0 
    ///     - For translations using TTBR1_EL1, bit 61 of each stage 1 translation table Block or Page entry cannot be used by hardware for an IMPLEMENTATION DEFINED purpose.
    /// * 0b1 
    ///     - For translations using TTBR1_EL1, bit 61 of each stage 1 translation table Block or Page entry can be used by hardware for an IMPLEMENTATION DEFINED purpose if the value of TCR_EL1.HPD1 is 1.
    /// 
    /// The Effective value of this field is 0 if the value of TCR_EL1.HPD1 is 0.
    /// 
    /// The reset behavior of this field is:
    /// *  On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    49 => hwu161,

    /// ## HWU160, bit 48
    /// 
    /// ### When FEAT_HPDS2 is implemented:
    /// 
    /// Hardware Use. Indicates IMPLEMENTATION DEFINED hardware use of bit 60 of the stage 1 translation table Block or Page entry for translations using TTBR1_EL1.
    /// 
    /// * 0b0 
    ///     - For translations using TTBR1_EL1, bit 60 of each stage 1 translation table Block or Page entry cannot be used by hardware for an IMPLEMENTATION DEFINED purpose.
    /// * 0b1 
    ///     - For translations using TTBR1_EL1, bit 60 of each stage 1 translation table Block or Page entry can be used by hardware for an IMPLEMENTATION DEFINED purpose if the value of TCR_EL1.HPD1 is 1.
    /// 
    /// The Effective value of this field is 0 if the value of TCR_EL1.HPD1 is 0.
    /// 
    /// The reset behavior of this field is:
    /// *  On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    48 => hwu160,

    /// ## HWU159, bit 47
    /// 
    /// ### When FEAT_HPDS2 is implemented:
    /// 
    /// Hardware Use. Indicates IMPLEMENTATION DEFINED hardware use of bit 59 of the stage 1 translation table Block or Page entry for translations using TTBR1_EL1.
    /// 
    /// * 0b0 
    ///     - For translations using TTBR1_EL1, bit 59 of each stage 1 translation table Block or Page entry cannot be used by hardware for an IMPLEMENTATION DEFINED purpose7
    /// * 0b1 
    ///     - For translations using TTBR1_EL1, bit 59 of each stage 1 translation table Block or Page entry can be used by hardware for an IMPLEMENTATION DEFINED purpose if the value of TCR_EL1.HPD1 is 17
    /// 
    /// The Effective value of this field is 0 if the value of TCR_EL1.HPD1 is 0.
    /// 
    /// The reset behavior of this field is:
    /// *  On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    47 => hwu159,

    /// ## HWU062, bit 46
    /// 
    /// ### When FEAT_HPDS2 is implemented:
    /// 
    /// Hardware Use. Indicates IMPLEMENTATION DEFINED hardware use of bit 62 of the stage 1 translation table Block or Page entry for translations using TTBR0_EL1.
    /// 
    /// * 0b0 
    ///     - For translations using TTBR0_EL1, bit 62 of each stage 1 translation table Block or Page entry cannot be used by hardware for an IMPLEMENTATION DEFINED purpose7
    /// * 0b1 
    ///     - For translations using TTBR0_EL1, bit 62 of each stage 1 translation table Block or Page entry can be used by hardware for an IMPLEMENTATION DEFINED purpose if the value of TCR_EL1.HPD0 is 17
    /// 
    /// The Effective value of this field is 0 if the value of TCR_EL1.HPD0 is 0.
    /// 
    /// The reset behavior of this field is:
    /// *  On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    46 => hwu062,

    /// ## HWU061, bit 45
    /// 
    /// ### When FEAT_HPDS2 is implemented:
    /// 
    /// Hardware Use. Indicates IMPLEMENTATION DEFINED hardware use of bit 61 of the stage 1 translation table Block or Page entry for translations using TTBR0_EL1.
    /// 
    /// * 0b0 
    ///     - For translations using TTBR0_EL1, bit 61 of each stage 1 translation table Block or Page entry cannot be used by hardware for an IMPLEMENTATION DEFINED purpose7
    /// * 0b1 
    ///     - For translations using TTBR0_EL1, bit 61 of each stage 1 translation table Block or Page entry can be used by hardware for an IMPLEMENTATION DEFINED purpose if the value of TCR_EL1.HPD0 is 17
    /// 
    /// The Effective value of this field is 0 if the value of TCR_EL1.HPD0 is 0.
    /// 
    /// The reset behavior of this field is:
    /// *  On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    45 => hwu061,

    /// ## HWU060, bit 44
    /// 
    /// ### When FEAT_HPDS2 is implemented:
    /// 
    /// Hardware Use. Indicates IMPLEMENTATION DEFINED hardware use of bit 60 of the stage 1 translation table Block or Page entry for translations using TTBR0_EL1.
    /// 
    /// * 0b0 
    ///     - For translations using TTBR0_EL1, bit 60 of each stage 1 translation table Block or Page entry cannot be used by hardware for an IMPLEMENTATION DEFINED purpose7
    /// * 0b1 
    ///     - For translations using TTBR0_EL1, bit 60 of each stage 1 translation table Block or Page entry can be used by hardware for an IMPLEMENTATION DEFINED purpose if the value of TCR_EL1.HPD0 is 17
    /// 
    /// The Effective value of this field is 0 if the value of TCR_EL1.HPD0 is 0.
    /// 
    /// The reset behavior of this field is:
    /// *  On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    44 => hwu060,

    /// ## HWU059, bit 43
    /// 
    /// ### When FEAT_HPDS2 is implemented:
    /// 
    /// Hardware Use. Indicates IMPLEMENTATION DEFINED hardware use of bit 59 of the stage 1 translation table Block or Page entry for translations using TTBR0_EL1.
    /// 
    /// * 0b0 
    ///     - For translations using TTBR0_EL1, bit 59 of each stage 1 translation table Block or Page entry cannot be used by hardware for an IMPLEMENTATION DEFINED purpose7
    /// * 0b1 
    ///     - For translations using TTBR0_EL1, bit 59 of each stage 1 translation table Block or Page entry can be used by hardware for an IMPLEMENTATION DEFINED purpose if the value of TCR_EL1.HPD0 is 17
    /// 
    /// The Effective value of this field is 0 if the value of TCR_EL1.HPD0 is 0.
    /// 
    /// The reset behavior of this field is:
    /// *  On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    43 => hwu059,

    /// ## HPD1, bit 42
    /// 
    /// ### When FEAT_HPDS is implemented:
    /// 
    /// Hierarchical Permission Disables. This affects the hierarchical control bits, APTable, PXNTable, and UXNTable, except NSTable, in the translation tables pointed to by TTBR1_EL1.
    /// 
    /// * 0b0 
    ///     - Hierarchical permissions are enabled.
    /// * 0b1 
    ///     - Hierarchical permissions are disabled.
    /// 
    /// When disabled, the permissions are treated as if the bits are zero.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    42 => hpd1,

    /// ## HPD0, bit 41
    /// 
    /// ### When FEAT_HPDS is implemented:
    /// 
    /// Hierarchical Permission Disables. This affects the hierarchical control bits, APTable, PXNTable, and UXNTable, except NSTable, in the translation tables pointed to by TTBR0_EL1.
    /// 
    /// * 0b0 
    ///     - Hierarchical permissions are enabled.
    /// * 0b1 
    ///     - Hierarchical permissions are disabled.
    /// 
    /// When disabled, the permissions are treated as if the bits are zero.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    41 => hpd0,

    /// ## HD, bit 40
    /// 
    /// ### When FEAT_HAFDBS is implemented:
    /// 
    /// Hardware management of dirty state in stage 1 translations from EL0 and EL1.
    /// 
    /// * 0b0 
    ///     - Stage 1 hardware management of dirty state disabled.
    /// * 0b1 
    ///     - Stage 1 hardware management of dirty state enabled, only if the HA bit is also set to 1.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    40 => hd,

    /// ## HA, bit 39
    /// 
    /// ### When FEAT_HAFDBS is implemented:
    /// 
    /// Hardware Access flag update in stage 1 translations from EL0 and EL1.
    /// 
    /// * 0b0 
    ///     - Stage 1 Access flag update disabled.
    /// * 0b1 
    ///     - Stage 1 Access flag update enabled.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ### Otherwise:
    /// 
    /// Reserved, RES0.
    39 => ha,

    /// ## TBI1, bit 38
    /// 
    /// ### AArch64 System Register Descriptions D19.2 General system control registers
    /// 
    /// Top Byte ignored. Indicates whether the top byte of an address is used for address match for the TTBR1_EL1 region, or ignored and used for tagged addresses.
    /// 
    /// * 0b0 
    ///     - Top Byte used in the address calculation.
    /// * 0b1 
    ///     - Top Byte ignored in the address calculation.
    /// 
    /// This affects addresses generated in EL0 and EL1 using AArch64 where the address would be translated by tables pointed to by TTBR1_EL1. It has an effect whether the EL1&0 translation regime is enabled or not.
    /// 
    /// If FEAT_PAuth is implemented and TCR_EL1.TBID1 is 1, then this field only applies to Data accesses.
    /// 
    /// Otherwise, if the value of TBI1 is 1 and bit 55 of the target address to be stored to the PC is 1, then bits 63:56 of that target address are also set to 1 before the address is stored in the PC, in the following cases:
    /// 
    /// * A branch or procedure return within EL0 or EL1.
    /// * An exception taken to EL1.
    /// * An exception return to EL0 or EL1. 
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    38 => tbi1,

    /// ## TBI0, bit 37
    /// 
    /// ### AArch64 System Register Descriptions D19.2 General system control registers
    /// 
    /// Top Byte ignored. Indicates whether the top byte of an address is used for address match for the TTBR0_EL1 region, or ignored and used for tagged addresses.
    /// 
    /// * 0b0 
    ///     - Top Byte used in the address calculation.
    /// * 0b1 
    ///     - Top Byte ignored in the address calculation.
    /// 
    /// This affects addresses generated in EL0 and EL1 using AArch64 where the address would be translated by tables pointed to by TTBR0_EL1. It has an effect whether the EL1&0 translation regime is enabled or not.
    /// 
    /// If FEAT_PAuth is implemented and TCR_EL1.TBID0 is 1, then this field only applies to Data accesses.
    /// 
    /// Otherwise, if the value of TBI0 is 1 and bit 55 of the target address to be stored to the PC is 1, then bits 63:56 of that target address are also set to 1 before the address is stored in the PC, in the following cases:
    /// 
    /// * A branch or procedure return within EL0 or EL1.
    /// * An exception taken to EL1.
    /// * An exception return to EL0 or EL1. 
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    37 => tbi0,

    /// ## AS, bit 36
    /// 
    /// Note: renamed due to identifier `as` being a reserved keyword.
    /// 
    /// ASID Size.
    /// 
    /// * 0b0 
    ///     - 8 bit - the upper 8 bits of TTBR0_EL1 and TTBR1_EL1 are ignored by hardware for every purpose except reading back the register, and are treated as if they are all zeros for when used for allocation and matching entries in the TLB.
    /// * 0b1 
    ///     - 16 bit - the upper 16 bits of TTBR0_EL1 and TTBR1_EL1 are used for allocation and matching in the TLB.
    /// 
    /// If the implementation has only 8 bits of ASID, this field is RES0.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    36 => as_,
    // 35 => reserved RES0,

    /// ## IPS, bits 34:32
    /// 
    /// Intermediate Physical Address Size.
    /// 
    /// * 0b000 
    ///     - 32 bits, 4GB.
    /// * 0b001 
    ///     - 36 bits, 64GB.
    /// * 0b010 
    ///     - 40 bits, 1TB.
    /// * 0b011 
    ///     - 42 bits, 4TB.
    /// * 0b100 
    ///     - 44 bits, 16TB.
    /// * 0b101 
    ///     - 48 bits, 256TB.
    /// * 0b110 
    ///     - 52 bits, 4PB.
    /// 
    /// All other values are reserved.
    /// 
    /// The reserved values behave in the same way as the 0b101 or 0b110 encoding, but software must not rely on this property as the behavior of the reserved values might change in a future revision of the architecture.
    /// 
    /// If the translation granule is not 64KB and FEAT_LPA2 is not implemented, the value 0b110 is treated as reserved.
    /// 
    /// It is IMPLEMENTATION DEFINED whether an implementation that does not implement FEAT_LPA supports setting the value of 0b110 for the 64KB translation granule size or whether setting this value behaves as the 0b101 encoding.
    /// 
    /// If the value of ID_AA64MMFR0_EL1.PARange is 0b0110, and the value of this field is not 0b110 or a value treated as 0b110, then bits[51:48] of every translation table base address for the stage of translation controlled by TCR_EL1 are 0b0000.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    34:32 => ips,
    
    /// ## TG1, bits 31:30
    /// 
    /// Granule size for the TTBR1_EL1. 
    /// 
    /// * 0b01 
    ///     - 16KB.
    /// * 0b10
    ///     - 4KB.
    /// * 0b11
    ///     - 64KB.
    /// 
    /// Other values are reserved.
    /// 
    /// If the value is programmed to either a reserved value or a size that has not been implemented, then the hardware will treat the field as if it has been programmed to an IMPLEMENTATION DEFINED choice of the sizes that has been implemented for all purposes other than the value read back from this register.
    /// 
    /// It is IMPLEMENTATION DEFINED whether the value read back is the value programmed or the value that corresponds to the size chosen.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    31:30 => tg1,

    /// ## SH1, bits 29:28
    /// Shareability attribute for memory associated with translation table walks using TTBR1_EL1. 
    /// 
    /// * 0b00
    ///     - Non-shareable.
    /// * 0b10
    ///     - Outer Shareable.
    /// * 0b11
    ///     - Inner Shareable.
    /// 
    /// Other values are reserved. The effect of programming this field to a Reserved value is that behavior is CONSTRAINED UNPREDICTABLE.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    29:28 => sh1,

    /// ## ORGN1, bits 27:26
    /// 
    /// Outer cacheability attribute for memory associated with translation table walks using TTBR1_EL1.
    /// 
    /// * 0b00 
    ///     - Normal memory, Outer Non-cacheable.
    /// * 0b01 
    ///     - Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable.
    /// * 0b10 
    ///     - Normal memory, Outer Write-Through Read-Allocate No Write-Allocate Cacheable.
    /// * 0b11 
    ///     - Normal memory, Outer Write-Back Read-Allocate No Write-Allocate Cacheable.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    27:26 => orgn1,

    /// ## IRGN1, bits 25:24
    /// 
    /// Inner cacheability attribute for memory associated with translation table walks using TTBR1_EL1.
    /// 
    /// * 0b00 
    ///     - Normal memory, Inner Non-cacheable.
    /// * 0b01 
    ///     - Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable.
    /// * 0b10 
    ///     - Normal memory, Inner Write-Through Read-Allocate No Write-Allocate Cacheable.
    /// * 0b11 
    ///     - Normal memory, Inner Write-Back Read-Allocate No Write-Allocate Cacheable.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    25:24 => irgn1,

    /// ## EPD1, bit 23
    /// 
    /// Translation table walk disable for translations using TTBR1_EL1. This bit controls whether a translation table walk is performed on a TLB miss, for an address that is translated using TTBR1_EL1. The encoding of this bit is:
    /// 
    /// * 0b0
    ///     - Perform translation table walks using TTBR1_EL1.
    /// * 0b1
    ///     - A TLB miss on an address that is translated using TTBR1_EL1 generates a Translationfault. No translation table walk is performed.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    23 => epd1,

    /// ## A1, bit 22
    /// 
    /// Selects whether TTBR0_EL1 or TTBR1_EL1 defines the ASID. The encoding of this bit is:
    /// 
    /// * 0b0
    ///     - TTBR0_EL1.ASID defines the ASID.
    /// * 0b1
    ///     - TTBR1_EL1.ASID defines the ASID.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    22 => a1,

    /// ## T1SZ, bits 21:16
    /// 
    /// The size offset of the memory region addressed by TTBR1_EL1. The region size is 2^(64-T1SZ) bytes.
    /// 
    /// The maximum and minimum possible values for T1SZ depend on the level of translation table and the memory translation granule size, as described in the AArch64 Virtual Memory System Architecture chapter.
    /// 
    /// > ### Note
    /// > For the 4KB translation granule, if FEAT_LPA2 is implemented and this field is less than 16, the translation table walk begins with a level -1 initial lookup.
    /// > For the 16KB translation granule, if FEAT_LPA2 is implemented and this field is less than 17, the translation table walk begins with a level 0 initial lookup.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    21:16 => t1sz,

    /// ## TG0, bits 15:14
    /// 
    /// Granule size for the TTBR0_EL1. 
    /// 
    /// * 0b01 
    ///     - 16KB.
    /// * 0b10
    ///     - 4KB.
    /// * 0b11
    ///     - 64KB.
    /// 
    /// Other values are reserved.
    /// 
    /// If the value is programmed to either a reserved value or a size that has not been implemented, then the hardware will treat the field as if it has been programmed to an IMPLEMENTATION DEFINED choice of the sizes that has been implemented for all purposes other than the value read back from this register.
    /// 
    /// It is IMPLEMENTATION DEFINED whether the value read back is the value programmed or the value that corresponds to the size chosen.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    15:14 => tg0,

    /// ## SH1, bits 13:12
    /// Shareability attribute for memory associated with translation table walks using TTBR0_EL1. 
    /// 
    /// * 0b00
    ///     - Non-shareable.
    /// * 0b10
    ///     - Outer Shareable.
    /// * 0b11
    ///     - Inner Shareable.
    /// 
    /// Other values are reserved. The effect of programming this field to a Reserved value is that behavior is CONSTRAINED UNPREDICTABLE.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    13:12 => sh0,

    /// ## ORGN0, bits 11:10
    /// 
    /// Outer cacheability attribute for memory associated with translation table walks using TTBR0_EL1.
    /// 
    /// * 0b00 
    ///     - Normal memory, Outer Non-cacheable.
    /// * 0b01 
    ///     - Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable.
    /// * 0b10 
    ///     - Normal memory, Outer Write-Through Read-Allocate No Write-Allocate Cacheable.
    /// * 0b11 
    ///     - Normal memory, Outer Write-Back Read-Allocate No Write-Allocate Cacheable.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    11:10 => orgn0,

    /// ## IRGN0, bits 9:8
    /// 
    /// Inner cacheability attribute for memory associated with translation table walks using TTBR0_EL1.
    /// 
    /// * 0b00 
    ///     - Normal memory, Inner Non-cacheable.
    /// * 0b01 
    ///     - Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable.
    /// * 0b10 
    ///     - Normal memory, Inner Write-Through Read-Allocate No Write-Allocate Cacheable.
    /// * 0b11 
    ///     - Normal memory, Inner Write-Back Read-Allocate No Write-Allocate Cacheable.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    9:8 => irgn0,

    /// ## EPD0, bit 7
    /// 
    /// Translation table walk disable for translations using TTBR0_EL1. This bit controls whether a translation table walk is performed on a TLB miss, for an address that is translated using TTBR0_EL1. The encoding of this bit is:
    /// 
    /// * 0b0 
    ///     - Perform translation table walks using TTBR0_EL1.
    /// * 0b1 
    ///     - A TLB miss on an address that is translated using TTBR0_EL1 generates a Translationfault. No translation table walk is performed.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    7 => epd0,
    // 6 => reserved RES0,

    /// ## T0SZ, bits 5:0
    /// 
    /// The size offset of the memory region addressed by TTBR0_EL1. The region size is 2^(64-T0SZ) bytes.
    /// 
    /// The maximum and minimum possible values for T0SZ depend on the level of translation table and the memory translation granule size, as described in the AArch64 Virtual Memory System Architecture chapter.
    /// 
    /// > ### Note
    /// > For the 4KB translation granule, if FEAT_LPA2 is implemented and this field is less than 16, the translation table walk begins with a level -1 initial lookup.
    /// > For the 16KB translation granule, if FEAT_LPA2 is implemented and this field is less than 17, the translation table walk begins with a level 0 initial lookup.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    5:0 => t0sz

);