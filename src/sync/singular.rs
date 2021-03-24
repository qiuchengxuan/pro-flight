use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU32, Ordering};

use super::{AgingDataReader, DataReader, DataWriter};

pub struct SingularData<T> {
    buffer: UnsafeCell<[T; 2]>,
    counter: AtomicU32,
}

impl<T: Default + Copy> Default for SingularData<T> {
    fn default() -> Self {
        Self { buffer: UnsafeCell::new([T::default(); 2]), counter: AtomicU32::new(0) }
    }
}

impl<T: Copy> DataWriter<T> for SingularData<T> {
    fn write(&self, data: T) {
        let counter = self.counter.load(Ordering::Relaxed);
        let buffer = unsafe { &mut *self.buffer.get() };
        buffer[counter as usize & 1] = data;
        self.counter.store((counter + 1) | 1 << 31, Ordering::Release);
    }
}

unsafe impl<T> Send for SingularData<T> {}
unsafe impl<T> Sync for SingularData<T> {} // FIXME: does not implement sync

#[derive(Copy, Clone)]
pub struct SingularDataSource<'a, T> {
    source: &'a SingularData<T>,
    counter: u32,
    age: usize,
}

impl<T: Copy> SingularData<T> {
    pub fn reader(&self) -> SingularDataSource<T> {
        SingularDataSource { source: self, counter: 0, age: 0 }
    }
}

impl<'a, T: Copy> DataReader<T> for SingularDataSource<'a, T> {
    fn get(&mut self) -> Option<T> {
        let buffer = unsafe { &*self.source.buffer.get() };
        loop {
            let counter = self.source.counter.load(Ordering::Acquire);
            if self.counter == counter {
                return None;
            }
            let result = buffer[(counter as usize & 1) ^ 1];
            if counter == self.source.counter.load(Ordering::Relaxed) {
                return Some(result);
            }
        }
    }

    fn get_last(&mut self) -> Option<T> {
        let buffer = unsafe { &*self.source.buffer.get() };
        loop {
            let counter = self.source.counter.load(Ordering::Acquire);
            if counter == 0 {
                return None;
            }
            let result = buffer[(counter as usize & 1) ^ 1];
            if counter == self.source.counter.load(Ordering::Relaxed) {
                return Some(result);
            }
        }
    }
}

impl<'a, T: Copy> AgingDataReader<T> for SingularDataSource<'a, T> {
    fn get_aging_last(&mut self, max_age: usize) -> Option<T> {
        let counter = self.source.counter.load(Ordering::Acquire);
        match counter {
            0 => return None,
            counter if counter == self.counter => {
                if self.age >= max_age {
                    return None;
                }
                self.age = core::cmp::min(self.age + 1, max_age);
            }
            _ => self.age = 0,
        }
        self.counter = counter;
        let buffer = unsafe { &*self.source.buffer.get() };
        return Some(buffer[(counter as usize & 1) ^ 1]);
    }
}
