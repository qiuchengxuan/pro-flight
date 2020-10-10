use nalgebra::UnitQuaternion;

use crate::datastructures::coordinate::Position;
use crate::datastructures::input::ControlInput;
use crate::datastructures::measurement::displacement::DistanceVector;
use crate::datastructures::measurement::unit::Meter;
use crate::datastructures::measurement::VelocityVector;
use crate::datastructures::waypoint::Steerpoint;

use super::sensor::Sensor;
use super::status::Status;

#[derive(Copy, Clone, Debug, Default)]
pub struct Misc {
    pub displacement: DistanceVector<i32, Meter>,
    pub input: ControlInput,
    pub quaternion: UnitQuaternion<f32>,
}

impl sval::value::Value for Misc {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.map_begin(Some(3))?;
        stream.map_key("displacement")?;
        stream.map_value(&self.displacement)?;
        stream.map_key("input")?;
        stream.map_value(&self.input)?;
        stream.map_key("quaternion")?;
        let q = &self.quaternion;
        let value: [f32; 4] = [q.i, q.j, q.k, q.w];
        stream.map_value(&value[..])?;
        stream.map_end()
    }
}

impl core::fmt::Display for Misc {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval_json::to_fmt(f, self).ok();
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default, Value)]
pub struct Navigation {
    pub position: Position,
    pub speed_vector: VelocityVector<f32, Meter>,
    pub steerpoint: Steerpoint,
}

impl core::fmt::Display for Navigation {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval_json::to_fmt(f, self).ok();
        Ok(())
    }
}

#[derive(Copy, Clone, Default, Debug, Value)]
pub struct TelemetryData {
    pub status: Status,
    pub navigation: Navigation,
    pub sensor: Sensor,
    pub misc: Misc,
}

impl core::fmt::Display for TelemetryData {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval_json::to_fmt(f, self).ok();
        Ok(())
    }
}
