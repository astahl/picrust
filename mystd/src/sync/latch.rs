use core::sync::atomic;

use super::{SleepError, SleepHandler, WakeError};


pub struct Latch<S: SleepHandler>  {
    auto_reset: bool,
    latch: atomic::AtomicBool,
    sleep_handler: S,
}

unsafe impl<S: SleepHandler> Sync for Latch<S> {}

impl<S: SleepHandler> Latch<S> {
    pub const fn new(auto_reset: bool, sleep_handler: S) -> Self {
        Self { auto_reset, latch: atomic::AtomicBool::new(false), sleep_handler }
    }

    /// Causes the calling thread to engage with the latch, using the supplied sleep handler to wait for the latch to be set.
    pub fn wait(&self) -> Result<(), SleepError> {
        // r l => l
        // t t => f
        // f t => t
        // t f => f
        // f t => t
        let _ = self.latch.compare_exchange(self.auto_reset, false, atomic::Ordering::SeqCst, atomic::Ordering::SeqCst);
        loop {
            if self.is_set() {
                break Ok(());
            }
            self.sleep_handler.sleep()?;
        }
    }

    pub fn set(&self) -> Result<bool, WakeError> {
        let prev_value = self.latch.swap(true, atomic::Ordering::SeqCst);
        if !prev_value {
            self.sleep_handler.wake()?;
        }
        Ok(prev_value)
    }

    pub fn reset(&self) -> bool {
        self.latch.swap(false, atomic::Ordering::SeqCst)
    }

    pub fn is_set(&self) -> bool {
        self.latch.load(atomic::Ordering::SeqCst)
    }
}

