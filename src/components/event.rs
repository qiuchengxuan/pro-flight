use super::schedule::{Rate, Schedulable};

pub trait Notify {
    fn notify(&mut self);
}

pub trait OnEvent {
    fn on_event(&mut self);
}

#[derive(Copy, Clone)]
pub struct SchedulableEvent<T>(T, Rate);

impl<T> SchedulableEvent<T> {
    pub fn new(notify: T, rate: Rate) -> Self {
        Self(notify, rate)
    }
}

impl<T: Notify> Schedulable for SchedulableEvent<T> {
    fn schedule(&mut self) -> bool {
        self.0.notify();
        true
    }

    fn rate(&self) -> Rate {
        self.1
    }
}
