use embedded_hal::{digital::v2::OutputPin, timer::CountDown};
use fugit::NanosDurationU64 as Duration;
use hal::waker::Waker;

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

impl<E, T: From<Duration>, P: OutputPin<Error = E>, C: CountDown<Time = T>> Waker for LED<P, C> {
    fn wakeup(&mut self) {
        if !self.count_down.wait().is_ok() {
            return;
        }
        if self.on {
            self.count_down.start(Duration::millis(980));
            self.pin.set_high().ok();
        } else {
            self.count_down.start(Duration::millis(20));
            self.pin.set_low().ok();
        }
        self.on = !self.on;
    }
}
