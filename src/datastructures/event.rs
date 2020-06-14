use core::marker::PhantomData;

pub trait EventHandler<T> {
    fn handle(&mut self, event: T);
}

pub struct EventNopHandler<T>(PhantomData<T>);

impl<T> EventNopHandler<T> {
    pub fn new() -> Self {
        Self(PhantomData {})
    }
}

impl<T> EventHandler<T> for EventNopHandler<T> {
    fn handle(&mut self, _: T) {}
}
