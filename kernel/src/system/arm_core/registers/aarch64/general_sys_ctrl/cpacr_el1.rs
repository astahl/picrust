use mystd::bit_field;

use crate::system_register_impl;

system_register_impl!(cpacr_el1 CpAcrEl1 (r,w));

bit_field!(
    /// # CPACR_EL1, Architectural Feature Access Control Register
    /// Controls access to trace, SME, Streaming SVE, SVE, and Advanced SIMD and floating-point functionality.
    pub CpAcrEl1(u64) {
    
    /// # TTA, bit \[28]
    /// Traps EL0 and EL1 System register accesses to all implemented trace registers from both Execution states to EL1, or to EL2 when it is implemented and enabled in the current Security state and HCR_EL2.TGE is 1, as follows:
    /// 
    /// * In AArch64 state, accesses to trace registers are trapped, reported using ESR_ELx.EC value 0x18.
    /// * In AArch32 state, MRC and MCR accesses to trace registers are trapped, reported using ESR_ELx.EC value 0x05.
    /// * In AArch32 state, MRRC and MCRR accesses to trace registers are trapped, reported using ESR_ELx.EC value 0x0C.
    /// 
    /// * 0b0 
    ///     - This control does not cause any instructions to be trapped.
    /// * 0b1 
    ///     - This control causes EL0 and EL1 System register accesses to all implemented trace registers to be trapped.
    /// 
    /// > ### Note
    /// > * The ETMv4 architecture and ETE do not permit EL0 to access the trace registers. If the trace unit implements FEAT_ETMv4 or FEAT_ETE, EL0 accesses to the trace registers are UNDEFINED, and any resulting exception is higher priority than an exception that would be generated because the value of CPACR_EL1.TTA is 1.
    /// > * The Arm architecture does not provide traps on trace register accesses through the optional memory-mapped interface.
    /// 
    /// System register accesses to the trace registers can have side-effects. When a System register access is trapped, any side-effects that are normally associated with the access do not occur before the exception is taken.
    ///
    ///  If System register access to the trace functionality is not implemented, this bit is RES0. 
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    28 => tta,

    /// # SMEN, bits \[25:24]
    /// ## When FEAT_SME is implemented:
    /// Traps execution at EL1 and EL0 of SME instructions, SVE instructions when FEAT_SVE is not implemented or the PE is in Streaming SVE mode, and instructions that directly access the SVCR or SMCR_EL1 System registers to EL1, or to EL2 when EL2 is implemented and enabled in the current Security state and HCR_EL2.TGE is 1.
    /// 
    /// When instructions that directly access the SVCR System register are trapped with reference to this control, the MSR SVCRSM, MSR SVCRZA, and MSR SVCRSMZA instructions are also trapped.
    /// 
    /// The exception is reported using ESR_ELx.EC value of 0x1D, with an ISS code of 0x0000000. This field does not affect whether Streaming SVE or SME register values are valid.
    /// 
    /// A trap taken as a result of CPACR_EL1.SMEN has precedence over a trap taken as a result of CPACR_EL1.FPEN.
    /// 
    /// * 0b00 
    ///     - This control causes execution of these instructions at EL1 and EL0 to be trapped.
    /// * 0b01 
    ///     - This control causes execution of these instructions at EL0 to be trapped, but does not cause execution of any instructions at EL1 to be trapped.
    /// * 0b10 
    ///     - This control causes execution of these instructions at EL1 and EL0 to be trapped.
    /// * 0b11 
    ///     - This control does not cause execution of any instructions to be trapped.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ## Otherwise:
    /// Reserved, RES0.
    #[cfg(feature = "feat_sme")]
    25:24 => smen,

    /// # FPEN, bits [21:20]
    /// Traps execution at EL1 and EL0 of instructions that access the Advanced SIMD and floating-point registers from both Execution states to EL1, reported using ESR_ELx.EC value 0x07, or to EL2 reported using ESR_ELx.EC value 0x00 when EL2 is implemented and enabled in the current Security state and HCR_EL2.TGE is 1, as follows:
    /// 
    /// * In AArch64 state, accesses to FPCR, FPSR, any of the SIMD and floating-point registers V0-V31, including their views as D0-D31 registers or S0-31 registers.
    /// * In AArch32 state, FPSCR, and any of the SIMD and floating-point registers Q0-15, including their views as D0-D31 registers or S0-31 registers.
    /// 
    /// Traps execution at EL1 and EL0 of SME and SVE instructions to EL1, or to EL2 when EL2 is implemented and enabled for the current Security state and HCR_EL2.TGE is 1. The exception is reported using ESR_ELx.EC value 0x07.
    /// 
    /// A trap taken as a result of CPACR_EL1.SMEN has precedence over a trap taken as a result of CPACR_EL1.FPEN.
    /// 
    /// A trap taken as a result of CPACR_EL1.ZEN has precedence over a trap taken as a result of CPACR_EL1.FPEN.
    /// * 0b00 
    ///     - This control causes execution of these instructions at EL1 and EL0 to be trapped.
    /// * 0b01 
    ///     - This control causes execution of these instructions at EL0 to be trapped, but does not cause execution of any instructions at EL1 to be trapped.
    /// * 0b10 
    ///     - This control causes execution of these instructions at EL1 and EL0 to be trapped.
    /// * 0b11 
    ///     - This control does not cause execution of any instructions to be trapped.
    /// 
    /// Writes to MVFR0, MVFR1, and MVFR2 from EL1 or higher are CONSTRAINED UNPREDICTABLE and whether these accesses can be trapped by this control depends on implemented CONSTRAINED UNPREDICTABLE behavior.
    /// 
    /// > ### Note
    /// > * Attempts to write to the FPSID count as use of the registers for accesses from EL1 or higher.
    /// > * Accesses from EL0 to FPSID, MVFR0, MVFR1, MVFR2, and FPEXC are UNDEFINED, and any resulting exception is higher priority than an exception that would be generated because the value of CPACR_EL1.FPEN is not 0b11.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    21:20 => fpen: enum FPEnableFlags {
        TrapEl0El1 = 0b00,
        TrapEl0 = 0b01,
        TrapEl1El0 = 0b10,
        NoTrap = 0b11,
    },

    /// # ZEN, bits \[17:16]
    /// ## When FEAT_SVE is implemented:
    /// Traps execution at EL1 and EL0 of SVE instructions when the PE is not in Streaming SVE mode, and instructions that directly access the ZCR_EL1 System register to EL1, or to EL2 when EL2 is implemented and enabled in the current Security state and HCR_EL2.TGE is 1.
    /// 
    /// The exception is reported using ESR_ELx.EC value 0x19.
    /// 
    /// A trap taken as a result of CPACR_EL1.ZEN has precedence over a trap taken as a result of
    /// CPACR_EL1.FPEN.
    /// * 0b00 
    ///     - This control causes execution of these instructions at EL1 and EL0 to be trapped.
    /// * 0b01 
    ///     - This control causes execution of these instructions at EL0 to be trapped, but does not cause execution of any instructions at EL1 to be trapped.
    /// * 0b10 
    ///     - This control causes execution of these instructions at EL1 and EL0 to be trapped.
    /// * 0b11 
    ///     - This control does not cause execution of any instructions to be trapped.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    /// 
    /// ## Otherwise:
    /// Reserved, RES0.
    #[cfg(feature = "feat_sve")]
    17:16 => zen,


});