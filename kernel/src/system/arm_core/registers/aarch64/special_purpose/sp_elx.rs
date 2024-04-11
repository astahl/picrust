use crate::system_register_impl;

system_register_impl!(sp_el3 SpEl3 (r,w));
system_register_impl!(sp_el2 SpEl2 (r,w));
system_register_impl!(sp_el1 SpEl1 (r,w));
system_register_impl!(sp_el0 SpEl0 (r,w));

/// # SP_ELn, Vector Base Address Register (ELn)
/// Holds the vector base address for any exception that is taken to ELn.
pub struct SpEl<const EL: usize>(u64);

/// # SP_EL0, Stack Pointer (EL0)
/// Holds the stack pointer associated with EL0. At higher Exception levels, this is used as the current stack pointer when the value of SPSel.SP is 0.
pub type SpEl0 = SpEl<0>;

/// # SP_EL1, Stack Pointer (EL1)
/// Holds the stack pointer associated with EL1. When executing at EL1, the value of SPSel.SP determines the current stack pointer:
/// 
/// | SPSel.SP | Current stack pointer |
/// | -------- | --------------------- |
/// | 0b0 | SP_EL0 |
/// | 0b1 | SP_EL1 |
pub type SpEl1 = SpEl<1>;

/// # SP_EL2, Stack Pointer (EL2)
/// Holds the stack pointer associated with EL2. When executing at EL2, the value of SPSel.SP determines the current stack pointer:
/// 
/// | SPSel.SP | Current stack pointer |
/// | -------- | --------------------- |
/// | 0b0 | SP_EL0 |
/// | 0b1 | SP_EL2 |
pub type SpEl2 = SpEl<2>;

/// # SP_EL3, Stack Pointer (EL3)
/// Holds the stack pointer associated with EL3. When executing at EL3, the value of SPSel.SP determines the current stack pointer:
/// 
/// | SPSel.SP | Current stack pointer |
/// | -------- | --------------------- |
/// | 0b0 | SP_EL0 |
/// | 0b1 | SP_EL3 |
pub type SpEl3 = SpEl<3>;

impl<const EL: usize> SpEl<EL> {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> u64 {
        self.0
    }
}
