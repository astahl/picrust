use crate::{peripherals::mmio::Mmio, system::hal::thread};

pub struct Gpio();

pub const GPIO_BASE: usize = 0x200000;

#[derive(Clone, Copy)]
pub struct PinSet(u32, u32);

impl PinSet {
    pub const fn select(pins: &[u8]) -> Self {
        let mut i = 0;
        let mut result = Self(0, 0);
        while i < pins.len() {
            let pin = pins[i];
            i += 1;
            if pin < 32 {
                result.0 |= 1 << pin;
            } else if pin < 54 {
                let pin = pin - 32;
                result.1 |= 1 << pin;
            } else {
                panic!("Pin number out of range (>=54)");
            }
        }
        result
    }
}

impl IntoIterator for PinSet {
    type Item = u8;

    type IntoIter = PinSelectIterator;

    fn into_iter(self) -> Self::IntoIter {
        PinSelectIterator {
            select: self,
            position: 0,
        }
    }
}

pub struct PinSelectIterator {
    select: PinSet,
    position: u8,
}

impl Iterator for PinSelectIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        while self.position < 54 {
            let pos = self.position;
            self.position += 1;
            if pos < 32 {
                if self.select.0 & (1 << pos) != 0 {
                    return Some(pos);
                }
            } else {
                if self.select.1 & (1 << (pos - 32)) != 0 {
                    return Some(pos);
                }
            }
        }
        None
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum PinFunction {
    Input = 0b000,
    Output = 0b001,
    Alt0 = 0b100,
    Alt1 = 0b101,
    Alt2 = 0b110,
    Alt3 = 0b111,
    Alt4 = 0b011,
    Alt5 = 0b010,
}

impl Gpio {
    // Function Select Registers
    const GPFSEL0: Mmio<GPIO_BASE, 0x00> = Mmio();
    const GPFSEL1: Mmio<GPIO_BASE, 0x04> = Mmio();
    const GPFSEL2: Mmio<GPIO_BASE, 0x08> = Mmio();
    const GPFSEL3: Mmio<GPIO_BASE, 0x0c> = Mmio();
    const GPFSEL4: Mmio<GPIO_BASE, 0x10> = Mmio();
    const GPFSEL5: Mmio<GPIO_BASE, 0x14> = Mmio();

    pub fn set_functions(pins: PinSet, function: PinFunction) {
        let mut state = [
            Self::GPFSEL0.read(),
            Self::GPFSEL1.read(),
            Self::GPFSEL2.read(),
            Self::GPFSEL3.read(),
            Self::GPFSEL4.read(),
            Self::GPFSEL5.read(),
        ];
        let mut updated = [false, false, false, false, false, false];
        for pin in pins {
            let bank_select = (pin as usize) / 10;
            let offset = (pin % 10) * 3;
            let old_state = state[bank_select];
            let mask = 0b111_u32 << offset;
            let new_value = (function as u32) << offset;
            let old_value = old_state & mask;
            if new_value != old_value {
                state[bank_select] = (old_state & !mask) | new_value;
                updated[bank_select] = true;
            }
        }
        if updated[0] {
            Self::GPFSEL0.write(state[0]);
        }
        if updated[1] {
            Self::GPFSEL1.write(state[1]);
        }
        if updated[2] {
            Self::GPFSEL2.write(state[2]);
        }
        if updated[3] {
            Self::GPFSEL3.write(state[3]);
        }
        if updated[4] {
            Self::GPFSEL4.write(state[4]);
        }
        if updated[5] {
            Self::GPFSEL5.write(state[5]);
        }
    }
}

#[cfg(feature = "bcm2837")]
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Resistor {
    None = 0,
    PullDown = 0b01,
    PullUp = 0b10,
    Reserved = 0b11,
}

#[cfg(any(feature = "bcm2811", feature = "bcm2812"))]
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Resistor {
    None = 0,
    PullUp = 0b01,
    PullDown = 0b10,
    Reserved = 0b11,
}

#[cfg(feature = "bcm2837")]
impl Gpio {
    // Pull-up/down Register
    const GPPUD: Mmio<GPIO_BASE, 0x94> = Mmio();
    // Pull-up/down Clock Registers
    const GPPUDCLK0: Mmio<GPIO_BASE, 0x98> = Mmio();
    const GPPUDCLK1: Mmio<GPIO_BASE, 0x9c> = Mmio();

