pub mod event;

use core::{
    ptr,
    sync::atomic::{AtomicBool, AtomicU8, Ordering},
};

pub struct ReadSpinLock<T> {
    write_lock: AtomicBool,
    version: AtomicU8,
    data: T,
}

impl<T: Default> Default for ReadSpinLock<T> {
    fn default() -> Self {
        Self { write_lock: Default::default(), version: Default::default(), data: T::default() }
    }
}

impl<T: Copy> ReadSpinLock<T> {
    pub fn write(&self, data: T) {
        let relaxed = Ordering::Relaxed;
        self.write_lock.compare_exchange_weak(false, true, relaxed, relaxed).unwrap();
        unsafe { ptr::write(ptr::addr_of!(self.data) as *mut T, data) };
        self.version.fetch_add(1, Ordering::Release);
        self.write_lock.store(false, Ordering::Relaxed);
    }

    pub fn read(&self) -> T {
        loop {
            let version = self.version.load(Ordering::Acquire);
            let data = self.data;
            if !self.write_lock.load(Ordering::Relaxed)
                && version == self.version.load(Ordering::Relaxed)
            {
                return data;
            }
        }
    }
}

unsafe impl<T: Sync> Sync for ReadSpinLock<T> {}
unsafe impl<T: Send> Send for ReadSpinLock<T> {}
