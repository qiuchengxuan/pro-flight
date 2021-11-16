use core::str::Split;

use fixed_point::FixedPoint;

use super::setter::{Error, Setter, Value};

#[derive(Copy, Clone, Default, Debug, PartialEq, Serialize)]
pub struct PID {
    #[serde(rename = "max-rate")]
    pub max_rate: u16,
    pub kp: FixedPoint<u16, 2>,
    pub ki: FixedPoint<u16, 2>,
    pub kd: FixedPoint<u16, 2>,
}

impl Setter for PID {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "max-rate" => self.max_rate = value.parse()?.unwrap_or_default(),
            "kp" => self.kp = value.parse()?.unwrap_or(FixedPoint(1_0)),
            "ki" => self.ki = value.parse()?.unwrap_or(FixedPoint(1_0)),
            "kd" => self.kd = value.parse()?.unwrap_or(FixedPoint(1_0)),
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Serialize)]
pub struct PIDs {
    pub roll: PID,
    pub pitch: PID,
    pub yaw: PID,
}

impl Setter for PIDs {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "roll" => self.roll.set(path, value),
            "pitch" => self.pitch.set(path, value),
            "yaw" => self.yaw.set(path, value),
            _ => Err(Error::MalformedPath),
        }
    }
}
