use mystd::sync::signal;


pub struct EventSleepHandler {}

impl mystd::sync::SleepHandler for EventSleepHandler {
    fn sleep(&self) -> Result<(), mystd::sync::SleepError> {
        crate::system::arm_core::wait_for_event();
        Ok(())
    }

    fn wake(&self) -> Result<(), mystd::sync::WakeError> {
        crate::system::arm_core::send_event();
        Ok(())
    }
}

pub type EventSignal = signal::Signal<EventSleepHandler>;

pub const fn new_signal() -> EventSignal {
    EventSignal::new(EventSleepHandler {})
}

pub type EventLatch = mystd::sync::latch::Latch<EventSleepHandler>;

pub const fn new_latch(auto_reset: bool) -> EventLatch {
    EventLatch::new(auto_reset, EventSleepHandler {})
}