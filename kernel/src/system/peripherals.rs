#![allow(dead_code)]

use super::hal::info::MemoryBlock;

pub mod dma;
pub mod gpio;
pub mod mailbox;
pub mod mmio;
pub mod power;
pub mod uart;
pub mod usb;
pub mod interrupts;
pub mod system_timer;

pub struct BcmHost {
    pub peripheral_address: usize,
    pub peripheral_size: usize,
    pub peripheral_range_inclusive: (usize, usize),
    pub sdram_address: usize,
}

#[cfg(feature = "bcm2711")]
pub const BCM_HOST: BcmHost = BcmHost {
    peripheral_address: 0xFE00_0000,
    peripheral_size: 0x0180_0000,
    peripheral_range_inclusive: (0xFE00_0000, 0xFFFF_FFFF),
    sdram_address: 0xC000_0000,
};

#[cfg(any(feature = "bcm2837"))]
pub const BCM_HOST: BcmHost = BcmHost {
    peripheral_address: 0x3F00_0000,
    peripheral_size: 0x0100_0000,
    peripheral_range_inclusive: (0x3F00_0000, 0x3FFF_FFFF),
    sdram_address: 0xC000_0000,
};

pub struct PeripheralMap();

impl core::fmt::Debug for PeripheralMap {

    #[cfg(feature = "bcm2711")]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Peripheral Map for BCM2711 not available")
    }

    #[cfg(any(feature = "bcm2837"))]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Peripherals")
            .field(
                "System Timers",
                &MemoryBlock::from_address_and_size(BCM_HOST.peripheral_address + 0x3000, 0x1c),
            )
            .field(
                "DMA Controller",
                &MemoryBlock::from_address_and_size(
                    BCM_HOST.peripheral_address + dma::DMA_BASE,
                    0x700,
                ),
            )
            .field(
                "Interrupt Controller",
                &MemoryBlock::from_address_and_size(BCM_HOST.peripheral_address + 0xb000, 0x228),
            )
            .field(
                "Timers (ARM Side)",
                &MemoryBlock::from_address_and_size(BCM_HOST.peripheral_address + 0xb000, 0x424),
            )
            .field(
                "Mailbox",
                &MemoryBlock::from_address_and_size(
                    BCM_HOST.peripheral_address + mailbox::MBOX_BASE,
                    0x00,
                ),
            )
            .field(
                "GPIO",
                &MemoryBlock::from_address_and_size(
                    BCM_HOST.peripheral_address + gpio::GPIO_BASE,
                    0xB1,
                ),
            )
            .field(
                "Uart",
                &MemoryBlock::from_address_and_size(
                    BCM_HOST.peripheral_address + uart::UART_BASE,
                    0x100,
                ),
            )
            .field(
                "PCM / I2S",
                &MemoryBlock::from_address_and_size(BCM_HOST.peripheral_address + 0x203000, 0x24),
            ) //? size ?
            .field(
                "Aux Peripherals (MiniUART, SPI1 & 2)",
                &MemoryBlock::from_address_and_size(BCM_HOST.peripheral_address + 0x215000, 0xd6),
            )
            .field(
                "SPI0",
                &MemoryBlock::from_address_and_size(BCM_HOST.peripheral_address + 0x204000, 0x18),
            )
            .field(
                "BSC0",
                &MemoryBlock::from_address_and_size(BCM_HOST.peripheral_address + 0x205000, 0xd6),
            )
            .field(
                "PWM1",
                &MemoryBlock::from_address_and_size(BCM_HOST.peripheral_address + 0x20c000, 0x28),
            )
            .field(
                "PWM2",
                &MemoryBlock::from_address_and_size(BCM_HOST.peripheral_address + 0x20c400, 0x28),
            )
            .field(
                "EMMC",
                &MemoryBlock::from_address_and_size(BCM_HOST.peripheral_address + 0x300000, 0x100),
            )
            .field(
                "BSC1",
                &MemoryBlock::from_address_and_size(BCM_HOST.peripheral_address + 0x804000, 0xd6),
            )
            .field(
                "BSC2",
                &MemoryBlock::from_address_and_size(BCM_HOST.peripheral_address + 0x805000, 0x20),
            )
            .field(
                "USB Core",
                &MemoryBlock::from_address_and_size(
                    BCM_HOST.peripheral_address + usb::USB_CORE_BASE,
                    0x400,
                ),
            )
            .field(
                "USB Host",
                &MemoryBlock::from_address_and_size(
                    BCM_HOST.peripheral_address + usb::USB_HOST_BASE,
                    0xe00 - 0x400,
                ),
            ) //? size ?
            .field(
                "USB POWER",
                &MemoryBlock::from_address_and_size(
                    BCM_HOST.peripheral_address + usb::USB_POWER_BASE,
                    0x0,
                ),
            ) //? size ?
            .finish()
    }
}
