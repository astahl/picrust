use crate::{peripherals::mailbox, system};

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum Led {
    Status = 42,
    Power = 130,
}

impl Led {
    pub fn set(&self, on: bool) {
        match mailbox::simple_single_call::<[u32; 2], ()>(
            mailbox::Tag::SetOnboardLedStatus as u32,
            [*self as u32, if on { 1 } else { 0 }],
        ) {
            Ok(_) => (),
            Err(_) => (),
        }
    }

    pub fn get(&self) -> bool {
        match mailbox::simple_single_call::<u32, [u32; 2]>(
            mailbox::Tag::GetOnboardLedStatus as u32,
            *self as u32,
        ) {
            Ok([pin, status]) => status == 1,
            Err(_) => false,
        }
    }

    pub fn on(&self) {
        self.set(true);
    }

    pub fn off(&self) {
        self.set(false);
    }

    pub fn blink_pattern(&self, pattern: u8, step_duration: core::time::Duration) {
        for i in 0..8 {
            self.set((pattern << i & 0x80) != 0);
            system::arm_core::counter::spin_wait_for(step_duration);
        }
    }
}

pub fn status_blink_twice(interval_msec: u64) {
    let status = Led::Status;
    let duration = core::time::Duration::from_millis(interval_msec);
    let is_on = status.get();
    status.set(!is_on);
    system::arm_core::counter::spin_wait_for(duration);
    status.set(is_on);
    system::arm_core::counter::spin_wait_for(duration);
    status.set(!is_on);
    system::arm_core::counter::spin_wait_for(duration);
    status.set(is_on);
}
