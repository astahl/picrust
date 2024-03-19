use core::arch::asm;

use mystd::bit_field;


pub fn read() -> IdAa64Mmfr0El1 {
    let value: u64;
    unsafe { asm!("mrs {0}, id_aa64mmfr0_el1", out(reg) value) };
    value.into()
}


bit_field!(pub IdAa64Mmfr0El1(u64){
    /// # ECV
    /// Indicates presence of Enhanced Counter Virtualization.
    /// All other values are reserved.
    /// FEAT_ECV implements the functionality identified by the values 0b0001 and 0b0010.
    /// From Armv8.6, the only permitted values are 0b0001 and 0b0010.
    63:60 => ecv: enum EnhancedCounterVisualization {
        /// Enhanced Counter Virtualization is not implemented.
        NotImplemented = 0b0000,
        /// Enhanced Counter Virtualization is implemented. Supports CNTHCTL_EL2.{EL1TVT, EL1TVCT, EL1NVPCT, EL1NVVCT, EVNTIS}, CNTKCTL_EL1.EVNTIS, CNTPCTSS_EL0 counter views, and CNTVCTSS_EL0 counter views. Extends the PMSCR_EL1.PCT, PMSCR_EL2.PCT, TRFCR_EL1.TS, and TRFCR_EL2.TS fields.
        Implemented = 0b0001,
        /// As 0b0001, and also includes support for CNTHCTL_EL2.ECV and CNTPOFF_EL2.
        Extended = 0b0010
    },

    /// # FGT
    /// Indicates presence of the Fine-Grained Trap controls.
    /// All other values are reserved.
    /// FEAT_FGT implements the functionality identified by the value 0b0001.
    /// From Armv8.6, the value 0b0000 is not permitted.
    59:56 => fgt: enum FineGrainedTrapControls {
        /// Fine-grained trap controls are not implemented.
        NotImplemented = 0b0000,
        /// Fine-grained trap controls are implemented. Supports:
        /// * If EL2 is implemented, the HAFGRTR_EL2, HDFGRTR_EL2, HDFGWTR_EL2, HFGRTR_EL2, HFGITR_EL2 and HFGWTR_EL2 registers, and their associated traps.
        /// * If EL2 is implemented, MDCR_EL2.TDCC.
        /// * If EL3 is implemented, MDCR_EL3.TDCC.
        /// * If both EL2 and EL3 are implemented, SCR_EL3.FGTEn.
        Implemented = 0b0001,
    },

    //55:48 => RES0,

    /// # ExS
    /// Indicates support for disabling context synchronizing exception entry and exit.
    /// All other values are reserved.
    /// FEAT_ExS implements the functionality identified by the value 0b0001.
    47:44 => ex_s: enum NonContextSynchronizingExceptions {
        /// All exception entries and exits are context synchronization events.
        NotSupported = 0b0000,
        /// Non-context synchronizing exception entry and exit are supported.
        Supported = 0b0001
    },

    /// # TGran4_2
    /// Indicates support for 4KB memory granule size at stage 2.
    /// All other values are reserved.
    43:40 => t_gran4_2: enum Stage2Granule4KBSupport {
        /// __Deprecated__ Support for 4KB granule at stage 2 is identified in the ID_AA64MMFR0_EL1.TGran4 field.
        #[deprecated]
        IndicatedByTGran4 = 0b0000,
        /// 4KB granule not supported at stage 2.
        NotSupportedAtStage2 = 0b0001,
        /// 4KB granule supported at stage 2.
        SupportedAtStage2 = 0b0010,
        /// _When FEAT_LPA2 is implemented:_
        /// 4KB granule at stage 2 supports 52-bit input addresses and can describe 52-bit output addresses.
        Supports52BitAtStage2 = 0b0011
    },

    /// # TGran64_2
    /// Indicates support for 64KB memory granule size at stage 2.
    /// All other values are reserved.
    39:36 => t_gran64_2: enum Stage2Granule64KBSupport {
        /// __Deprecated__ Support for 64KB granule at stage 2 is identified in the ID_AA64MMFR0_EL1.TGran64 field.
        #[deprecated]
        IndicatedByTGran64 = 0b0000,
        /// 64KB granule not supported at stage 2.
        NotSupportedAtStage2 = 0b0001,
        /// 64KB granule supported at stage 2.
        SupportedAtStage2 = 0b0010,
    },

    /// # TGran16_2
    /// Indicates support for 16KB memory granule size at stage 2.
    /// All other values are reserved.
    35:32 => t_gran16_2: enum Stage2Granule16KBSupport {
        /// __Deprecated__ Support for 16KB granule at stage 2 is identified in the ID_AA64MMFR0_EL1.TGran4 field.
        #[deprecated]
        IndicatedByTGran16 = 0b0000,
        /// 16KB granule not supported at stage 2.
        NotSupportedAtStage2 = 0b0001,
        /// 16KB granule supported at stage 2.
        SupportedAtStage2 = 0b0010,
        /// _When FEAT_LPA2 is implemented:_
        /// 16KB granule at stage 2 supports 52-bit input addresses and can describe 52-bit output addresses.
        Supports52BitAtStage2 = 0b0011
    },

    /// # TGran4
    /// Indicates support for 4KB memory translation granule size. 
    31:28 => t_gran4: 
    /// Value of the ID_AA64MMFR0_EL1.TGran4 field
    enum Granule4KBSupport {
        /// 4KB granule supported.
        Supported = 0b0000, 
        /// _When FEAT_LPA2 is implemented:_
        /// 4KB granule supports 52-bit input addresses and can describe 52-bit output addresses.
        Supports52Bit = 0b0001,
        /// 4KB granule not supported.
        NotSupported = 0b1111 
    },

    /// # TGran64
    /// Indicates support for 64KB memory translation granule size. 
    27:24 => t_gran64: 
    /// Value of the ID_AA64MMFR0_EL1.TGran64 field
    enum Granule64KBSupport {
        /// 64KB granule supported.
        Supported = 0b0000, 
        /// 64KB granule not supported.
        NotSupported = 0b1111 
    },

    /// # TGran16
    /// Indicates support for 16KB memory translation granule size. 
    23:20 => t_gran16: 
    /// Value of the ID_AA64MMFR0_EL1.TGran16 field
    enum Granule16KBSupport {
        /// 16KB granule supported.
        Supported = 0b0000, 
        /// _When FEAT_LPA2 is implemented:_
        /// 16KB granule supports 52-bit input addresses and can describe 52-bit output addresses.
        Supports52Bit = 0b0001,
        /// 16KB granule not supported.
        NotSupported = 0b1111 
    },

    /// # BigEndEL0
    /// Indicates support for mixed-endian at EL0 only.
    /// 
    /// This field is invalid and is RES0 if ID_AA64MMFR0_EL1.BigEnd is not 0b0000.
    19:16 => big_end_el0: enum MixedEndianAtEl0Support {
        /// No mixed-endian support at EL0. The SCTLR_EL1.E0E bit has a fixed value.
        NotSupported = 0b0000,
        /// Mixed-endian support at EL0. The SCTLR_EL1.E0E bit can be configured.
        Supported = 0b0001
    },

    /// # SNSMem
    /// Indicates support for a distinction between Secure and Non-secure Memory.
    /// > ### Note
    /// > If EL3 is implemented, the value 0b0000 is not permitted. 
    15:12 => s_ns_mem: enum MemorySecurityDistinctionSupport {
        /// Does not support a distinction between Secure and Non-secure Memory.
        NotSupported = 0b0000,
        /// Does support a distinction between Secure and Non-secure Memory.
        Supported = 0b0001
    },

    /// # BigEnd
    /// Indicates support for mixed-endian configuration.
    11:8 => big_end: enum MixedEndianSupport {
        /// No mixed-endian support. The SCTLR_ELx.EE bits have a fixed value. See the BigEndEL0 field, bits\[19:16], for whether EL0 supports mixed-endian.
        NotSupported = 0b0000,
        /// Mixed-endian support. The SCTLR_ELx.EE and SCTLR_EL1.E0E bits can be configured.
        Supported = 0b0001
    },

    /// # ASID
    /// Number of ASID bits.
    7:4 => asid: enum AsidBitNum {
        /// 8 bits ASID
        _8Bits = 0b0000,
        /// 16 bits ASID
        _16Bits = 0b0010
    },

    /// # PARange
    /// Physical Address range supported
    3:0 => pa_range: 
    #[derive(PartialOrd)]
    enum PhysicalAddressRangeSupport {
        /// 32 bits, 4GB.
        _32Bits4GB = 0b0000,
        /// 36 bits, 64GB.
        _36Bits64GB = 0b0001,
        /// 40 bits, 1TB.
        _40Bits1TB = 0b0010,
        /// 42 bits, 4TB.
        _42Bits4TB = 0b0011,
        /// 44 bits, 16TB.
        _44Bits16TB = 0b0100,
        /// 48 bits, 256TB.
        _48Bits256TB = 0b0101,
        /// _When FEAT_LPA is implemented or FEAT_LPA2 is implemented:_
        /// 52 bits, 4PB.
        _52Bits4PB = 0b0110,
        Reserved = 0b0111,
    }
});


