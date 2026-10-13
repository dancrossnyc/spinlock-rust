//! MCS locks in Rust.

#![no_std]

use core::cell::UnsafeCell;
use core::marker::{Send, Sync};
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct Lock<T: ?Sized> {
    held: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> Lock<T> {
    /// Creates a new `Lock` that owns an object of type T.
    pub const fn new(value: T) -> Lock<T> {
        Lock {
            held: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    /// Consumes the lock and returns the contained object.
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<T: ?Sized> Lock<T> {
    /// Acquires the lock.  Returns a LockGuard that will
    /// release the lock when dropped.
    pub fn lock(&self) -> LockGuard<'_, T> {
        while self.held.load(Ordering::Relaxed) || self.held.swap(true, Ordering::AcqRel) {
            core::hint::spin_loop();
        }
        LockGuard {
            data: unsafe { &mut *self.data.get() },
            lock: self,
        }
    }

    /// Releases the lock, assuming it is held.  Private, so
    /// that the only way to unlock is through the `drop` method
    /// of a `LockGuard<T>` (that itself can only exist as the
    /// result of a successful call to `lock`), or by calling
    /// the unsafe `force_unlock` method.
    fn unlock(&self) {
        self.held.store(false, Ordering::Release);
    }

    /// Force-releases the lock.
    ///
    /// Suppose that a lock guard holding the lock is leaked
    /// (e.g., via `std::mem::forget()`); its `drop` method
    /// can never be called to unlock.  This function provides
    /// an unsafe escape hatch to release the lock.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the lock is currently locked,
    /// that the underlying object is in a state that it is
    /// safe to unlock, and that nothing else will try to unlock
    /// the current lock-holder later; leaked LockGuards must
    /// be forgotten: their `drop` methods must never be
    /// invoked, and their borrows to the lock-owned data must
    /// be gone.
    pub unsafe fn force_unlock(&self) {
        self.unlock();
    }
}

unsafe impl<T: ?Sized + Send> Send for Lock<T> {}
unsafe impl<T: ?Sized + Send> Sync for Lock<T> {}

pub struct LockGuard<'a, T: ?Sized> {
    lock: &'a Lock<T>,
    data: &'a mut T,
}

impl<T: ?Sized> Deref for LockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<T: ?Sized> DerefMut for LockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<T: ?Sized> Drop for LockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}
