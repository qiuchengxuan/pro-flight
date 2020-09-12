use alloc::rc::Rc;
use alloc::vec::Vec;

#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nmea0183::message::Message;
use nmea0183::messages::gga::GGA;
use nmea0183::messages::rmc::RMC;
use nmea0183::types::latitude::Latitude;
use nmea0183::types::longitude::Longitude;
use nmea0183::types::position_mode::PositionMode;
use nmea0183::types::{IntegerFloat, Quality};
use nmea0183::Parser;

use crate::datastructures::coordinate::Position;
use crate::datastructures::coordinate::{latitude, longitude};
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::u16_source::{U16Data, U16DataSource};
use crate::datastructures::data_source::DataWriter;
use crate::datastructures::decimal::IntegerDecimal;
use crate::datastructures::measurement::distance::Distance;
use crate::datastructures::measurement::unit::{CentiMeter, Knot, MilliMeter};
use crate::datastructures::measurement::VelocityVector;
use crate::datastructures::measurement::{Course, HeadingOrCourse};
use crate::datastructures::GNSSFixed;

pub struct NMEADecoder {
    parser: Parser,
    fixed: Rc<U16Data<GNSSFixed>>,
    position: Rc<SingularData<Position>>,
    velocity: Rc<SingularData<VelocityVector<i32, MilliMeter>>>,
    heading: Rc<SingularData<HeadingOrCourse>>,
    course: Rc<SingularData<Course>>,
}

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

impl<I: Copy, D: Copy> From<&IntegerFloat<I, D>> for IntegerDecimal<I, D> {
    fn from(float: &IntegerFloat<I, D>) -> Self {
        Self {
            integer: float.integer,
            decimal: float.decimal,
            decimal_length: float.decimal_length,
        }
    }
}

impl NMEADecoder {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            fixed: Rc::new(U16Data::default()),
            position: Rc::new(SingularData::default()),
            velocity: Rc::new(SingularData::default()),
            heading: Rc::new(SingularData::default()),
            course: Rc::new(SingularData::default()),
        }
    }

    pub fn fixed(&self) -> U16DataSource<GNSSFixed> {
        U16DataSource::new(&self.fixed)
    }

    pub fn position(&self) -> SingularDataSource<Position> {
        SingularDataSource::new(&self.position)
    }

    pub fn velocity(&self) -> SingularDataSource<VelocityVector<i32, MilliMeter>> {
        SingularDataSource::new(&self.velocity)
    }

    pub fn course(&self) -> SingularDataSource<Course> {
        SingularDataSource::new(&self.course)
    }

    pub fn heading(&self) -> SingularDataSource<HeadingOrCourse> {
        SingularDataSource::new(&self.heading)
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

        self.course.write((&rmc.course).into());
        if let Some(heading) = &rmc.heading {
            self.heading.write(HeadingOrCourse::Heading(heading.into()));
        } else {
            self.heading.write(HeadingOrCourse::Course((&rmc.course).into()));
        }

        let mut course: f32 = rmc.course.into();
        if course > 180.0 {
            course = -360.0 + course;
        }
        let speed: f32 = rmc.speed.into();
        let x = speed * course.to_radians().sin();
        let y = speed * course.to_radians().cos();
        let velocity = VelocityVector::new(x, y, 0.0, Knot);
        self.velocity.write(velocity.to_unit(MilliMeter).convert(|v| v as i32));
    }

    fn handle_gga(&mut self, gga: &GGA) {
        let fixed = match gga.quality {
            Quality::Autonomous | Quality::Differential => true,
            _ => false,
        };
        self.fixed.write(GNSSFixed(fixed));
        if !fixed {
            return;
        }
        let latitude = gga.latitude.into();
        let longitude = gga.longitude.into();

        let mut decimal = gga.altitude.decimal as i32;
        if gga.altitude.integer < 0 {
            decimal = -decimal;
        }
        let decimal_length = gga.altitude.decimal_length as u32;
        let decimal_part = match decimal_length {
            1 => decimal * 10,
            _ => decimal / 10i32.pow(decimal_length - 2),
        };
        let altitude = Distance::new(gga.altitude.integer * 100 + decimal_part, CentiMeter);
        self.position.write(Position { latitude, longitude, altitude });
    }

    pub fn handle(&mut self, ring: &[u8]) {
        let messages: Vec<Message> = self.parser.parse_bytes(&ring).collect();
        for message in messages.iter() {
            match message {
                Message::GGA(gga) => self.handle_gga(&gga),
                Message::RMC(rmc) => self.handle_rmc(&rmc),
                _ => continue,
            }
        }
    }
}
