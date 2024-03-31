pub mod mutex;
pub mod signal;
pub mod latch;

#[derive(Debug)]
pub enum SleepError {
    TooManySleepers,
}


#[derive(Debug)]
pub enum WakeError{
    NoOneSleeping
}

pub trait SleepHandler {
    fn sleep(&self) -> Result<(), SleepError>;
    fn wake(&self) -> Result<(), WakeError>;
}
