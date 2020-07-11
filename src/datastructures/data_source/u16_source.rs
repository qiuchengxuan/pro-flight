use core::sync::atomic::{AtomicU32, Ordering};

use super::{DataSource, DataWriter};

pub struct U16Data(AtomicU32);

impl Default for U16Data {
    fn default() -> Self {
        Self(AtomicU32::new(0))
    }
}

impl DataWriter<u16> for U16Data {
    fn write(&self, value: u16) {
        let counter = self.0.load(Ordering::Relaxed) >> 16;
        self.0.store((counter + 1) << 16 | value as u32, Ordering::Relaxed)
    }
}

pub struct U16DataSource<'a> {
    data: &'a U16Data,
    counter: u16,
}

impl<'a> U16DataSource<'a> {
    pub fn new(data: &'a U16Data) -> Self {
        Self { data, counter: (data.0.load(Ordering::Relaxed) >> 16) as u16 }
    }
}

impl<'a, T: From<u16>> DataSource<T> for U16DataSource<'a> {
    fn capacity(&self) -> usize {
        1
    }

    fn read(&mut self) -> Option<T> {
        let value = self.data.0.load(Ordering::Relaxed);
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
        let value = self.data.0.load(Ordering::Relaxed);
        (value as u16).into()
    }
}
