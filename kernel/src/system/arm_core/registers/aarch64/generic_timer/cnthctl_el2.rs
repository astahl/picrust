use mystd::bit_field;

use crate::system_register_impl;

system_register_impl!(cnthctl_el2 CntHCtlEl2 (r,w));

bit_field!(
    /// # CNTHCTL_EL2, Counter-timer Hypervisor Control register
    /// Controls the generation of an event stream from the physical counter, and access from EL1 to the physical counter and the EL1 physical timer.
    pub CntHCtlEl2(u64) {
    
    #[cfg(feature = "feat_rme")]
    19 => cntpmask,
    #[cfg(feature = "feat_rme")]
    18 => cntvmask,
    #[cfg(feature = "feat_ecv")]
    17 => evntis,
    #[cfg(feature = "feat_ecv")]
    16 => el1nvvct,
    #[cfg(feature = "feat_ecv")]
    15 => el1nvpct,
    #[cfg(feature = "feat_ecv")]
    14 => el1tvct,
    #[cfg(feature = "feat_ecv")]
    13 => el1tvt,
    #[cfg(feature = "feat_ecv")]
    12 => ecv,
    #[cfg(feature = "feat_vhe")]
    11 => el1pten,
    #[cfg(feature = "feat_vhe")]
    10 => el1pcten,
    #[cfg(feature = "feat_vhe")]
    9 => el0pten,
    #[cfg(feature = "feat_vhe")]
    8 => el0vten,

    /// # EVNTI, bits \[7:4]
    /// Selects which bit of CNTPCT_EL0, as seen from EL2,is the trigger for the event stream generated from that counter when that stream is enabled.
    /// 
    /// If FEAT_ECV is implemented, and CNTHCTL_EL2.EVNTIS is 1, this field selects a trigger bit in the range 8 to 23 of CNTPCT_EL0.
    ///
    /// Otherwise, this field selects a trigger bit in the range 0 to 15 of CNTPCT_EL0. 
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    7:4 => evntni,

    /// # EVNTDIR, bit \[3]
    /// Controls which transition of the CNTPCT_EL0 trigger bit, as seen from EL2 and defined by EVNTI, generates an event when the event stream is enabled.
    /// * 0b0 
    ///     - A 0 to 1 transition of the trigger bit triggers an event.
    /// * 0b1 
    ///     - A 1 to 0 transition of the trigger bit triggers an event.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    3 => evntdir,

    /// # EVNTEN, bit \[2]
    /// Enables the generation of an event stream from CNTPCT_EL0 as seen from EL2.
    /// 
    /// * 0b0 
    ///     - Disables the event stream.
    /// * 0b1 
    ///     - Enables the event stream.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    2 => evnten,

    /// # EL1PCEN, bit \[1]
    /// Traps EL0 and EL1 accesses to the EL1 physical timer registers to EL2 when EL2 is enabled in the current Security state, as follows:
    /// 
    /// * In AArch64 state, accesses to CNTP_CTL_EL0, CNTP_CVAL_EL0, CNTP_TVAL_EL0 are trapped to EL2, reported using EC syndrome value 0x18.
    /// * In AArch32 state, MRC or MCR accesses to the following registers are trapped to EL2 reported using EC syndrome value 0x3 and MRRC and MCRR accesses are trapped to EL2, reported using EC syndrome value 0x04:
    ///     - CNTP_CTL, CNTP_CVAL, CNTP_TVAL.
    /// 
    /// * 0b0 
    ///     - From AArch64 state: EL0 and EL1 accesses to the CNTP_CTL_EL0, CNTP_CVAL_EL0, and CNTP_TVAL_EL0 are trapped to EL2 when EL2 is enabled in the current Security state, unless they are trapped by CNTKCTL_EL1.EL0PTEN.
    ///     - From AArch32 state: EL0 and EL1 accesses to the CNTP_CTL, CNTP_CVAL, and CNTP_TVAL are trapped to EL2 when EL2 is enabled in the current Security state, unless they are trapped by CNTKCTL_EL1.EL0PTEN or CNTKCTL.PL0PTEN.
    /// * 0b1
    ///     - This control does not cause any instructions to be trapped.
    /// 
    /// If EL3 is implemented and EL2 is not implemented, behavior is as if this bit is 1 other than for the purpose of a direct read.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    1 => el1pcen,
    
    /// # EL1PCTEN, bit \[0]
    /// Traps EL0 and EL1 accesses to the EL1 physical counter register to EL2 when EL2 is enabled in the current Security state, as follows:
    /// 
    /// * In AArch64 state, accesses to CNTPCT_EL0 are trapped to EL2, reported using EC syndrome value 0x18.
    /// * In AArch32 state, MRRC or MCRR accesses to CNTPCT are trapped to EL2, reported using EC syndrome value 0x04.
    /// * 0b0 
    ///     - From AArch64 state: EL0 and EL1 accesses to the CNTPCT_EL0 are trapped to EL2 when EL2 is enabled in the current Security state, unless they are trapped by CNTKCTL_EL1.EL0PCTEN.
    ///     - From AArch32 state: EL0 and EL1 accesses to the CNTPCT are trapped to EL2 when EL2 is enabled in the current Security state, unless they are trapped by CNTKCTL_EL1.EL0PCTEN or CNTKCTL.PL0PCTEN.
    /// * 0b1 
    ///     - This control does not cause any instructions to be trapped.
    /// 
    /// If EL3 is implemented and EL2 is not implemented, behavior is as if this bit is 1 other than for the
    /// purpose of a direct read.
    /// 
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to an architecturally UNKNOWN value.
    0 => el1pcten,
});