use super::schedule::{Hertz, Schedulable};

pub trait Notify {
    fn notify(&mut self);
}

pub trait OnEvent {
    fn on_event(&mut self);
}

#[derive(Copy, Clone)]
pub struct SchedulableEvent<T>(T, Hertz);

impl<T> SchedulableEvent<T> {
    pub fn new(notify: T, rate: Hertz) -> Self {
        Self(notify, rate)
    }
}

impl<T: Notify> Schedulable for SchedulableEvent<T> {
    fn schedule(&mut self) -> bool {
        self.0.notify();
        true
    }

    fn rate(&self) -> Hertz {
        self.1
    }
}
