use ascii_osd_hud::telemetry as hud;
#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::{Quaternion, UnitQuaternion};

use crate::datastructures::coordinate::Position;
use crate::datastructures::input::{ControlInput, RSSI};
use crate::datastructures::measurement::battery::Battery;
use crate::datastructures::measurement::displacement::DistanceVector;
use crate::datastructures::measurement::euler::Euler;
use crate::datastructures::measurement::unit::Meter;
use crate::datastructures::measurement::Course;
use crate::datastructures::measurement::{Acceleration, Altitude, Gyro, VelocityVector};
use crate::datastructures::waypoint::Steerpoint;

#[derive(Debug, Default, Copy, Clone)]
pub struct Attitude {
    pub roll: i16,
    pub pitch: i16,
}

impl From<Euler> for Attitude {
    fn from(euler: Euler) -> Self {
        let roll = (-euler.roll * 10.0) as i16;
        let pitch = (-euler.pitch * 10.0) as i16;
        Self { roll, pitch }
    }
}

impl Into<hud::Attitude> for Attitude {
    fn into(self) -> hud::Attitude {
        hud::Attitude { roll: self.roll / 10, pitch: (self.pitch / 10) as i8 }
    }
}

impl sval::value::Value for Attitude {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.map_begin(Some(2))?;
        stream.map_key("roll")?;
        stream.map_value(self.roll / 10)?;
        stream.map_key("pitch")?;
        stream.map_value(self.pitch / 10)?;
        stream.map_end()
    }
}

#[derive(Copy, Clone, Default, Debug, Value)]
pub struct Basic {
    pub altitude: Altitude,
    pub attitude: Attitude,
    pub heading: u16,
    pub height: Altitude,
    pub g_force: u8,
    pub airspeed: u16,
    pub vario: i16,
}

#[derive(Copy, Clone, Default, Debug, Value)]
pub struct Misc {
    pub rssi: RSSI,
    pub input: ControlInput,

    pub position: Position,
    pub steerpoint: Steerpoint,

    pub battery: Battery,
}

#[derive(Copy, Clone, Debug)]
pub struct GNSS {
    pub fixed: bool,
    pub course: Course,
}

impl sval::value::Value for GNSS {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.map_begin(Some(if self.fixed { 2 } else { 1 }))?;
        stream.map_key("fixed")?;
        stream.map_value(self.fixed)?;
        if self.fixed {
            stream.map_key("fixed")?;
            let course: f32 = self.course.into();
            stream.map_value(course)?;
        }
        stream.map_end()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Raw {
    pub acceleration: Acceleration,
    pub gyro: Gyro,
    pub quaternion: UnitQuaternion<f32>,
    pub gnss: Option<GNSS>,
    pub speed_vector: VelocityVector<f32, Meter>,
    pub displacement: DistanceVector<i32, Meter>,
}

impl Default for Raw {
    fn default() -> Self {
        Self {
            quaternion: UnitQuaternion::new_normalize(Quaternion::new(1.0, 0.0, 0.0, 0.0)),
            acceleration: Acceleration::default(),
            gyro: Gyro::default(),
            gnss: None,
            speed_vector: VelocityVector::default(),
            displacement: DistanceVector::default(),
        }
    }
}

impl sval::value::Value for Raw {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.map_begin(Some(5 + if self.gnss.is_some() { 1 } else { 0 }))?;
        stream.map_key("acceleration")?;
        stream.map_value(&self.acceleration)?;
        stream.map_key("gyro")?;
        stream.map_value(&self.gyro)?;
        stream.map_key("quaternion")?;
        let q = &self.quaternion;
        let value: [f32; 4] = [q.i, q.j, q.k, q.w];
        stream.map_value(&value[..])?;
        if let Some(gnss) = self.gnss {
            stream.map_key("gnss")?;
            stream.map_value(gnss)?;
        }
        stream.map_key("speed-vector")?;
        stream.map_value(&self.speed_vector)?;
        stream.map_key("displacement")?;
        stream.map_value(&self.displacement)?;
        stream.map_end()
    }
}

impl core::fmt::Display for Raw {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval_json::to_fmt(f, self).ok();
        Ok(())
    }
}

#[derive(Copy, Clone, Default, Debug, Value)]
pub struct TelemetryData {
    pub basic: Basic,
    pub misc: Misc,
    pub raw: Raw,
}

impl core::fmt::Display for TelemetryData {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval_json::to_fmt(f, self).ok();
        Ok(())
    }
}
