use core::time::Duration;

use embedded_hal::digital::v2::ToggleableOutputPin;
use embedded_hal::timer::CountDown;

use crate::sys::timer::SysTimer;

pub struct Sysled<P> {
    pin: P,
    timer: SysTimer,
}

impl<E, P: ToggleableOutputPin<Error = E>> Sysled<P> {
    pub fn new(pin: P) -> Self {
        let mut timer = SysTimer::new();
        timer.start(Duration::from_millis(100));
        Self { pin, timer }
    }

    pub fn check_toggle(&mut self) -> Result<(), E> {
        if self.timer.wait().is_ok() {
            self.pin.toggle()?;
            self.timer.start(Duration::from_millis(100))
        }
        Ok(())
    }
}
