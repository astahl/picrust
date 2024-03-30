use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{self, AtomicU32},
};

pub struct Mutex<T> {
    is_locked: atomic::AtomicBool,
    inner: core::cell::UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            is_locked: atomic::AtomicBool::new(false),
            inner: core::cell::UnsafeCell::new(value),
        }
    }

    /// Tries to acquire the lock and returns None if it fails.
    /// ### Panic
    /// Panics when the atomic compare_exchange contract is violated, which really shouldn't happen.
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        match self.is_locked.compare_exchange(
            false,
            true,
            atomic::Ordering::Acquire,
            atomic::Ordering::Relaxed,
        ) {
            Ok(false) => Some(MutexGuard::with_locked_mutex(self)),
            Ok(true) => panic!("This should never happen"),
            Err(_) => None,
        }
    }

    /// Blocks in a busy wait until the lock can be acquired.
    /// ### Safety
    /// Unsafe to call while holding the lock, leading to a deadlock,
    /// because it doesn't protect against reentrancy.
    /// To fail in a controlled manner use `try_lock(&self)`
    pub unsafe fn lock(&self) -> MutexGuard<T> {
        loop {
            match self.try_lock() {
                Some(guard) => break guard,
                None => core::hint::spin_loop(),
            }
        }
    }

    // pub fn unlock(guard: MutexGuard<'_, T>) {
    //     drop(guard)
    // }

    fn unlock_internal(&self) {
        self.is_locked.store(false, atomic::Ordering::Release);
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked.load(atomic::Ordering::Relaxed)
    }
}

unsafe impl<T> Sync for Mutex<T> {}

impl<T> From<T> for Mutex<T> {
    fn from(value: T) -> Self {
        Mutex::new(value)
    }
}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.inner.get() }
    }
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.inner.get() }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.unlock_internal()
    }
}

impl<'a, T> MutexGuard<'a, T> {
    fn with_locked_mutex(mutex: &'a Mutex<T>) -> Self {
        assert!(mutex.is_locked());
        Self { mutex }
    }
}

pub struct ReadWriteMutex<T> {
    // 2^32 - 1 concurrent readers should be enough...
    counter: AtomicU32,
    inner: UnsafeCell<T>,
}

impl<T> ReadWriteMutex<T> {
    const READ_BIT: u32 = 0b10;
    const WRITE_BIT: u32 = 0b1;
    const READ_MASK: u32 = !Self::WRITE_BIT;

    pub const fn new(value: T) -> Self {
        Self {
            counter: atomic::AtomicU32::new(0),
            inner: core::cell::UnsafeCell::new(value),
        }
    }

    /// Tries to acquire a shared lock for reading and returns None if it fails.
    pub fn try_lock_read(&self) -> Option<ReadMutexGuard<T>> {
        match self.counter.fetch_update(
            atomic::Ordering::SeqCst,
            atomic::Ordering::SeqCst,
            |count| {
                if count & Self::WRITE_BIT != 0 {
                    None
                } else {
                    count.checked_add(Self::READ_BIT)
                }
            },
        ) {
            Ok(_) => Some(ReadMutexGuard::with_locked_mutex(self)),
            Err(_) => None,
        }
    }

    /// Tries to acquire an exclusive lock for writing and returns None if it fails.
    pub fn try_lock_write(&self) -> Option<WriteMutexGuard<T>> {
        match self.counter.fetch_update(
            atomic::Ordering::SeqCst,
            atomic::Ordering::SeqCst,
            |count| {
                if count != 0 {
                    None
                } else {
                    Some(Self::WRITE_BIT)
                }
            },
        ) {
            Ok(_) => Some(WriteMutexGuard::with_locked_mutex(self)),
            Err(_) => None,
        }
    }

    /// Blocks in a busy wait until the read lock can be acquired.
    /// ### Safety
    /// Unsafe to call while holding the write lock, leading to a deadlock,
    /// because it doesn't protect against reentrancy.
    /// To fail in a controlled manner use `try_lock(&self)`
    pub unsafe fn lock_read(&self) -> ReadMutexGuard<T> {
        loop {
            match self.try_lock_read() {
                Some(guard) => break guard,
                None => core::hint::spin_loop(),
            }
        }
    }

