pub type EventHandler<T> = fn(item: T);

pub fn event_nop_handler<T>(_: T) {}
