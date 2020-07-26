use core::time::Duration;

use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::timer::CountDown;
use nb;
use void::Void;

fn no_jiffies() -> Duration {
    Duration::default()
}

static mut JIFFIES: fn() -> Duration = no_jiffies;

pub fn get_jiffies() -> Duration {
    unsafe { JIFFIES() }
}

pub fn init(jiffies: fn() -> Duration) {
    unsafe { JIFFIES = jiffies }
}

pub struct SysTimer(Duration);

impl SysTimer {
    pub fn new() -> Self {
        Self(Duration::default())
    }
}

impl CountDown for SysTimer {
    type Time = Duration;

    fn start<T: Into<Duration>>(&mut self, duration: T) {
        self.0 = get_jiffies() + duration.into();
    }

    fn wait(&mut self) -> Result<(), nb::Error<Void>> {
        if get_jiffies() >= self.0 {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<T: Into<u64>> DelayMs<T> for SysTimer {
    fn delay_ms(&mut self, ms: T) {
        self.start(Duration::from_millis(ms.into()));
        nb::block!(self.wait()).unwrap();
    }
}

impl<T: Into<u64>> DelayUs<T> for SysTimer {
    fn delay_us(&mut self, us: T) {
        self.start(Duration::from_micros(us.into()));
        nb::block!(self.wait()).unwrap();
    }
}
