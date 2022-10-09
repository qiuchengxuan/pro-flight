use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use fixed_point::FixedPoint;
use nmea0183::{
    message::Message,
    messages::{gga::GGA, rmc::RMC},
    types::{latitude::Latitude, longitude::Longitude, position_mode::PositionMode, Quality},
    Parser, MAX_MESSAGE_SIZE,
};

use super::out::{Fixed, GNSS};
use crate::types::{
    coordinate::{latitude, longitude, Position, U_DEGREE},
    measurement::{unit, Altitude, Course, Distance, Heading, Velocity},
};

pub const CHUNK_SIZE: usize = MAX_MESSAGE_SIZE;

impl Into<longitude::Longitude<U_DEGREE>> for Longitude {
    fn into(self) -> longitude::Longitude<U_DEGREE> {
        let degrees = self.degrees() as i32 * 3600 * longitude::SUB_SECOND;
        let minutes = self.minutes() as i32 * 60 * longitude::SUB_SECOND;
        let seconds = self.seconds() as i32 * longitude::SUB_SECOND;
        let value = degrees + minutes + seconds + self.sub_seconds() as i32;
        longitude::Longitude(if self.is_east() { value } else { -value })
    }
}

impl Into<latitude::Latitude<U_DEGREE>> for Latitude {
    fn into(self) -> latitude::Latitude<U_DEGREE> {
        let degrees = self.degrees() as i32 * 3600 * latitude::SUB_SECOND;
        let minutes = self.minutes() as i32 * 60 * latitude::SUB_SECOND;
        let seconds = self.seconds() as i32 * latitude::SUB_SECOND;
        let value = degrees + minutes + seconds + self.sub_seconds() as i32;
        latitude::Latitude(if self.is_north() { value } else { -value })
    }
}

fn to_fixed_point(decimal: nmea0183::types::IntegerDecimal) -> FixedPoint<i32, 1> {
    FixedPoint(decimal.real() / (decimal.exp() as i32 / 10))
}

pub fn rmc_to_datetime(rmc: &RMC) -> NaiveDateTime {
    let (date, time) = (rmc.date, rmc.time);
    let date = NaiveDate::from_ymd_opt(date.year.into(), date.month.into(), date.day.into());
    let time = NaiveTime::from_hms_opt(time.hour.into(), time.minute.into(), time.seconds.into());
    NaiveDateTime::new(date.unwrap(), time.unwrap())
}

pub struct NMEA {
    parser: Parser,
    rmc: Option<RMC>,
    gga: Option<GGA>,
}

fn nmea_to_gnss(rmc: &RMC, gga: &GGA) -> GNSS {
    let datetime = Some(rmc_to_datetime(rmc));
    if !rmc.status.0 || rmc.position_mode == PositionMode::NoFix || gga.quality == Quality::NoFix {
        return GNSS { datetime, ..Default::default() };
    }

    let heading = rmc.heading.map(|h| Heading(to_fixed_point(h)));
    let course = Course(to_fixed_point(rmc.course));
    let latitude = gga.latitude.into();
    let longitude = gga.longitude.into();
    let alt = gga.altitude;
    let altitude =
        Altitude(Distance::new(alt.real(), unit::Meter).u(unit::CentiMeter) / alt.exp() as i32);
    let position = Position { latitude, longitude, altitude };
    let ground_speed =
        Velocity::new(rmc.speed.real(), unit::Knot).u(unit::MMs) / rmc.speed.exp() as i32;
    let fixed = Fixed { position, course, heading, ground_speed, velocity_vector: None };
    GNSS { datetime, fixed: Some(fixed) }
}

impl NMEA {
    pub fn new() -> Self {
        Self { parser: Parser::new(), rmc: None, gga: None }
    }

    pub fn receive(&mut self, bytes: &[u8]) -> Option<GNSS> {
        let mut retval = None;
        for message in self.parser.parse_bytes(&bytes) {
            match message {
                Message::GGA(gga) => self.gga = Some(gga),
                Message::RMC(rmc) => self.rmc = Some(rmc),
                _ => continue,
            };
            match (&self.rmc, &self.gga) {
                (Some(rmc), Some(gga)) => {
                    retval = Some(nmea_to_gnss(&rmc, &gga));
                    self.rmc = None;
                    self.gga = None;
                }
                _ => continue,
            }
        }
        retval
    }

    pub fn reset(&mut self) {
        self.parser.reset();
        self.rmc = None;
        self.gga = None;
    }
}
