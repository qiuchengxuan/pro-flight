use core::str::Split;

use fixed_point::FixedPoint;
use heapless::LinearMap;
use serde::ser::SerializeMap;

use crate::types::control::AxisType;

use super::setter::{Error, Setter, Value};

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Axis {
    pub channel: u8,
    pub scale: FixedPoint<u8, 2>,
}

impl Setter for Axis {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "channel" => {
                self.channel = value.parse()?.ok_or(Error::ExpectValue)?;
            }
            "scale" => self.scale = value.parse()?.unwrap_or(fixed_point::fixed!(1.0)),
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Axes(pub LinearMap<AxisType, Axis, 4>);

impl Setter for Axes {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let type_sring = path.next().ok_or(Error::MalformedPath)?;
        let axis_type = type_sring.parse().map_err(|_| Error::MalformedPath)?;
        if self.0.contains_key(&axis_type) {
            return self.0[&axis_type].set(path, value);
        }
        let mut config = Axis { channel: u8::MAX, scale: fixed_point::fixed!(1.0) };
        config.set(path, value)?;
        self.0.insert(axis_type, config).ok();
        Ok(())
    }
}

impl serde::Serialize for Axes {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (axis_type, config) in self.0.iter() {
            map.serialize_entry(axis_type, config)?;
        }
        map.end()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Inputs {
    pub axes: Axes,
}

impl Setter for Inputs {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let key = path.next().ok_or(Error::MalformedPath)?;
        if key != "axes" {
            return Err(Error::MalformedPath);
        }
        self.axes.set(path, value)
    }
}

impl Default for Inputs {
    fn default() -> Self {
        let mut axes = Axes::default();
        let scale = fixed_point::fixed!(1.0);
        axes.0.insert(AxisType::Throttle, Axis { channel: 3, scale }).ok();
        axes.0.insert(AxisType::Roll, Axis { channel: 1, scale }).ok();
        axes.0.insert(AxisType::Pitch, Axis { channel: 2, scale }).ok();
        axes.0.insert(AxisType::Yaw, Axis { channel: 4, scale }).ok();
        Self { axes }
    }
}
