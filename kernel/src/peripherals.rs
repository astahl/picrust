#![allow(dead_code)]

pub mod gpio;
pub mod mailbox;
pub mod mmio;
pub mod uart;

pub struct BcmHost {
    pub peripheral_address: usize,
    pub peripheral_size: usize,
    pub sdram_address: usize,
}

#[cfg(feature = "bcm2711")]
pub const BCM_HOST: BcmHost = BcmHost {
    peripheral_address: 0xFE000000,
    peripheral_size: 0x01800000,
    sdram_address: 0xC0000000,
};

#[cfg(any(feature = "bcm2837"))]
pub const BCM_HOST: BcmHost = BcmHost {
    peripheral_address: 0x3F000000,
    peripheral_size: 0x01000000,
    sdram_address: 0xC0000000,
};
