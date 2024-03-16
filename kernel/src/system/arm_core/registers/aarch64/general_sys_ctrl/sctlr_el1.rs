use mystd::bit_field;


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
    pub SctlrEl1(u64)
    
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
    