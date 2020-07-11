use core::cell::UnsafeCell;
use core::num::Wrapping;
use core::sync::atomic::{AtomicUsize, Ordering};

use super::{DataSource, DataWriter};

pub struct OverwritingData<'a, T> {
    buffer: UnsafeCell<&'a mut [T]>,
    write: AtomicUsize,
    written: AtomicUsize,
}

impl<'a, T: Copy + Clone> OverwritingData<'a, T> {
    pub fn new(buffer: &'a mut [T]) -> Self {
        Self {
            buffer: UnsafeCell::new(buffer),
            write: AtomicUsize::new(0),
            written: AtomicUsize::new(0),
        }
    }
}

impl<'a, T> DataWriter<T> for OverwritingData<'a, T> {
    fn write(&self, value: T) {
        let buffer = unsafe { &mut *self.buffer.get() };
        if buffer.len() > 0 {
            let write = self.write.load(Ordering::Relaxed) as usize;
            let next_write = (Wrapping(write) + Wrapping(1)).0;
            self.write.store(next_write, Ordering::Release);
            buffer[write % buffer.len()] = value;
            self.written.store(next_write, Ordering::Release);
        }
    }
}

pub struct OverwritingDataSource<'a, T> {
    ring: &'a OverwritingData<'a, T>,
    read: Wrapping<usize>,
}

impl<'a, T> OverwritingDataSource<'a, T> {
    pub fn new(ring: &'a OverwritingData<'a, T>) -> Self {
        Self { ring, read: Wrapping(ring.write.load(Ordering::Acquire)) }
    }
}

impl<'a, T: Copy + Clone> DataSource<T> for OverwritingDataSource<'a, T> {
    fn read(&mut self) -> Option<T> {
        let buffer = unsafe { &*self.ring.buffer.get() };
        let mut written = Wrapping(self.ring.written.load(Ordering::Acquire));
        let mut delta = (written - self.read).0;
        if delta == 0 {
            return None;
        }
        loop {
            delta = (written - self.read).0;
            if delta > buffer.len() {
                self.read = written - Wrapping(buffer.len());
            }
            let value = buffer[self.read.0 % buffer.len()];
            let write = self.ring.write.load(Ordering::Acquire);
            if (Wrapping(write) - self.read).0 <= buffer.len() {
                self.read += Wrapping(1);
                return Some(value);
            }
            written = Wrapping(self.ring.written.load(Ordering::Acquire));
            self.read += Wrapping(1);
        }
    }

    fn read_last(&mut self) -> Option<T> {
        let buffer = unsafe { &*self.ring.buffer.get() };
        let written = Wrapping(self.ring.written.load(Ordering::Acquire));
        if (written - self.read).0 == 0 {
            return None;
        }
        self.read = written;
        Some(buffer[(self.read - Wrapping(1)).0 % buffer.len()])
    }

    fn read_last_unchecked(&self) -> T {
        let buffer = unsafe { &*self.ring.buffer.get() };
        let written = Wrapping(self.ring.written.load(Ordering::Acquire));
        buffer[(written - Wrapping(1)).0 % buffer.len()]
    }

    fn capacity(&self) -> usize {
        let buffer = unsafe { &*self.ring.buffer.get() };
        buffer.len()
    }
}

impl<'a, T> Clone for OverwritingDataSource<'a, T> {
    fn clone(&self) -> Self {
        Self { ring: self.ring, read: Wrapping(self.ring.write.load(Ordering::Acquire)) }
    }
}

mod test {
    #[test]
    fn test_ring_buffer() {
        use core::sync::atomic::Ordering;

        use super::{DataSource, DataWriter, OverwritingData, OverwritingDataSource};

        let mut buffer = [0usize; 32];
        let ring = OverwritingData::new(&mut buffer);
        let mut reader = OverwritingDataSource::new(&ring);

        assert_eq!(reader.read(), None);

        ring.write(10086);
        assert_eq!(reader.read(), Some(10086));

        ring.write(10010);
        assert_eq!(reader.read(), Some(10010));

        for i in 1..33 {
            ring.write(i);
        }

        assert_eq!(reader.read(), Some(1));
        assert_eq!(reader.read(), Some(2));

        ring.write.store(usize::MAX, Ordering::Relaxed);
        ring.write(10010);
        ring.write(10086);
        assert_eq!(reader.read_last(), Some(10086));
    }
}
