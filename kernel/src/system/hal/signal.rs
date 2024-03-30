use mystd::sync::signal;


pub struct EventSleepHandler {}

impl signal::SleepHandler for EventSleepHandler {
    fn sleep(&self) -> Result<(), signal::SleepError> {
        crate::system::arm_core::wait_for_event();
        Ok(())
    }

    fn wake(&self) -> Result<(), signal::WakeError> {
        crate::system::arm_core::send_event();
        Ok(())
    }
}

pub type EventSignal = signal::Signal<EventSleepHandler>;

pub const fn new_signal() -> EventSignal {
    EventSignal::new(EventSleepHandler {  })
}