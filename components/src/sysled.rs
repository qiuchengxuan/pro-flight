use core::time::Duration;

use embedded_hal::digital::v2::ToggleableOutputPin;
use embedded_hal::timer::CountDown;

pub struct Sysled<P, C> {
    pin: P,
    count_down: C,
}

impl<E, P: ToggleableOutputPin<Error = E>, C: CountDown<Time = Duration>> Sysled<P, C> {
    pub fn new(pin: P, mut count_down: C) -> Self {
        count_down.start(Duration::from_millis(100));
        Self { pin, count_down }
    }

    pub fn check_toggle(&mut self) -> Result<(), E> {
        match self.count_down.wait() {
            Ok(_) => {
                self.pin.toggle()?;
                self.count_down.start(Duration::from_millis(100));
            }
            Err(_) => (),
        }
        Ok(())
    }
}
