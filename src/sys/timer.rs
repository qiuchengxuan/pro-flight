use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;

use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::timer::CountDown;
use nb;
use void::Void;

use super::jiffies;

#[derive(Default)]
pub struct SysTimer(jiffies::Jiffies);

impl Future for SysTimer {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _ctx: &mut Context) -> Poll<Self::Output> {
        return if jiffies::get() >= self.0 { Poll::Ready(()) } else { Poll::Pending };
    }
}

impl CountDown for SysTimer {
    type Time = jiffies::Jiffies;

    fn start<T: Into<jiffies::Jiffies>>(&mut self, duration: T) {
        self.0 = jiffies::get() + duration.into();
    }

    fn wait(&mut self) -> Result<(), nb::Error<Void>> {
        return if jiffies::get() >= self.0 { Ok(()) } else { Err(nb::Error::WouldBlock) };
    }
}

extern "Rust" {
    #[cfg(feature = "sleep-ms")]
    fn timer_sleep_ms();
    #[cfg(feature = "sleep-us")]
    fn timer_sleep_us();
}

impl<T: Into<u64>> DelayMs<T> for SysTimer {
    fn delay_ms(&mut self, ms: T) {
        self.start(Duration::from_millis(ms.into()));
        match () {
            #[cfg(feature = "sleep-ms")]
            _ => {
                while self.wait().is_err() {
                    unsafe { timer_sleep_ms() };
                }
            }
            #[cfg(not(feature = "sleep-ms"))]
            _ => nb::block!(self.wait()).unwrap(),
        }
    }
}

impl<T: Into<u64>> DelayUs<T> for SysTimer {
    fn delay_us(&mut self, us: T) {
        self.start(Duration::from_micros(us.into()));
        match () {
            #[cfg(feature = "sleep-us")]
            _ => {
                while self.wait().is_err() {
                    unsafe { timer_sleep_us() };
                }
            }
            #[cfg(not(feature = "sleep-us"))]
            _ => nb::block!(self.wait()).unwrap(),
        }
    }
}
