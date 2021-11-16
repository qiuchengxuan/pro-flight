use alloc::vec::Vec;
use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicU32, Ordering},
};

use super::{DataReader, DataWriter, WithCapacity};

const VALID: u32 = 1 << 31;
const COUNTER_MASK: u32 = !VALID;

pub struct OverwritingData<T> {
    buffer: UnsafeCell<Vec<T>>,
    meta: AtomicU32,
}

impl<T: Copy + Clone> OverwritingData<T> {
    pub fn new(vec: Vec<T>) -> Self {
        Self { buffer: UnsafeCell::new(vec), meta: AtomicU32::new(0) }
    }
}

impl<T: Copy + Clone + Default> OverwritingData<T> {
    pub fn sized(size: usize) -> Self {
        Self::new(vec![T::default(); size])
    }
}

impl<T> DataWriter<T> for OverwritingData<T> {
    fn write(&self, value: T) {
        let buffer = unsafe { &mut *self.buffer.get() };
        let size = buffer.len();
        if size > 0 {
            let meta = self.meta.load(Ordering::Relaxed);
            let counter = meta & COUNTER_MASK;
            buffer[counter as usize % size] = value;
            self.meta.store((counter + 1) | VALID, Ordering::Release);
        }
    }
}

pub struct OverwritingDataSource<'a, T> {
    ring: &'a OverwritingData<T>,
    index: u32,
}

impl<T> OverwritingData<T> {
    pub fn reader(&self) -> OverwritingDataSource<T> {
        let index = self.meta.load(Ordering::Relaxed) & COUNTER_MASK;
        OverwritingDataSource { ring: self, index }
    }
}

impl<'a, T> WithCapacity for OverwritingDataSource<'a, T> {
    fn capacity(&self) -> usize {
        let buffer = unsafe { &*self.ring.buffer.get() };
        buffer.len()
    }
}

impl<'a, T: Copy + Clone> DataReader<T> for OverwritingDataSource<'a, T> {
    fn get(&mut self) -> Option<T> {
        let buffer = unsafe { &*self.ring.buffer.get() };
        loop {
            let meta = self.ring.meta.load(Ordering::Acquire);
            let counter = meta & COUNTER_MASK;
            if counter.wrapping_sub(self.index) & COUNTER_MASK == 0 || meta & VALID == 0 {
                return None;
            }
            if counter.wrapping_sub(self.index) & COUNTER_MASK > buffer.len() as u32 {
                self.index = counter.wrapping_sub(buffer.len() as u32) & COUNTER_MASK;
            }
            let value = buffer[self.index as usize % buffer.len()];
            if meta == self.ring.meta.load(Ordering::Relaxed) {
                self.index = self.index.wrapping_add(1);
                return Some(value);
            }
        }
    }

    fn get_last(&mut self) -> Option<T> {
        let buffer = unsafe { &*self.ring.buffer.get() };
        let meta = self.ring.meta.load(Ordering::Acquire);
        if meta & VALID == 0 {
            return None;
        }
        let counter = meta & COUNTER_MASK;
        self.index = counter.wrapping_sub(1) & COUNTER_MASK;
        Some(buffer[self.index as usize % buffer.len()])
    }
}

impl<'a, T> Clone for OverwritingDataSource<'a, T> {
    fn clone(&self) -> Self {
        Self { ring: self.ring, index: self.ring.meta.load(Ordering::Relaxed) & COUNTER_MASK }
    }
}

mod test {
    #[test]
    fn test_ring_buffer() {
        use super::{DataReader, DataWriter, OverwritingData};

        let ring: OverwritingData<usize> = OverwritingData::sized(32);
        let mut reader = ring.reader();

        assert_eq!(reader.get(), None);

        ring.write(10086);
        assert_eq!(reader.get(), Some(10086));

        ring.write(10010);
        assert_eq!(reader.get(), Some(10010));

        for i in 1..33 {
            ring.write(i);
        }

        assert_eq!(reader.get(), Some(1));
        assert_eq!(reader.get(), Some(2));
    }

    #[test]
    fn test_ring_buffer_as_static() {
        use core::sync::atomic::Ordering;

        use super::{DataReader, DataWriter, OverwritingData};

        let ring: OverwritingData<usize> = OverwritingData::sized(32);
        let mut reader = ring.reader();

        ring.meta.store(u32::MAX, Ordering::Relaxed);
        ring.write(10010);
        ring.write(10086);
        assert_eq!(reader.get_last(), Some(10086));
    }
}
