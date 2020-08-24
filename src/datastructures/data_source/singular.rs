use alloc::rc::Rc;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use super::{AgingStaticData, DataWriter, OptionData, StaticData};

pub struct SingularData<T> {
    buffer: UnsafeCell<[T; 2]>,
    counter: AtomicUsize,
}

impl<T: Default + Copy> Default for SingularData<T> {
    fn default() -> Self {
        Self { buffer: UnsafeCell::new([T::default(); 2]), counter: AtomicUsize::new(0) }
    }
}

impl<T: Copy> DataWriter<T> for SingularData<T> {
    fn write(&self, data: T) {
        let counter = self.counter.load(Ordering::Relaxed);
        let buffer = unsafe { &mut *self.buffer.get() };
        buffer[counter & 1] = data;
        self.counter.fetch_add(1, Ordering::Relaxed);
    }
}

pub struct SingularDataSource<T> {
    source: Rc<SingularData<T>>,
    counter: usize,
    age: usize,
}

impl<T: Copy> SingularDataSource<T> {
    pub fn new(data: &Rc<SingularData<T>>) -> Self {
        Self { source: Rc::clone(data), counter: 0, age: 0 }
    }
}

impl<T: Copy> StaticData<T> for SingularDataSource<T> {
    fn read(&mut self) -> T {
        let buffer = unsafe { &*self.source.buffer.get() };
        let counter = self.source.counter.load(Ordering::Relaxed);
        buffer[(counter & 1) ^ 1]
    }
}

impl<T: Copy> AgingStaticData<T> for SingularDataSource<T> {
    fn read(&mut self, max_age: usize) -> Option<T> {
        let counter = self.source.counter.load(Ordering::Relaxed);
        if self.counter == counter && max_age > 0 {
            if self.age >= max_age {
                return None;
            }
            self.age += 1;
        } else {
            self.age = 0;
        }
        self.counter = counter;
        let buffer = unsafe { &*self.source.buffer.get() };
        return Some(buffer[(counter & 1) ^ 1]);
    }
}

impl<T: Copy> OptionData<T> for SingularDataSource<T> {
    fn read(&mut self) -> Option<T> {
        let counter = self.source.counter.load(Ordering::Relaxed);
        if self.counter == counter {
            return None;
        }
        self.counter = counter;
        let buffer = unsafe { &*self.source.buffer.get() };
        let counter = self.source.counter.load(Ordering::Relaxed);
        Some(buffer[(counter & 1) ^ 1])
    }
}
