use core::time::Duration;

use embedded_hal::{digital::v2::OutputPin, timer::CountDown};
use hal::event::Notifier;

pub struct LED<P, C> {
    pin: P,
    count_down: C,
    on: bool,
}

impl<E, T: From<Duration>, P: OutputPin<Error = E>, C: CountDown<Time = T>> LED<P, C> {
    pub fn new(mut pin: P, count_down: C) -> Self {
        pin.set_low().ok();
        Self { pin, count_down, on: true }
    }
}

impl<E, T: From<Duration>, P: OutputPin<Error = E>, C: CountDown<Time = T>> Notifier for LED<P, C> {
    fn notify(&mut self) {
        if !self.count_down.wait().is_ok() {
            return;
        }
        if self.on {
            self.count_down.start(Duration::from_millis(980));
            self.pin.set_high().ok();
        } else {
            self.count_down.start(Duration::from_millis(20));
            self.pin.set_low().ok();
        }
        self.on = !self.on;
    }
}
