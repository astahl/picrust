use crate::delay;
use crate::peripherals::mmio::MMIO;

pub struct Gpio();

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