    /// Blocks in a busy wait until the write lock can be acquired.
    /// ### Safety
    /// Unsafe to call while holding the write lock, leading to a deadlock,
    /// because it doesn't protect against reentrancy.
    /// To fail in a controlled manner use `try_lock(&self)`
    pub unsafe fn lock_write(&self) -> WriteMutexGuard<T> {
        loop {
            match self.try_lock_write() {
                Some(guard) => break guard,
                None => core::hint::spin_loop(),
            }
        }
    }

    fn unlock_read_internal(&self) {
        self.counter
            .fetch_update(
                atomic::Ordering::SeqCst,
                atomic::Ordering::SeqCst,
                |count| {
                    assert_eq!(
                        0,
                        count & Self::WRITE_BIT,
                        "Can't read-unlock a held write lock"
                    );
                    assert_ne!(
                        0,
                        count & Self::READ_MASK,
                        "Can't read-unlock if no read lock is held"
                    );
                    Some(count - Self::READ_BIT)
                },
            )
            .expect("All Error cases should be handled by the internal asserts");
    }

    fn unlock_write_internal(&self) {
        self.counter
            .fetch_update(
                atomic::Ordering::SeqCst,
                atomic::Ordering::SeqCst,
                |count| {
                    assert_eq!(
                        0,
                        count & Self::READ_MASK,
                        "Can't write-unlock if any read lock is held"
                    );
                    assert_ne!(
                        0,
                        count & Self::WRITE_BIT,
                        "Can't write-unlock if no write lock is held"
                    );
                    Some(0)
                },
            )
            .expect("All Error cases should be handled by the internal asserts");
    }

    fn reader_count(&self) -> u32 {
        self.counter.load(atomic::Ordering::SeqCst) >> 1
    }

    fn is_locked_read(&self) -> bool {
        self.counter.load(atomic::Ordering::SeqCst) & Self::READ_MASK != 0
    }

    fn is_locked_write(&self) -> bool {
        self.counter.load(atomic::Ordering::SeqCst) & Self::WRITE_BIT != 0
    }
}

unsafe impl<T> Sync for ReadWriteMutex<T> {}

impl<T> From<T> for ReadWriteMutex<T> {
    fn from(value: T) -> Self {
        ReadWriteMutex::new(value)
    }
}

pub struct ReadMutexGuard<'a, T> {
    mutex: &'a ReadWriteMutex<T>,
}

impl<'a, T> Deref for ReadMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.inner.get() }
    }
}

impl<'a, T> Drop for ReadMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.unlock_read_internal()
    }
}

impl<'a, T> ReadMutexGuard<'a, T> {
    fn with_locked_mutex(mutex: &'a ReadWriteMutex<T>) -> Self {
        assert!(mutex.is_locked_read());
        Self { mutex }
    }
}

pub struct WriteMutexGuard<'a, T> {
    mutex: &'a ReadWriteMutex<T>,
}

impl<'a, T> DerefMut for WriteMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.inner.get() }
    }
}

impl<'a, T> Deref for WriteMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.inner.get() }
    }
}

impl<'a, T> Drop for WriteMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.unlock_write_internal()
    }
}

impl<'a, T> WriteMutexGuard<'a, T> {
    fn with_locked_mutex(mutex: &'a ReadWriteMutex<T>) -> Self {
        assert!(mutex.is_locked_write());
        Self { mutex }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rw_mutex_works() {
        let mutex: ReadWriteMutex<u32> = 69.into();
        assert!(!mutex.is_locked_read());
        assert!(!mutex.is_locked_write());
        {
            let _hold_read1 = mutex.try_lock_read().expect("Should allow first read lock");
            let _hold_read2 = mutex
                .try_lock_read()
                .expect("Should allow a second read lock");
            assert!(
                mutex.try_lock_write().is_none(),
                "Should block write lock while read locks are held"
            );
            assert_eq!(2, mutex.reader_count());
        }
        assert!(!mutex.is_locked_read());
        assert!(!mutex.is_locked_write());
        {
            let _hold_write = mutex.try_lock_write().expect("Should be write lockable");
            assert!(
                mutex.try_lock_write().is_none(),
                "Should block write lock while write lock is held"
            );
            assert!(
                mutex.try_lock_read().is_none(),
                "Should block read lock while write lock is held"
            );
        }
        assert!(!mutex.is_locked_read());
        assert!(!mutex.is_locked_write());
    }
}
