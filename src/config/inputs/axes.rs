use fixed_point::FixedPoint;
use heapless::LinearMap;
use serde::ser::SerializeMap;

use crate::{
    config::pathset::{Error, Path, PathClear, PathSet, Value},
    types::control::AxisType,
    utils::LinearMapVisitor,
};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Axis {
    pub channel: u8,
    pub scale: FixedPoint<u8, 2>,
}

impl PathSet for Axis {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "channel" => self.channel = value.parse()?,
            "scale" => self.scale = value.parse_or(fixed_point::fixed!(1.0))?,
            _ => return Err(Error::UnknownPath),
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Axes(pub LinearMap<AxisType, Axis, 4>);

impl Default for Axes {
    fn default() -> Self {
        let mut axes = LinearMap::new();
        let scale = fixed_point::fixed!(1.0);
        axes.insert(AxisType::Throttle, Axis { channel: 3, scale }).ok();
        axes.insert(AxisType::Roll, Axis { channel: 1, scale }).ok();
        axes.insert(AxisType::Pitch, Axis { channel: 2, scale }).ok();
        axes.insert(AxisType::Yaw, Axis { channel: 4, scale }).ok();
        Self(axes)
    }
}

impl PathSet for Axes {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        let axis_type = path.parse()?;
        if self.0.contains_key(&axis_type) {
            return self.0[&axis_type].set(path, value);
        }
        let mut config = Axis { channel: u8::MAX, scale: fixed_point::fixed!(1.0) };
        config.set(path, value)?;
        self.0.insert(axis_type, config).ok();
        Ok(())
    }
}

impl PathClear for Axes {
    fn clear(&mut self, mut path: Path) -> Result<(), Error> {
        let axis_type = path.parse()?;
        self.0.remove(&axis_type);
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

impl<'de> serde::Deserialize<'de> for Axes {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(deserializer.deserialize_map(LinearMapVisitor::new())?))
    }
}
