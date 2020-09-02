use alloc::rc::Rc;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};

use super::{AgingStaticData, DataWriter, OptionData, StaticData};

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
    age: u16,
}

impl<T> U16DataSource<T> {
    pub fn new(data: &Rc<U16Data<T>>) -> Self {
        Self { data: Rc::clone(data), counter: 0, age: 0 }
    }
}

impl<T: From<u16>> StaticData<T> for U16DataSource<T> {
    fn read(&mut self) -> T {
        (self.data.value.load(Ordering::Relaxed) as u16).into()
    }
}

impl<T: From<u16>> AgingStaticData<T> for U16DataSource<T> {
    fn read(&mut self, max_age: usize) -> Option<T> {
        let raw = self.data.value.load(Ordering::Relaxed);
        let (counter, value) = ((raw >> 16) as u16, raw as u16);
        if self.counter == counter && max_age > 0 {
            if self.age as usize >= max_age {
                return None;
            }
            self.age += 1;
        } else {
            self.age = 0;
        }
        self.counter = counter;
        return Some(value.into());
    }
}

impl<T: From<u16>> OptionData<T> for U16DataSource<T> {
    fn read(&mut self) -> Option<T> {
        let value = self.data.value.load(Ordering::Relaxed);
        let written = (value >> 16) as u16;
        if self.counter == written {
            return None;
        }
        self.counter = written;
        Some((value as u16).into())
    }
}
