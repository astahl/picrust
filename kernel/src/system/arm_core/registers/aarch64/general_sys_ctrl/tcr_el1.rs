use mystd::bit_field;

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
    pub TcrEl1(u64){
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
    
    });