    pub fn set_pull_resistors(pins: PinSet, resistor: Resistor) {
        // The GPIO Pull-up/down Clock Registers control the actuation of internal pull-downs on the respective GPIO pins. These registers must be used in conjunction with the GPPUD register to effect GPIO Pull-up/down changes.
        // The following sequence of events is required:
        // 1. Write to GPPUD to set the required control signal (i.e. Pull-up or Pull-Down or neither to remove the current Pull-up/down)
        Self::GPPUD.write(resistor as u32);
        // 2. Wait 150 cycles – this provides the required set-up time for the control signal
        thread::spin_wait_cycles(150);
        // 3. Write to GPPUDCLK0/1 to clock the control signal into the GPIO pads you wish to modify – NOTE only the pads which receive a clock will be modified, all others will retain their previous state.
        Self::GPPUDCLK0.write(pins.0);
        Self::GPPUDCLK1.write(pins.1);
        // 4. Wait 150 cycles – this provides the required hold time for the control signal
        thread::spin_wait_cycles(150);
        // 5. Write to GPPUD to remove the control signal
        Self::GPPUD.write(Resistor::None as u32);
        // 6. Write to GPPUDCLK0/1 to remove the clock
        Self::GPPUDCLK0.write(0);
        Self::GPPUDCLK1.write(0);
    }
}

#[cfg(any(feature = "bcm2811", feature = "bcm2812"))]
impl Gpio {
    // Pull-up/down Control Registers
    const GPIO_PUP_PDN_CNTRL_REG0: Mmio<GPIO_BASE, 0xe4> = Mmio();
    const GPIO_PUP_PDN_CNTRL_REG1: Mmio<GPIO_BASE, 0xe8> = Mmio();
    const GPIO_PUP_PDN_CNTRL_REG2: Mmio<GPIO_BASE, 0xec> = Mmio();
    const GPIO_PUP_PDN_CNTRL_REG3: Mmio<GPIO_BASE, 0xf0> = Mmio();

    const fn get_pull_control_register(bank_select: usize) -> u32 {
        match bank_select {
            0 => Self::GPIO_PUP_PDN_CNTRL_REG0.read(),
            1 => Self::GPIO_PUP_PDN_CNTRL_REG1.read(),
            2 => Self::GPIO_PUP_PDN_CNTRL_REG2.read(),
            3 => Self::GPIO_PUP_PDN_CNTRL_REG3.read(),
        }
    }

    const fn set_pull_control_register(bank_select: usize, value: u32) {
        match bank_select {
            0 => Self::GPIO_PUP_PDN_CNTRL_REG0.write(value),
            1 => Self::GPIO_PUP_PDN_CNTRL_REG1.write(value),
            2 => Self::GPIO_PUP_PDN_CNTRL_REG2.write(value),
            3 => Self::GPIO_PUP_PDN_CNTRL_REG3.write(value),
        }
    }

    pub fn set_pull_resistor(pin: u8, resistor: Resistor) {
        let offset = (pin << 1) & 0x1f; // (pin * 2) % 32
        let bank_select = pin >> 4; // (pin * 2) / 32
        let old_state = Self::get_pull_control_register(bank_select);
        let mask = 0b11_u32 << offset;
        let new_value = (resistor as u32) << offset;
        let old_value = old_state & mask;
        if new_value != old_value {
            Self::set_pull_control_register(bank_select, (old_state & !mask) | new_value);
        }
    }

    pub fn set_pull_resistors(pins: PinSet, resistor: Resistor) {
        let mut state = [
            Self::GPIO_PUP_PDN_CNTRL_REG0.read(),
            Self::GPIO_PUP_PDN_CNTRL_REG1.read(),
            Self::GPIO_PUP_PDN_CNTRL_REG2.read(),
            Self::GPIO_PUP_PDN_CNTRL_REG3.read(),
        ];
        let mut updated = [false, false, false, false];
        for pin in pins {
            let offset = (pin << 1) & 0x1f; // (pin * 2) % 32
            let bank_select = pin >> 4; // (pin * 2) / 32
            let old_state = state[bank_select];
            let mask = 0b11_u32 << offset;
            let new_value = (resistor as u32) << offset;
            let old_value = old_state & mask;
            if new_value != old_value {
                state[bank_select] = (old_state & !mask) | new_value;
                updated[bank_select] = true;
            }
        }
        if updated[0] {
            Self::GPIO_PUP_PDN_CNTRL_REG0.write(state[0]);
        }
        if updated[1] {
            Self::GPIO_PUP_PDN_CNTRL_REG1.write(state[1]);
        }
        if updated[2] {
            Self::GPIO_PUP_PDN_CNTRL_REG2.write(state[2]);
        }
        if updated[3] {
            Self::GPIO_PUP_PDN_CNTRL_REG3.write(state[3]);
        }
    }
}
