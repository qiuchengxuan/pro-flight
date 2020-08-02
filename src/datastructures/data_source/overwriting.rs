use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::num::Wrapping;
use core::sync::atomic::{AtomicUsize, Ordering};

use super::{DataSource, DataWriter};

pub struct OverwritingData<T> {
    buffer: UnsafeCell<Vec<T>>,
    write: AtomicUsize,
    written: AtomicUsize,
}

impl<T: Copy + Clone> OverwritingData<T> {
    pub fn new(vec: Vec<T>) -> Self {
        Self {
            buffer: UnsafeCell::new(vec),
            write: AtomicUsize::new(0),
            written: AtomicUsize::new(0),
        }
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
            let write = self.write.load(Ordering::Relaxed) as usize;
            let next_write = (Wrapping(write) + Wrapping(1)).0;
            self.write.store(next_write, Ordering::Relaxed);
            buffer[write % size] = value;
            self.written.store(next_write, Ordering::Relaxed);
        }
    }
}

pub struct OverwritingDataSource<T> {
    ring: Rc<OverwritingData<T>>,
    read: Wrapping<usize>,
}

impl<T> OverwritingDataSource<T> {
    pub fn new(ring: &Rc<OverwritingData<T>>) -> Self {
        Self { ring: Rc::clone(ring), read: Wrapping(ring.write.load(Ordering::Relaxed)) }
    }
}

impl<T: Copy + Clone> DataSource<T> for OverwritingDataSource<T> {
    fn read(&mut self) -> Option<T> {
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

    fn read_last(&mut self) -> Option<T> {
        let buffer = unsafe { &*self.ring.buffer.get() };
        let written = Wrapping(self.ring.written.load(Ordering::Relaxed));
        if (written - self.read).0 == 0 {
            return None;
        }
        self.read = written;
        Some(buffer[(self.read - Wrapping(1)).0 % buffer.len()])
    }

    fn read_last_unchecked(&self) -> T {
        let buffer = unsafe { &*self.ring.buffer.get() };
        let written = Wrapping(self.ring.written.load(Ordering::Relaxed));
        buffer[(written - Wrapping(1)).0 % buffer.len()]
    }

    fn capacity(&self) -> usize {
        let buffer = unsafe { &*self.ring.buffer.get() };
        buffer.len()
    }
}

impl<T> Clone for OverwritingDataSource<T> {
    fn clone(&self) -> Self {
        Self {
            ring: Rc::clone(&self.ring),
            read: Wrapping(self.ring.write.load(Ordering::Relaxed)),
        }
    }
}

mod test {
    #[test]
    fn test_ring_buffer() {
        use alloc::rc::Rc;
        use core::sync::atomic::Ordering;

        use super::{DataSource, DataWriter, OverwritingData, OverwritingDataSource};

        let ring: Rc<OverwritingData<usize>> = Rc::new(OverwritingData::sized(32));
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
