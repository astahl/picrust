use core::arch::asm;

use mystd::bit_field;


/// # MPIDR_EL1, Multiprocessor Affinity Register
/// 
/// In a multiprocessor system, provides an additional PE identification mechanism for scheduling purposes.
/// 
/// AArch64 System register MPIDR_EL1 bits \[31:0] are architecturally mapped to AArch32 System register MPIDR\[31:0].
/// In a uniprocessor system, Arm recommends that each Aff<n> field of this register returns a value of 0.
pub fn read() -> MpidrEl1 {
    let value: usize;
    unsafe { asm!("mrs {0}, mpidr_el1", out(reg) value) };
    value.into()
}


bit_field!(pub MpidrEl1(usize){
    // 63:40 => RES0,
    /// Affinity level 3. See the description of Aff0 for more information. Aff3 is not supported in AArch32 state.
    /// 
    /// Not supported on Cortex-A72, where it is RES0
    // #[cfg(not(feature = "cortex_a72"))]
    39:32 => aff_3: u8,
    // 31 => RES1,
    /// Indicates a Uniprocessor system, as distinct from PE 0 in a multiprocessor system.
    30 => u: enum UniprocessorFlag {
        /// Processor is part of a multiprocessor system.
        Multiprocessor,
        /// Processor is part of a uniprocessor system.
        Uniprocessor
    },
    // 29:25 => RES0
    /// Indicates whether the lowest level of affinity consists of logical PEs that are implemented using a multithreading type approach. See the description of Aff0 for more information about affinity levels.
    24 => mt: enum MultithreadingFlag {
        /// Performance of PEs with different affinity level 0 values, and the same values for affinity level 1 and higher, is largely independent.
        LargelyIndependent,
        /// Performance of PEs with different affinity level 0 values, and the same values for affinity level 1 and higher, is very interdependent.
        VeryInterdependent
    },
    /// Affinity level 2. See the description of Aff0 for more information.
    23:16 => aff_2: u8,
    /// Affinity level 1. See the description of Aff0 for more information.
    15:8 => aff_1: u8,
    /// Affinity level 0. 
    /// 
    /// This is the affinity level that is most significant for determining PE behavior. Higher affinity levels are increasingly less significant in determining PE behavior. The assigned value of the MPIDR.{Aff2, Aff1, Aff0} or MPIDR_EL1.{Aff3, Aff2, Aff1, Aff0} set of fields of each PE must be unique within the system as a whole.
    /// 
    /// Not supported on Cortex-A72, where it is RES0
    //#[cfg(not(feature = "cortex_a72"))]
    7:0 => aff_0: u8,
    /// Indicates the core number in the Cortex-A72 processor. The possible values are:
    /// * `0x0` in a cluster with one processor only.
    /// * `0x0` or `0x1` in a cluster with two processors.
    /// * `0x0`, `0x1`, or `0x2` in a cluster with three processors.
    /// * `0x0`, `0x1`, `0x2`, or `0x3` in a cluster with four processors.
    //#[cfg(feature = "cortex_a72")]
    1:0 => cpu_id
});
