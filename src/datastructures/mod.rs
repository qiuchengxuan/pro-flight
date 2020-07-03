pub mod input;
pub mod measurement;
pub mod ring_buffer;

use core::marker::PhantomData;
use core::sync::atomic::{AtomicU16, Ordering};

pub struct U16DataSource(AtomicU16);

impl Default for U16DataSource {
    fn default() -> Self {
        Self(AtomicU16::new(0))
    }
}

impl U16DataSource {
    pub fn write(&self, value: u16) {
        self.0.store(value, Ordering::Relaxed)
    }
}

#[derive(Copy, Clone)]
pub struct U16DataReader<'a, T> {
    source: &'a U16DataSource,
    t: PhantomData<T>,
}

impl<'a, T: From<u16>> U16DataReader<'a, T> {
    pub fn new(source: &'a U16DataSource) -> Self {
        Self { source, t: PhantomData }
    }

    pub fn read(&self) -> T {
        self.source.0.load(Ordering::Relaxed).into()
    }
}
