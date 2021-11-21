use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct Notifier(Arc<AtomicBool>);

impl Notifier {
    pub fn set(&self) {
        self.0.as_ref().store(true, Ordering::Relaxed)
    }

    pub fn get(&self) -> bool {
        self.0.as_ref().load(Ordering::Relaxed)
    }
}

pub struct Receiver(Arc<AtomicBool>);

impl Receiver {
    pub fn get(&self) -> bool {
        self.0.as_ref().load(Ordering::Relaxed)
    }

    pub fn clear(&self) {
        self.0.as_ref().store(false, Ordering::Relaxed)
    }
}

pub fn trigger() -> (Notifier, Receiver) {
    let v = Arc::new(AtomicBool::new(false));
    (Notifier(v.clone()), Receiver(v))
}
