use alloc::vec::Vec;

#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nmea0183::{
    message::Message,
    messages::{gga::GGA, rmc::RMC},
    types::{latitude::Latitude, longitude::Longitude, position_mode::PositionMode, Quality},
    Parser,
};

use crate::datastructures::{
    coordinate::{latitude, longitude, Position},
    fixed_point::FixedPoint,
    measurement::{distance::Distance, unit, Course, Heading, VelocityVector},
};
use crate::protocol::serial;
use crate::protocol::serial::gnss::DataSource;
use crate::sync::singular::SingularData;
use crate::sync::DataWriter;

impl Into<longitude::Longitude> for Longitude {
    fn into(self) -> longitude::Longitude {
        let degrees = self.degrees() as i32 * 3600 * longitude::SUB_SECOND;
        let minutes = self.minutes() as i32 * 60 * longitude::SUB_SECOND;
        let seconds = self.seconds() as i32 * longitude::SUB_SECOND;
        let value = degrees + minutes + seconds + self.sub_seconds() as i32;
        longitude::Longitude(if self.is_east() { value } else { -value })
    }
}

impl Into<latitude::Latitude> for Latitude {
    fn into(self) -> latitude::Latitude {
        let degrees = self.degrees() as i32 * 3600 * latitude::SUB_SECOND;
        let minutes = self.minutes() as i32 * 60 * latitude::SUB_SECOND;
        let seconds = self.seconds() as i32 * latitude::SUB_SECOND;
        let value = degrees + minutes + seconds + self.sub_seconds() as i32;
        latitude::Latitude(if self.is_north() { value } else { -value })
    }
}

impl From<nmea0183::types::IntegerDecimal> for FixedPoint<i32, 1> {
    fn from(decimal: nmea0183::types::IntegerDecimal) -> Self {
        Self(decimal.0)
    }
}

pub struct NMEA<'a> {
    parser: Parser,
    fixed: &'a SingularData<bool>,
    position: &'a SingularData<Position>,
    velocity: &'a SingularData<VelocityVector<i32, unit::MMpS>>,
    heading: &'a SingularData<Heading>,
    course: &'a SingularData<Course>,
}

impl<'a> NMEA<'a> {
    pub fn new(data_source: DataSource<'a>) -> Self {
        Self {
            parser: Parser::new(),
            fixed: data_source.fixed,
            position: data_source.position,
            velocity: data_source.velocity,
            heading: data_source.heading,
            course: data_source.course,
        }
    }

    fn handle_rmc(&mut self, rmc: &RMC) {
        match rmc.position_mode {
            PositionMode::Autonomous | PositionMode::Differential => (),
            _ => return,
        };

        if !rmc.status.0 {
            self.velocity.write(VelocityVector::default());
            return;
        }

        let course_valid = rmc.speed.integer() > 0;
        if course_valid {
            self.course.write(rmc.course.into());
        }
        if let Some(heading) = rmc.heading {
            self.heading.write(heading.into());
        }

        let course: f32 = rmc.course.into();
        let speed: f32 = rmc.speed.into();
        let x = speed * course.to_radians().sin();
        let y = speed * course.to_radians().cos();
        let velocity = VelocityVector::new(x, y, 0.0, unit::Knot);
        self.velocity.write(velocity.to_unit(unit::MMpS).convert(|v| v as i32));
    }

    fn handle_gga(&mut self, gga: &GGA) {
        let fixed = match gga.quality {
            Quality::Autonomous | Quality::Differential => true,
            _ => false,
        };
        self.fixed.write(fixed);
        if !fixed {
            return;
        }
        let latitude = gga.latitude.into();
        let longitude = gga.longitude.into();

        let integer = gga.altitude.integer() as i32;
        let decimal = gga.altitude.decimal() as i32;
        let decimal_part = match gga.altitude.decimal_length() {
            0 => 0,
            1 => decimal * 10,
            _ => decimal / 10i32.pow(gga.altitude.decimal_length() as u32 - 2),
        };
        let altitude = Distance::new(integer * 100 + decimal_part, unit::CentiMeter);
        self.position.write(Position { latitude, longitude, altitude });
    }
}

impl<'a> serial::Receiver for NMEA<'a> {
    fn receive(&mut self, bytes: &[u8]) {
        let messages: Vec<Message> = self.parser.parse_bytes(&bytes).collect();
        for message in messages.iter() {
            match message {
                Message::GGA(gga) => self.handle_gga(&gga),
                Message::RMC(rmc) => self.handle_rmc(&rmc),
                _ => continue,
            }
        }
    }
}
