use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Default)]
pub struct Event(Arc<AtomicBool>);

pub trait Notifier {
    fn notify(&self);
}

impl Notifier for Event {
    fn notify(&self) {
        self.0.as_ref().store(true, Ordering::Relaxed)
    }
}

pub trait Subscriber {
    fn wait(&self) -> bool;
    fn clear(&self);
}

impl Subscriber for Event {
    fn wait(&self) -> bool {
        self.0.as_ref().load(Ordering::Relaxed)
    }

    fn clear(&self) {
        self.0.as_ref().store(false, Ordering::Relaxed)
    }
}
