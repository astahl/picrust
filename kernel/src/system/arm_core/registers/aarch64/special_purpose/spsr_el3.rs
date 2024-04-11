use mystd::bit_field;

use crate::system_register_impl;

system_register_impl!(spsr_el3 SpsrEl3 (r,w));
bit_field!(
    /// # C5.2.20 SPSR_EL3, Saved Program Status Register (EL3)
    /// Holds the saved process state when an exception is taken to EL3.
    pub SpsrEl3 (u64) {
        31 => n,
        30 => z,
        29 => c,
        28 => v,
        /// only when exception is taken from Aarch32, else res0
        27 => aarch32_q,
        /// only when exception is taken from Aarch32, else 26: res0
        26:25 => aarch32_it_lsb,
        /// only when exception is taken from Aarch64
        25 => aarch64_tco,
        24 => dit,
        /// only when exception is taken from Aarch64
        23 => aarch64_uao,
        23 => aarch32_ssbs,
        22 => pan,
        21 => ss,
        20 => il,
        19:16 => ge,
        15:10 => aarch32_it_msb,
        13 => aarch64_allint,
        12 => aarch64_ssbs,
        11:10 => aarch64_btype,
        9 => d,
        8 => a,
        7 => i,
        6 => f,
        5 => aarch32_t,
        4 => m: enum ExecutionState {
            AArch64,
            AArch32
        },
        3:0 => aarch32_m: enum AArch32Mode {
            User = 0b0000,
            Fiq = 0b0001,
            Irq = 0b0010,
            Supervisor = 0b0011,
            Monitor = 0b0110,
            Abort = 0b0111,
            Hyp = 0b1010,
            Undefined = 0b1011,
            System = 0b1111,
        },
        3:0 => aarch64_m: enum AArch64Mode {
            EL0t = 0b0000,
            EL1t = 0b0100,
            EL1h = 0b0101,
            EL2t = 0b1000,
            EL2h = 0b1001,
            EL3t = 0b1100,
            EL3h = 0b1101,
        },
    }
);