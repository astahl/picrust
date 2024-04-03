use core::arch::asm;

use mystd::bit_field;

use crate::system_register_impl;

#[cfg(any(feature = "cortex_a72", feature = "cortex_a53"))]
system_register_impl!(cpuectlr_el1 CpuECtlREl1 (r,w));

/// CPU Extended Control Register, EL1
/// Provides additional IMPLEMENTATION DEFINED configuration and control options for the processor.
///
#[cfg(any(feature = "cortex_a72", feature = "cortex_a53"))]
bit_field!(pub CpuECtlREl1 (u64) {

    /// # Disable table walk descriptor access prefetch
    /// * 0
    ///     - Enables table walk descriptor access prefetch. This is the reset value.
    /// * 1
    ///     - Disables table walk descriptor access prefetch.
    38 => disable_table_walk_descriptor_access_prefetch: enum TableWalkDescriptorAccessPrefetch {
        Enable,
        Disable
    } = TableWalkDescriptorAccessPrefetch::Enable,

    /// Indicates the L2 instruction fetch prefetch distance. It is the number of requests by which the prefetcher is ahead of the demand request stream. It also specifies the maximum number of prefetch requests generated on a demand miss.
    36:35 => l2_instruction_fetch_prefetch_distance: enum IFetchDistance {
        /// 0 requests, disables instruction prefetch.
        Requests0 = 0b00,
        Request1 = 0b01,
        Requests2 = 0b10,
        Requests3 = 0b11,
    } = IFetchDistance::Requests3,

    33:32 => l2_load_data_prefetch_distance: enum DFetchDistance {
        Requests16 = 0b00,
        Requests18 = 0b01,
        Requests20 = 0b10,
        Requests22 = 0b11,
    } = DFetchDistance::Requests22,

    /// Enables the processor to receive instruction cache and TLB maintenance operations broadcast from other processors in the cluster.
    ///
    /// __You must set this bit before enabling the caches and MMU, or performing any cache and TLB maintenance operations.__
    ///
    /// You must clear this bit during a processor power down sequence. See 2.4 Power management on page 2-42.
    ///
    /// > ### Notes:
    /// > * Any processor instruction cache and TLB maintenance operations can execute the request, regardless of the value of the SMPEN bit.
    /// > * This bit has no impact on data cache maintenance operations.
    /// > * In the Cortex-A72 processor, the L1 data cache and L2 cache are always coherent, for shared or non-shared data, regardless of the value of the SMPEN bit.
    6 => smpen: enum SMPEN {
        Disable,
        Enable
    } = SMPEN::Disable,

    /// Processor dynamic retention control.
    2:0 => processor_dynamic_retention_control: enum DynamicRetentionDelay {
        /// Processor dynamic retention disabled. This is the reset value.
        Disable = 0b000,
        /// 2 Generic Timer ticks required before retention entry.
        _2Ticks = 0b001,
        /// 8 Generic Timer ticks required before retention entry.
        _8Ticks = 0b010,
        /// 32 Generic Timer ticks required before retention entry.
        _32Ticks = 0b011,
        /// 64 Generic Timer ticks required before retention entry.
        _64Ticks = 0b100,
        /// 128 Generic Timer ticks required before retention entry.
        _128Ticks = 0b101,
        /// 256 Generic Timer ticks required before retention entry.
        _256Ticks = 0b110,
        /// 512 Generic Timer ticks required before retention entry.
        _512Ticks = 0b111,
    } = DynamicRetentionDelay::Disable,
});
