use core::arch::asm;

use mystd::bit_field;

impl CpuECtlREl1 {
    pub fn load_register() -> Self {
        let value: u64;
        unsafe { asm!("mrs {0}, cpuectlr_el1", out(reg) value) };
        value.into()
    }
    
    pub fn write_register(self) {
        unsafe { asm!("msr cpuectlr_el1, {}", in(reg) self.0) };
    }
}

/// CPU Extended Control Register, EL1
/// Provides additional IMPLEMENTATION DEFINED configuration and control options for the processor.
/// 
#[cfg(feature = "cortex_a72")]
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
        _0Requests = 0b00,
        _1Request = 0b01,
        _2Requests = 0b10,
        /// 3 requests. This is the reset value.
        _3Requests = 0b11,
    } = IFetchDistance::_3Requests,

    33:32 => l2_load_data_prefetch_distance: enum DFetchDistance {
        _16Requests = 0b00,
        _18Requests = 0b01,
        _20Requests = 0b10,
        /// 22 requests. This is the reset value.
        _22Requests = 0b11,
    } = DFetchDistance::_22Requests,

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