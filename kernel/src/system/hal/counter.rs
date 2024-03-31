use core::{ops::{Add, Sub}, time::Duration};

use crate::system::arm_core::registers::aarch64::generic_timer::{self, cntp_cval_el0::CntPCValEl0, cntp_tval_el0::CntPTValEl0};

#[derive(Clone, Copy)]
pub struct PointInTime {
    counter_val: u64,
    frequency: u32
}

impl Add<core::time::Duration> for PointInTime {
    type Output = Self;

    fn add(self, rhs: core::time::Duration) -> Self::Output {
        Self {
            counter_val: self.counter_val + ((rhs.as_nanos() * self.frequency as u128) / 1_000_000_000) as u64,
            frequency: self.frequency
        }
    }
}

impl Sub for PointInTime {
    type Output = core::time::Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        assert_eq!(rhs.frequency, self.frequency);
        let nanos = ((self.counter_val - rhs.counter_val) * 1_000_000_000) / self.frequency as u64;
        core::time::Duration::from_nanos(nanos)
    }
}

impl PointInTime {
    pub fn now_ordered() -> Self {
        Self { 
            counter_val: generic_timer::cntpct_el0::CntPCtEl0::read_register_ordered().to_underlying(), 
            frequency: generic_timer::cntfrq_el0::CntFrqEl0::read_register().to_underlying() as u32
        }
    }

    pub fn now() -> Self {
        Self { 
            counter_val: generic_timer::cntpct_el0::CntPCtEl0::read_register().to_underlying(), 
            frequency: generic_timer::cntfrq_el0::CntFrqEl0::read_register().to_underlying() as u32
        }
    }

    pub fn elapsed(self) -> core::time::Duration {
        let now = Self::now();
        now - self
    }

    pub fn elapsed_ordered(self) -> core::time::Duration {
        let now = Self::now_ordered();
        now - self
    }

    pub fn is_in_the_future(self) -> bool {
        self.counter_val > generic_timer::cntpct_el0::CntPCtEl0::read_register().to_underlying()
    }

    pub fn is_in_the_future_ordered(self) -> bool {
        self.counter_val > generic_timer::cntpct_el0::CntPCtEl0::read_register_ordered().to_underlying()
    }

    pub fn set_as_compare_val(self) {
        generic_timer::cntp_cval_el0::CntPCValEl0::write_register(CntPCValEl0::new(self.counter_val))
    }
}

pub fn set_timer_val(duration: core::time::Duration) {
    let frequency = generic_timer::cntfrq_el0::CntFrqEl0::read_register().clock_frequency().value();
    let timer_val = frequency as u128 * duration.as_nanos() / 1_000_000_000;
    assert!(timer_val <= u32::MAX as u128);
    generic_timer::cntp_tval_el0::CntPTValEl0::write_register(CntPTValEl0::new(timer_val as u64));
}


/// Returns the counter frequency in Hz
pub fn frequency() -> u32 {
    generic_timer::cntfrq_el0::CntFrqEl0::read_register().clock_frequency().value() as u32
}

