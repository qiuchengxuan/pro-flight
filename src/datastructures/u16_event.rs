use core::sync::atomic::{AtomicU32, Ordering};

pub struct U16Event(AtomicU32);

impl Default for U16Event {
    fn default() -> Self {
        Self(AtomicU32::new(0))
    }
}

impl U16Event {
    pub fn notify(&self, value: u16) {
        let counter = self.0.load(Ordering::Relaxed) >> 16;
        self.0.store((counter + 1) << 16 | value as u32, Ordering::Relaxed)
    }
}

pub struct U16EventReader<'a> {
    source: &'a U16Event,
    counter: u16,
}

impl<'a> U16EventReader<'a> {
    pub fn new(source: &'a U16Event) -> Self {
        Self { source, counter: (source.0.load(Ordering::Relaxed) >> 16) as u16 }
    }

    pub fn get<T: From<u16>>(&mut self) -> Option<T> {
        let value = self.source.0.load(Ordering::Relaxed);
        if (value >> 16) as u16 == self.counter {
            None
        } else {
            self.counter += 1;
            Some((value as u16).into())
        }
    }
}
