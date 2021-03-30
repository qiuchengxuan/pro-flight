use nalgebra::UnitQuaternion;

use crate::datastructures::{
    coordinate::Position,
    input::ControlInput,
    input::RSSI,
    measurement::{
        battery::Battery,
        displacement::DistanceVector,
        unit::{CentiMeter, Meter},
        VelocityVector,
    },
    waypoint::Steerpoint,
};

use super::aviation::Aviation;
use super::sensor::Sensor;

#[derive(Copy, Clone, Debug, Default)]
pub struct Misc {
    pub battery: Battery,
    pub displacement: DistanceVector<i32, CentiMeter>,
    pub input: ControlInput,
    pub quaternion: UnitQuaternion<f32>,
    pub rssi: RSSI,
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
pub struct FlightData {
    pub aviation: Aviation,
    pub navigation: Navigation,
    pub sensor: Sensor,
    pub misc: Misc,
}

impl core::fmt::Display for FlightData {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval_json::to_fmt(f, self).ok();
        Ok(())
    }
}
