use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use embedded_hal::{
    blocking::delay::{DelayMs, DelayUs},
    timer::CountDown,
};
use fugit::NanosDurationU64 as Duration;
use hal::rtc::{RTCReader, RTCWriter};
use nb;
use void::Void;

pub const STEP_THRESHOLD: Duration = Duration::secs(5);

use super::jiffies;

#[derive(Copy, Clone, Debug)]
pub enum Error {
    NotImplemented,
    NotInitialized,
}

static mut RTC_READER: Option<Box<dyn RTCReader>> = None;
static mut RTC_WRITER: Option<Box<dyn RTCWriter>> = None;

pub fn date() -> NaiveDate {
    unsafe { RTC_READER.as_ref() }.map(|rtc| rtc.date()).unwrap_or(NaiveDate::from_ymd(1970, 1, 1))
}

pub fn time() -> NaiveTime {
    unsafe { RTC_READER.as_ref() }.map(|rtc| rtc.time()).unwrap_or_else(|| {
        let jiffies = jiffies::get();
        let nanos = jiffies.to_nanos();
        let (seconds, nanos) = (nanos / 1_000_000_000, nanos % 1_000_000_000);
        NaiveTime::from_num_seconds_from_midnight(seconds as u32, nanos as u32)
    })
}

pub fn now() -> NaiveDateTime {
    NaiveDateTime::new(date(), time())
}

pub fn update(datetime: &NaiveDateTime) -> Result<(), Error> {
    let now = now();
    let delta = (datetime.time() - now.time()).abs().num_nanoseconds().unwrap_or_default() as u64;
    if datetime.date() != now.date() || Duration::nanos(delta) > STEP_THRESHOLD {
        match unsafe { RTC_WRITER.as_ref() } {
            Some(w) => w.set_datetime(datetime),
            None => return Err(Error::NotInitialized),
        }
    }
    Ok(())
}

pub struct TickTimer(Duration);

impl Default for TickTimer {
    fn default() -> Self {
        Self(Duration::secs(0))
    }
}

impl TickTimer {
    pub fn after<T: Into<Duration>>(duration: T) -> Self {
        let mut timer = Self::default();
        timer.start(duration);
        timer
    }
}

impl Future for TickTimer {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _ctx: &mut Context) -> Poll<Self::Output> {
        return if jiffies::get() >= self.0 { Poll::Ready(()) } else { Poll::Pending };
    }
}

impl CountDown for TickTimer {
    type Time = Duration;

    fn start<T: Into<Duration>>(&mut self, duration: T) {
        self.0 = jiffies::get() + duration.into();
    }

    fn wait(&mut self) -> Result<(), nb::Error<Void>> {
        return if jiffies::get() >= self.0 { Ok(()) } else { Err(nb::Error::WouldBlock) };
    }
}

impl<T: Into<u32>> DelayMs<T> for TickTimer {
    fn delay_ms(&mut self, ms: T) {
        self.start(Duration::millis(ms.into() as u64));
        nb::block!(self.wait()).unwrap();
    }
}

impl<T: Into<u32>> DelayUs<T> for TickTimer {
    fn delay_us(&mut self, us: T) {
        self.start(Duration::micros(us.into() as u64));
        nb::block!(self.wait()).unwrap();
    }
}

#[inline]
pub fn async_sleep(d: Duration) -> TickTimer {
    let mut timer = TickTimer::default();
    timer.start(d);
    timer
}

pub fn init(reader: impl RTCReader + 'static, writer: impl RTCWriter + 'static) {
    unsafe { RTC_READER = Some(Box::new(reader)) }
    unsafe { RTC_WRITER = Some(Box::new(writer)) }
}
