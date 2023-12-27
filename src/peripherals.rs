#![allow(dead_code)]

pub mod mmio;
pub mod gpio;
pub mod uart;
pub mod mailbox;


pub struct BcmHost {
    pub peripheral_address: usize,
    pub peripheral_size: usize,
    pub sdram_address: usize
}


#[cfg(feature="raspi1")]
compile_error!("Can't compile for Raspberry Pi Model 1 / Zero.");

#[cfg(
    any(
        all(feature="raspi4", feature="raspi3b"), 
        all(feature="raspi2b", feature="raspi3b"),
        all(feature="raspi2b", feature="raspi4"), 
    ))]
compile_error!("Can't compile for multiple Raspberry Pi Models.");

#[cfg(feature="bcm2711")]
pub const BCM_HOST: BcmHost = BcmHost{
    peripheral_address: 0xFE000000,
    peripheral_size: 0x01800000,
    sdram_address: 0xC0000000,
};

#[cfg(any(feature="bcm2837", feature="bcm2836"))]
pub const BCM_HOST: BcmHost = BcmHost{
    peripheral_address: 0x3F000000,
    peripheral_size: 0x01000000,
    sdram_address: 0xC0000000,
};

#[cfg(feature="bcm2835")]
pub const BCM_HOST: BcmHost = BcmHost{
    peripheral_address: 0x20000000,
    peripheral_size: 0x01000000,
    sdram_address: 0x40000000,
};

