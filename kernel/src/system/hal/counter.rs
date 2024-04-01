use core::{ops::{Add, Sub}, time::Duration};

use crate::system::arm_core::registers::aarch64::generic_timer::{self, cntp_cval_el0::CntPCValEl0, cntp_tval_el0::CntPTValEl0};

#[derive(Clone, Copy)]
pub struct PointInTime {
    counter_val: u64,
    frequency: u32
}

impl core::cmp::Eq for PointInTime {}

impl core::cmp::PartialEq for PointInTime {
    fn eq(&self, other: &Self) -> bool {
        assert_eq!(self.frequency, other.frequency);
        self.counter_val == other.counter_val
    }
}

impl core::cmp::PartialOrd for PointInTime {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        if self.frequency == other.frequency {
            self.counter_val.partial_cmp(&other.counter_val)
        } else {
            None
        }
    }
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
        generic_timer::cntp_cval_el0::CntPCValEl0::zero().compare_value().set_value(self.counter_val).write_register()
    }

    pub fn time_since_zero(self) -> core::time::Duration {
        let nanos = (self.counter_val * 1_000_000_000) / self.frequency as u64;
        core::time::Duration::from_nanos(nanos)
    }
}

pub fn enable_interrupt() {
    generic_timer::cntp_ctl_el0::CntPCtlEl0::read_register().enable().set().write_register();
}

pub fn is_interrupt_enabled() -> bool {
    generic_timer::cntp_ctl_el0::CntPCtlEl0::read_register().enable().is_set()
}

pub fn disable_interrupt() {
    generic_timer::cntp_ctl_el0::CntPCtlEl0::read_register().enable().clear().write_register();
}

pub fn mask_interrupt() {
    generic_timer::cntp_ctl_el0::CntPCtlEl0::read_register().imask().set().write_register();
}

pub fn is_interrupt_masked() -> bool {
    generic_timer::cntp_ctl_el0::CntPCtlEl0::read_register().imask().is_set()
}

pub fn unmask_interrupt() {
    generic_timer::cntp_ctl_el0::CntPCtlEl0::read_register().imask().clear().write_register();
}

pub fn is_timer_condition_met() -> bool {
    generic_timer::cntp_ctl_el0::CntPCtlEl0::read_register().istatus().is_set()
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

pub fn uptime() -> core::time::Duration {
    PointInTime::now().time_since_zero()
}