use alloc::rc::Rc;
use alloc::vec::Vec;

#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nmea0183::{GPSQuality, Mode, ParseResult, Parser, GGA, RMC};

use crate::datastructures::coordinate::Position;
use crate::datastructures::coordinate::{latitude, longitude};
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::u16_source::{U16Data, U16DataSource};
use crate::datastructures::data_source::DataWriter;
use crate::datastructures::measurement::distance::Distance;
use crate::datastructures::measurement::unit::{CentiMeter, Meter, MilliMeter};
use crate::datastructures::measurement::VelocityVector;
use crate::datastructures::measurement::{Course, HeadingOrCourse};
use crate::datastructures::GNSSFixed;

pub struct NMEADecoder {
    parser: Parser,
    fixed: Rc<U16Data<GNSSFixed>>,
    position: Rc<SingularData<Position>>,
    velocity: Rc<SingularData<VelocityVector<i32, MilliMeter>>>,
    heading: Rc<U16Data<HeadingOrCourse>>,
    course: Rc<U16Data<Course>>,
}

impl NMEADecoder {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            fixed: Rc::new(U16Data::default()),
            position: Rc::new(SingularData::default()),
            velocity: Rc::new(SingularData::default()),
            heading: Rc::new(U16Data::default()),
            course: Rc::new(U16Data::default()),
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

    pub fn course(&self) -> U16DataSource<Course> {
        U16DataSource::new(&self.course)
    }

    pub fn heading(&self) -> U16DataSource<HeadingOrCourse> {
        U16DataSource::new(&self.heading)
    }

    fn handle_rmc(&mut self, rmc: &RMC) {
        let fixed = match rmc.mode {
            Mode::Autonomous | Mode::Differential => true,
            _ => false,
        };
        self.fixed.write(GNSSFixed(fixed));
        if !fixed {
            return;
        }
        if let Some(course) = &rmc.course {
            let mut degrees = course.degrees;
            self.course.write(degrees as u16);
            if let Some(magnetic) = &rmc.magnetic {
                let degrees: f32 = unsafe { core::mem::transmute(magnetic.clone()) };
                self.heading.write(HeadingOrCourse::Heading(degrees as u16));
            } else {
                self.heading.write(HeadingOrCourse::Course(degrees as u16));
            }

            if degrees > 180.0 {
                degrees = -360.0 + degrees;
            }

            let speed = rmc.speed.as_mps();
            let x = speed * degrees.to_radians().sin();
            let y = speed * degrees.to_radians().cos();
            let velocity = VelocityVector::new(x, y, 0.0, Meter);
            self.velocity.write(velocity.to_unit(MilliMeter).convert(|v| v as i32));
        } else {
            self.velocity.write(VelocityVector::default());
        }
    }

    fn handle_gga(&mut self, gga: &GGA) {
        let fixed = match gga.gps_quality {
            GPSQuality::GPS | GPSQuality::DGPS => true,
            _ => false,
        };
        self.fixed.write(GNSSFixed(fixed));
        if !fixed {
            return;
        }
        let lat = &gga.latitude;
        let as_seconds = lat.degrees as i32 * 3600 + lat.minutes as i32 * 60;
        let seconds = (lat.seconds * (latitude::SUB_SECOND as f32)) as i32;
        let value = as_seconds * latitude::SUB_SECOND + seconds;
        let latitude = latitude::Latitude(if lat.is_north() { value } else { -value });

        let lon = &gga.longitude;
        let as_seconds = lon.degrees as i32 * 3600 + lon.minutes as i32 * 60;
        let seconds = (lon.seconds * (longitude::SUB_SECOND as f32)) as i32;
        let value = as_seconds * latitude::SUB_SECOND + seconds;
        let longitude = longitude::Longitude(if lon.is_east() { value } else { -value });
        let meters = Distance::new(gga.altitude.meters, Meter);
        let altitude = meters.to_unit(CentiMeter).convert(|v| v as i32);
        self.position.write(Position { latitude, longitude, altitude });
    }

    pub fn handle(&mut self, ring: &[u8]) {
        let entries: Vec<ParseResult> =
            self.parser.parse_from_bytes(&ring).filter(|r| r.is_ok()).map(|r| r.unwrap()).collect();
        for entry in entries {
            match entry {
                ParseResult::RMC(Some(rmc)) => self.handle_rmc(&rmc),
                ParseResult::GGA(Some(gga)) => self.handle_gga(&gga),
                _ => continue,
            }
        }
    }
}
