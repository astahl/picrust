use core::arch::asm;

use mystd::bit_field;

use crate::system_register_impl;

system_register_impl!(cntpct_el0 CntPCtEl0 (r,read_ordered));

bit_field!(
    /// # CNTPCT_EL0, Counter-timer Physical Count register
    /// Holds the 64-bit physical count value.
    pub CntPCtEl0(u64) {

    /// # Physical count value.
    /// 
    /// Reads of CNTPCT_EL0 from EL0 or EL1 return (PhysicalCountInt<63:0> - CNTPOFF_EL2<63:0>) if the access is not trapped, and all of the following are true:
    /// 
    /// * CNTHCTL_EL2.ECV is 1.
    /// * HCR_EL2.{E2H, TGE} is not {1, 1}.
    /// 
    /// Where PhysicalCountInt<63:0> is the physical count returned when CNTPCT_EL0 is read from EL2 or EL3.
    63:0 => physical_count_value
});