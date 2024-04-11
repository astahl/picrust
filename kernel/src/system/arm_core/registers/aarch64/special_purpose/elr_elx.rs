use crate::system_register_impl;


system_register_impl!(elr_el3 ElrEl3 (r,w));
system_register_impl!(elr_el2 ElrEl2 (r,w));
system_register_impl!(elr_el1 ElrEl1 (r,w));

pub struct ElrEl<const EL: usize>(u64);
pub type ElrEl1 = ElrEl<1>;
pub type ElrEl2 = ElrEl<2>;
pub type ElrEl3 = ElrEl<3>;

impl<const EL: usize> ElrEl<EL> {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> u64 {
        self.0
    }
}
