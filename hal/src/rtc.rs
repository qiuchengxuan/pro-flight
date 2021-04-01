use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};

pub trait RTCWriter: Send + Sync {
    fn set_date(&self, date: &NaiveDate);
    fn set_time(&self, time: &NaiveTime);
    fn set_datetime(&self, datetime: &NaiveDateTime);
}

pub trait RTCReader: Send + Sync {
    fn date(&self) -> NaiveDate;
    fn time(&self) -> NaiveTime;
}
