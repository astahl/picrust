use core::sync::atomic;

use super::{SleepError, SleepHandler, WakeError};


pub trait TicketCounter {
    type TicketType;

    /// Generates a new ticket. Return none if a ticket could not be generated (e.g. a full queue)
    fn pull_ticket(&self) -> Option<Self::TicketType>;

    /// Test if a ticket has been served. Since serving and testing are two different operations, strict
    /// guarantees about serving a particular ticket can't be given because of ABA problems. In other words: the ticket 
    /// that was the next "up" in the queue during the call to `serve_one` might not be the one that tests as "up" subsequently, 
    /// if another thread has entered the queue in the mean time.
    fn is_ticket_up(&self, ticket: &Self::TicketType) -> bool;

    /// Mark a single ticket as served, i.e. released from the queue. Return the number of remaining tickets, or None if there was no ticket in the queue before the call.
    fn serve_one(&self) -> Option<usize>;

    /// Mark all tickets as served, i.e. clear the queue. Return the number of tickets cleared in the process.
    fn serve_all(&self) -> usize;
}

pub struct LifoTicketCounter {
    counter: atomic::AtomicU32,
}

impl LifoTicketCounter {
    pub const fn new() -> Self {
        Self { counter: atomic::AtomicU32::new(0) }
    }
}

impl TicketCounter for LifoTicketCounter {
    type TicketType = u32;

    fn pull_ticket(&self) -> Option<Self::TicketType> {
        let num = self.counter.fetch_update(atomic::Ordering::SeqCst, atomic::Ordering::SeqCst, |num| {
            if num == u32::MAX {
                None
            } else {
                Some(num + 1)
            }
        });
        num.ok()
    }

    fn is_ticket_up(&self, ticket: &Self::TicketType) -> bool {
        self.counter.load(atomic::Ordering::SeqCst) <= *ticket
    }

    fn serve_one(&self) -> Option<usize> {
        self.counter.fetch_update(atomic::Ordering::SeqCst, atomic::Ordering::SeqCst,
            |count| {
                if count == 0 {
                    None
                } else {
                    Some (count - 1)
                }
            }).map(|n| n as usize).ok()
    }

    fn serve_all(&self) -> usize {
        self.counter.swap(0, atomic::Ordering::SeqCst) as usize
    }
}

pub struct BusyWaitSleepHandler {
    flag: atomic::AtomicI32
}

impl SleepHandler for BusyWaitSleepHandler {
    fn sleep(&self) -> Result<(), SleepError> {
        todo!("clean this mess up, if we ever need it");
        self.flag.fetch_update(atomic::Ordering::SeqCst, atomic::Ordering::SeqCst, |f| {
            if f.is_negative() {
                None
            } else {
                Some(f + 1)
            }
        });
        loop {
            if self.flag.load(atomic::Ordering::SeqCst).is_negative() {
                self.flag.fetch_add(1, atomic::Ordering::SeqCst);
                break;
            }
            core::hint::spin_loop();
        }
        Ok(())
    }

    fn wake(&self) -> Result<(), WakeError> {
        self.flag.fetch_update(atomic::Ordering::SeqCst, atomic::Ordering::SeqCst, |f| {
            if f.is_negative() {
                Some(f)
            } else {
                Some(-f)
            }
        });
        Ok(())
    }
}


pub struct Signal<S: SleepHandler>  {
    ticket_counter: LifoTicketCounter,
    sleep_handler: S,
}

unsafe impl<S: SleepHandler> Sync for Signal<S> {}

impl<S: SleepHandler> Signal<S> {
    pub const fn new(sleep_handler: S) -> Self {
        Self { ticket_counter: LifoTicketCounter::new(), sleep_handler }
    }

    /// Causes the calling thread to engage with the signal, using the supplied sleep handler to wait for the signal to be signaled.
    pub fn wait(&self) -> Result<(), SleepError> {
        match self.ticket_counter.pull_ticket() {
            None => Err(SleepError::TooManySleepers),
            Some(ticket) => {
                loop {
                    if self.ticket_counter.is_ticket_up(&ticket) {
                        break Ok(());
                    }
                    self.sleep_handler.sleep()?;
                }
            }
        }
    }

    /// Wakes all waiting threads, returning the number of woken threads. The order of threads being awoken is indeterminate. 
    /// If you need LIFO behavior, consider calling `wake_one(&self)` repeatedly until it returns an error.
    /// 
    /// Returns an `Err(WakeError::NoOneSleeping)` when no thread is actually waiting on this signal.
    pub fn wake_all(&self) -> Result<usize, WakeError> {
        match self.ticket_counter.serve_all() {
            0 => Err(WakeError::NoOneSleeping),
            sleeper_count => self.sleep_handler.wake().map(|_| sleeper_count)
        }
    }


    /// Wakes exactly one waiting thread, returning the number of waiting threads remaining.
    /// 
    /// This wakes the chronologically last thread that engaged the signal.
    /// 
    /// If no thread is waiting, this returns `Err(WakeError::NoOneSleeping)`.
    pub fn wake_one(&self) -> Result<usize, WakeError> {
        match self.ticket_counter.serve_one() {
            None => Err(WakeError::NoOneSleeping),
            Some(remaining) => { 
                self.sleep_handler.wake()?; 
                Ok(remaining) 
            },
        }
        
    }
}


