use mystd::bitfield::BitField;

use super::mailbox;

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum PowerDevice {
    SdCard = 0x00000000,
    Uart0 = 0x00000001,
    Uart1 = 0x00000002,
    UsbHcd = 0x00000003,
    I2C0 = 0x00000004,
    I2C1 = 0x00000005,
    I2C2 = 0x00000006,
    Spi = 0x00000007,
    Ccp2Tx = 0x00000008,
    Unknown0RPi4 = 0x00000009,
    Unknown1RPi4 = 0x0000000a,
}

mod tags {
    pub const PWR_GET_STATE: u32 = 0x00020001;
    pub const PWR_SET_STATE: u32 = 0x00028001;
    pub const PWR_GET_TIMING: u32 = 0x00020002;
}

impl PowerDevice {
    pub fn state(&self) -> Option<PowerState> {
        let (_, state): (PowerDevice, PowerState) =
            mailbox::simple_single_call(tags::PWR_GET_STATE, *self as u32).ok()?;
        Some(state)
    }

    pub fn set_state(&self, state: PowerState) -> Option<PowerState> {
        let (_, state): (PowerDevice, PowerState) =
            mailbox::simple_single_call(tags::PWR_SET_STATE, (*self as u32, state)).ok()?;
        Some(state)
    }

    pub fn timing_ms(&self) -> Option<u32> {
        let (_, state): (PowerDevice, u32) =
            mailbox::simple_single_call(tags::PWR_GET_TIMING, *self as u32).ok()?;
        Some(state)
    }
}

#[derive(Clone, Copy)]
pub struct PowerState(BitField<u32>);

impl PowerState {
    pub fn is_on(self) -> bool {
        self.0.bit_test(0)
    }

    pub fn with_on(self) -> Self {
        Self(self.0.with_bit_set(0))
    }

    pub fn with_off(self) -> Self {
        Self(self.0.with_bit_cleared(0))
    }

    pub fn with_wait_set(self) -> Self {
        Self(self.0.with_bit_set(1))
    }

    pub fn with_wait_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(1))
    }

    pub fn wait(self) -> bool {
        self.0.bit_test(1)
    }

    pub fn exists(self) -> bool {
        !self.0.bit_test(1)
    }
}
