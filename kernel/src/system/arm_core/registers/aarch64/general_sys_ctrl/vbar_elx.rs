use crate::system_register_impl;

system_register_impl!(vbar_el3 VbarEl3 (r,w));
system_register_impl!(vbar_el2 VbarEl2 (r,w));
system_register_impl!(vbar_el1 VbarEl1 (r,w));

/// # VBAR_ELn, Vector Base Address Register (ELn)
/// Holds the vector base address for any exception that is taken to ELn.
pub struct VbarEl<const EL: usize>(u64);

/// # VBAR_EL1, Vector Base Address Register (EL1)
/// Holds the vector base address for any exception that is taken to EL1.
pub type VbarEl1 = VbarEl<1>;

/// # VBAR_EL2, Vector Base Address Register (EL2)
/// Holds the vector base address for any exception that is taken to EL2
pub type VbarEl2 = VbarEl<2>;

/// # VBAR_EL3, Vector Base Address Register (EL3)
/// Holds the vector base address for any exception that is taken to EL3
pub type VbarEl3 = VbarEl<3>;

impl<const EL: usize> VbarEl<EL> {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> u64 {
        self.0
    }
}
