#![allow(dead_code)]



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

mod mmio {

    pub fn write_to(ptr: *mut u32, data: u32) {
        unsafe { core::ptr::write_volatile(ptr, data) };
    }

    pub fn read_from(ptr: *const u32) -> u32 {
        unsafe { core::ptr::read_volatile(ptr) }
    }

    pub struct  MMIO<const BASE: usize, const OFFSET: usize>();
    impl<const BASE: usize, const OFFSET: usize> MMIO<BASE, OFFSET> {
        const ADDRESS: usize = crate::peripherals::BCM_HOST.peripheral_address + BASE + OFFSET;

        pub fn write(&self, data: u32) {
            write_to(Self::ADDRESS as *mut u32, data);
        }

        pub fn read(&self) -> u32 {
            read_from(Self::ADDRESS as *const u32)
        }

        pub fn update(&self, mask: u32, data: u32) -> u32 {
            let old_value = self.read();
            let new_value = (!mask & old_value) | (mask & data);
            self.write(new_value);
            old_value
        }
    }
}

mod gpio {

    use crate::peripherals::delay;
    use crate::peripherals::mmio::MMIO;

    pub struct Gpio ();

    const GPIO_BASE: usize = 0x200000;
    impl Gpio {
        const GPPUD: MMIO<GPIO_BASE, 0x94> = MMIO();
        const GPPUDCLK0: MMIO<GPIO_BASE, 0x98> = MMIO();
    
        pub fn init_uart0() {
            // select GPIO Pin Update Disable
            Self::GPPUD.write(0x00000000);
            delay(150);
    
            // select Pin 14 and 15
            Self::GPPUDCLK0.write((1 << 14) | (1 << 15));
            delay(150);
    
            // Commit Pin Update
            Self::GPPUDCLK0.write(0x00000000);
        }
    }
}

pub mod uart;
pub mod mailbox;
