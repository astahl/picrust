use mystd::bitfield::BitField;
use crate::peripherals::mailbox;

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum Clock {
    Reserved =  0x00,
    EMMC =      0x01,
    UART =      0x02,
    ARM =       0x03,
    Core =      0x04,
    V3D =       0x05,
    H264 =      0x06,
    ISP =       0x07,
    SDRAM =     0x08,
    Pixel =     0x09,
    PWM =       0x0a,
    HEVC =      0x0b,
    EMMC2 =     0x0c,
    M2MC =      0x0d,
    PixelBVB =  0x0e,
}


impl Clock {
    pub fn state(&self) -> Option<ClockState> {
        let (_, state): (Clock, ClockState) = mailbox::simple_single_call(tags::CLK_GET_CLOCK_STATE, *self as u32).ok()?;
        Some(state)
    }

    pub fn set_state(&self, state: ClockState) -> Option<ClockState> {
        let (_, state): (Clock, ClockState) = mailbox::simple_single_call(tags::CLK_SET_CLOCK_STATE, (*self as u32, state)).ok()?;
        Some(state)
    }

    pub fn rate(&self) -> Option<u32> {
        let (_, rate): (Clock, u32) = mailbox::simple_single_call(tags::CLK_GET_CLOCK_RATE, *self as u32).ok()?;
        Some(rate)
    }

    pub fn set_rate(&self, rate_hz: u32, skip_setting_turbo: bool) -> Option<u32> {
        let (_, rate): (Clock, u32) = mailbox::simple_single_call(tags::CLK_SET_CLOCK_RATE, (*self as u32, rate_hz, if skip_setting_turbo { 1_u32 } else { 0_u32 })).ok()?;
        Some(rate)
    }

    pub fn rate_measured(&self) -> Option<u32> {
        let (_, rate): (Clock, u32) = mailbox::simple_single_call(tags::CLK_GET_CLOCK_RATE_MEASURED, *self as u32).ok()?;
        Some(rate)
    }

    pub fn max_clock_rate(&self) -> Option<u32> {
        let (_, rate): (Clock, u32) = mailbox::simple_single_call(tags::CLK_GET_MAX_CLOCK_RATE, *self as u32).ok()?;
        Some(rate)
    }

    pub fn min_clock_rate(&self) -> Option<u32> {
        let (_, rate): (Clock, u32) = mailbox::simple_single_call(tags::CLK_GET_MIN_CLOCK_RATE, *self as u32).ok()?;
        Some(rate)
    }

    pub fn get_turbo(&self) -> Option<bool> {
        let (_, turbo_u32): (Clock, u32) = mailbox::simple_single_call(tags::CLK_GET_TURBO, *self as u32).ok()?;
        Some(turbo_u32 == 1)
    }

    pub fn set_turbo(&self, turbo: bool) -> Option<bool> {
        let (_, turbo_u32): (Clock, u32) = mailbox::simple_single_call(tags::CLK_SET_TURBO, (*self as u32, if turbo { 1_u32 } else { 0_u32 })).ok()?;
        Some(turbo_u32 == 1)
    }
}

mod tags {
    pub const CLK_GET_CLOCK_STATE: u32 = 0x00030001;
    pub const CLK_SET_CLOCK_STATE: u32 = 0x00038001;
    pub const CLK_GET_CLOCK_RATE: u32 = 0x00030002;
    pub const CLK_SET_CLOCK_RATE: u32 = 0x00038002;
    pub const CLK_GET_MAX_CLOCK_RATE: u32 = 0x00030004;
    pub const CLK_GET_MIN_CLOCK_RATE: u32 = 0x00030007;
    pub const CLK_GET_TURBO: u32 = 0x00030009;
    pub const CLK_SET_TURBO: u32 = 0x00038009;
    pub const CLK_GET_CLOCK_RATE_MEASURED: u32 = 0x00030047;
}

#[derive(Debug, Clone, Copy)]
pub struct ClockDescription {
    clock: Clock,
    available: bool,
    on: bool,
    rate_hz: u32,
    measured_rate_hz: u32,
    max_rate_hz: u32,
    min_rate_hz: u32,
    turbo: bool,
}

impl ClockDescription {
    pub fn get(clock: Clock) -> Option<ClockDescription> {
        use crate::peripherals::mailbox::*;
        use tags::*;
        let mut mailbox = Mailbox::<64>::new();
        let clock_id = clock as u32;
        *mailbox
            .push_request(CLK_GET_CLOCK_STATE, 8)
            .ok()? = clock_id;
        *mailbox
            .push_request(CLK_GET_CLOCK_RATE, 8)
            .ok()? = clock_id;
        *mailbox
            .push_request(CLK_GET_CLOCK_RATE_MEASURED, 8)
            .ok()? = clock_id;
        *mailbox
            .push_request(CLK_GET_MAX_CLOCK_RATE, 8)
            .ok()? = clock_id;
        *mailbox
            .push_request(CLK_GET_MIN_CLOCK_RATE, 8)
            .ok()? = clock_id;
        *mailbox
            .push_request(CLK_GET_TURBO, 8)
            .ok()? = clock_id;
        let mut responses = mailbox.submit_messages(8).ok()?;
        let (_, state): (u32, ClockState) = *responses.next()?.ok()?.try_value_as()?;
        let (_, rate_hz): (u32, u32) = *responses.next()?.ok()?.try_value_as()?;
        let (_, measured_rate_hz): (u32, u32) = *responses.next()?.ok()?.try_value_as()?;
        let (_, max_rate_hz): (u32, u32) = *responses.next()?.ok()?.try_value_as()?;
        let (_, min_rate_hz): (u32, u32) = *responses.next()?.ok()?.try_value_as()?;
        let (_, turbo_u32): (u32, u32) = *responses.next()?.ok()?.try_value_as()?;
        
        Some(Self {
            clock,
            available: state.exists(),
            on: state.is_on(),
            rate_hz,
            measured_rate_hz,
            max_rate_hz,
            min_rate_hz,
            turbo: turbo_u32 == 1
        })
    }
}

#[derive(Clone, Copy)]
pub struct ClockState (BitField<u32>);

impl ClockState {
    pub fn is_on(self) -> bool {
        self.0.bit_test(0)
    }

    pub fn with_on(self) -> Self {
        Self(self.0.with_bit_set(0))
    }

    pub fn with_off(self) -> Self {
        Self(self.0.with_bit_cleared(0))
    }

    pub fn exists(self) -> bool {
        !self.0.bit_test(1)
    }
}