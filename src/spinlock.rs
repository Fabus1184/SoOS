use core::sync::atomic::{AtomicBool, Ordering};

use log::warn;

#[derive(Debug)]
pub struct Spinlock<T> {
    lock: AtomicBool,
    inner: T,
}

impl<T> Spinlock<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            inner,
        }
    }

    pub fn with_locked<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.try_lock().map(f).and_then(|r| {
            self.unlock();
            Some(r)
        })
    }

    pub unsafe fn as_mut_unchecked(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn lock_spin(&mut self) -> &mut T {
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            warn!("Spinlock failed!");
            core::hint::spin_loop();
        }

        &mut self.inner
    }

    pub fn unlock(&mut self) {
        self.lock.store(false, Ordering::Release);
    }

    pub fn try_lock(&mut self) -> Option<&mut T> {
        self.lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| &mut self.inner)
    }
}
