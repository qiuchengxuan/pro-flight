use core::cell::UnsafeCell;
use core::num::Wrapping;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct RingBuffer<'a, T> {
    buffer: UnsafeCell<&'a mut [T]>,
    write: AtomicUsize,
    written: AtomicUsize,
}

impl<'a, T: Copy + Clone> RingBuffer<'a, T> {
    pub fn new(buffer: &'a mut [T]) -> Self {
        Self {
            buffer: UnsafeCell::new(buffer),
            write: AtomicUsize::new(0),
            written: AtomicUsize::new(0),
        }
    }

    pub fn write(&self, value: T) {
        let buffer = unsafe { &mut *self.buffer.get() };
        if buffer.len() > 0 {
            let write = self.write.load(Ordering::Relaxed) as usize;
            let next_write = (Wrapping(write) + Wrapping(1)).0;
            self.write.store(next_write, Ordering::Relaxed);
            buffer[write % buffer.len()] = value;
            self.written.store(next_write, Ordering::Relaxed);
        }
    }
}

pub struct RingBufferReader<'a, T> {
    ring: &'a RingBuffer<'a, T>,
    read: Wrapping<usize>,
}

impl<'a, T: Copy + Clone> RingBufferReader<'a, T> {
    pub fn new(ring: &'a RingBuffer<'a, T>) -> Self {
        Self { ring, read: Wrapping(ring.write.load(Ordering::Relaxed)) }
    }

    pub fn read(&mut self) -> Option<T> {
        let buffer = unsafe { &*self.ring.buffer.get() };
        let mut written = Wrapping(self.ring.written.load(Ordering::Relaxed));
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
            let write = self.ring.write.load(Ordering::Relaxed);
            if (Wrapping(write) - self.read).0 <= buffer.len() {
                self.read += Wrapping(1);
                return Some(value);
            }
            written = Wrapping(self.ring.written.load(Ordering::Relaxed));
            self.read += Wrapping(1);
        }
    }

    pub fn read_latest(&mut self) -> Option<T> {
        let buffer = unsafe { &*self.ring.buffer.get() };
        let written = Wrapping(self.ring.written.load(Ordering::Relaxed));
        if (written - self.read).0 == 0 {
            return None;
        }
        self.read = written;
        Some(buffer[(self.read - Wrapping(1)).0 % buffer.len()])
    }
}

impl<'a, T> Clone for RingBufferReader<'a, T> {
    fn clone(&self) -> Self {
        Self { ring: self.ring, read: Wrapping(self.ring.write.load(Ordering::Relaxed)) }
    }
}

mod test {
    #[test]
    fn test_ring_buffer() {
        use core::sync::atomic::Ordering;

        use super::{RingBuffer, RingBufferReader};

        let mut buffer = [0usize; 32];
        let ring = RingBuffer::new(&mut buffer);
        let mut reader = RingBufferReader::new(&ring);

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
        assert_eq!(reader.read_latest(), Some(10086));
    }
}
