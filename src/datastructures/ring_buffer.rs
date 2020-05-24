use core::cell::Cell;
use core::cmp::min;
use core::sync::atomic::{AtomicPtr, Ordering};

pub struct RingBuffer<'a, T> {
    ring: &'a mut [T],
    write: AtomicPtr<usize>,
    read: Cell<usize>,
}

impl<'a, T: Copy> RingBuffer<'a, T> {
    pub fn new(buffer: &'a mut [T]) -> Self {
        Self {
            ring: buffer,
            write: AtomicPtr::new(0 as *mut usize),
            read: Cell::new(0),
        }
    }

    pub fn push(&mut self, t: T) {
        let write = self.write.load(Ordering::Relaxed) as usize;
        self.ring[write % self.ring.len()] = t;
        let write = (write + 1) as *mut usize;
        self.write.store(write, Ordering::Relaxed)
    }

    pub fn pop(&self) -> Option<T> {
        let mut write = self.write.load(Ordering::Relaxed) as usize;
        let read = self.read.take();
        let length = min(write - read, self.ring.len());
        if length == 0 {
            self.read.replace(read);
            return None;
        }
        let mut read = write - length;
        loop {
            let t = self.ring[read % self.ring.len()];
            let new_write = self.write.load(Ordering::Relaxed) as usize;
            if new_write == write {
                self.read.replace(read);
                return Some(t);
            }
            write = new_write;
            if write - read > self.ring.len() {
                read = write - self.ring.len()
            }
        }
    }
}

mod test {
    #[test]
    fn test_ring_buffer() {
        use super::RingBuffer;

        let mut buffer = [0u8; 32];
        let mut ring = RingBuffer::new(&mut buffer);
        assert_eq!(ring.pop(), None);
        ring.push(1u8);
        assert_eq!(ring.pop(), Some(1u8));
    }
}
