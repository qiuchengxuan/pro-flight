use alloc::boxed::Box;

use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use hal::rtc::{RTCReader, RTCWriter};

use crate::sys::jiffies;

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
        let (seconds, nanos) = (jiffies.as_secs() as u32, jiffies.subsec_nanos());
        NaiveTime::from_num_seconds_from_midnight(seconds, nanos)
    })
}

pub fn now() -> NaiveDateTime {
    NaiveDateTime::new(date(), time())
}

pub fn update(datetime: &NaiveDateTime) -> Result<(), Error> {
    match unsafe { RTC_WRITER.as_ref() } {
        Some(w) => w.set_datetime(datetime),
        None => return Err(Error::NotInitialized),
    }
    Ok(())
}

pub fn init(reader: impl RTCReader + 'static, writer: impl RTCWriter + 'static) {
    unsafe { RTC_READER = Some(Box::new(reader)) }
    unsafe { RTC_WRITER = Some(Box::new(writer)) }
}
