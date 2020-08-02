use alloc::rc::Rc;

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use super::{DataSource, DataWriter};

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
        let counter = self.counter.fetch_add(1, Ordering::Relaxed);
        let buffer = unsafe { &mut *self.buffer.get() };
        buffer[counter & 1] = data;
    }
}

pub struct SingularDataSource<T> {
    source: Rc<SingularData<T>>,
    counter: usize,
}

impl<T: Copy> SingularDataSource<T> {
    pub fn new(data: &Rc<SingularData<T>>) -> Self {
        Self { source: Rc::clone(data), counter: 0 }
    }
}

impl<T: Copy> DataSource<T> for SingularDataSource<T> {
    fn capacity(&self) -> usize {
        1
    }

    fn read(&mut self) -> Option<T> {
        let counter = self.source.counter.load(Ordering::Relaxed);
        if self.counter == counter {
            return None;
        }
        self.counter = counter;
        let buffer = unsafe { &*self.source.buffer.get() };
        Some(buffer[counter & 1])
    }

    fn read_last(&mut self) -> Option<T> {
        self.read()
    }

    fn read_last_unchecked(&self) -> T {
        let counter = self.source.counter.load(Ordering::Relaxed);
        let buffer = unsafe { &*self.source.buffer.get() };
        buffer[counter & 1]
    }
}
