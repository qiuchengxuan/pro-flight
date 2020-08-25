use alloc::rc::Rc;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};

use super::{DataSource, DataWriter};

pub struct U16Data<T> {
    value: AtomicU32,
    t: PhantomData<T>,
}

impl<T: Default> Default for U16Data<T> {
    fn default() -> Self {
        Self { value: AtomicU32::new(0), t: PhantomData }
    }
}

impl<T: Into<u16>> DataWriter<T> for U16Data<T> {
    fn write(&self, value: T) {
        let counter = self.value.load(Ordering::Relaxed) >> 16;
        self.value.store((counter + 1) << 16 | value.into() as u32, Ordering::Relaxed)
    }
}

pub struct U16DataSource<T> {
    data: Rc<U16Data<T>>,
    counter: u16,
}

impl<T> U16DataSource<T> {
    pub fn new(data: &Rc<U16Data<T>>) -> Self {
        Self { data: Rc::clone(data), counter: (data.value.load(Ordering::Relaxed) >> 16) as u16 }
    }
}

impl<T: From<u16>> DataSource<T> for U16DataSource<T> {
    fn capacity(&self) -> usize {
        1
    }

    fn read(&mut self) -> Option<T> {
        let value = self.data.value.load(Ordering::Relaxed);
        if (value >> 16) as u16 == self.counter {
            None
        } else {
            self.counter += 1;
            Some((value as u16).into())
        }
    }

    fn read_last(&mut self) -> Option<T> {
        self.read()
    }

    fn read_last_unchecked(&self) -> T {
        let value = self.data.value.load(Ordering::Relaxed);
        (value as u16).into()
    }
}
