use core::{ops::{Deref, DerefMut}, sync::atomic};
pub struct Mutex<T> {
    is_locked: atomic::AtomicBool, 
    inner: core::cell::UnsafeCell<T>
}


impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            is_locked: atomic::AtomicBool::new(false),
            inner: core::cell::UnsafeCell::new(value)
        }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        match self.is_locked.compare_exchange(false, true, atomic::Ordering::Acquire, atomic::Ordering::Relaxed) {
            Ok(false) => Some(MutexGuard::with_locked_mutex(self)),
            Ok(true) => panic!("This should never happen"),
            Err(_) => None,
        }
    }

    pub fn unlock(guard: MutexGuard<'_, T>) {
        drop(guard)
    }

    fn unlock_internal(&self) {
        self.is_locked.store(false, atomic::Ordering::Release);
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked.load(atomic::Ordering::Relaxed)
    }
}

unsafe impl<T> Sync for Mutex<T> {}


pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T> 
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
        Self {
            mutex,
        }
    }
}
