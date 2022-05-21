use fixed_point::{fixed, FixedPoint};

use crate::types::sensor::{Bias, Gain, Rotation};

use super::pathset::{Error, Path, PathSet, Value};

pub type Sensitive = FixedPoint<i32, 2>; // LSB/unit

#[derive(Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Accelerometer {
    pub bias: Bias,
    pub gain: Gain,
    pub sensitive: Sensitive,
}

impl PathSet for Accelerometer {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "bias" => return self.bias.set(path, value),
            "gain" => return self.gain.set(path, value),
            "sensitive" => self.sensitive = value.parse()?,
            _ => return Err(Error::UnknownPath),
        }
        Ok(())
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Magnetometer {
    pub bias: Bias,
    pub gain: Gain,
    pub declination: FixedPoint<i32, 1>,
}

impl PathSet for Magnetometer {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "bias" => return self.bias.set(path, value),
            "gain" => return self.gain.set(path, value),
            "declination" => self.declination = value.parse()?,
            _ => return Err(Error::UnknownPath),
        }
        Ok(())
    }
}

const DEFAULT_KP: FixedPoint<i32, 3> = fixed!(0.25, 3);
const DEFAULT_KI: FixedPoint<i32, 3> = fixed!(0.005, 3);

#[derive(Debug, Copy, Clone, PartialEq, Serialize)]
pub struct Mahony {
    pub kp: FixedPoint<i32, 3>,
    pub ki: FixedPoint<i32, 3>,
}

impl Default for Mahony {
    fn default() -> Self {
        Self { kp: DEFAULT_KP, ki: DEFAULT_KI }
    }
}

impl PathSet for Mahony {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "kp" => self.kp = value.parse_or(DEFAULT_KP)?,
            "ki" => self.ki = value.parse_or(DEFAULT_KI)?,
            _ => return Err(Error::UnknownPath),
        }
        Ok(())
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize)]
pub struct IMU {
    pub accelerometer: Accelerometer,
    pub magnetometer: Magnetometer,
    pub mahony: Mahony,
    pub rotation: Rotation,
}

impl PathSet for IMU {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "accelerometer" => self.accelerometer.set(path, value),
            "magnetometer" => self.magnetometer.set(path, value),
            "mahony" => self.mahony.set(path, value),
            "rotation" => {
                self.rotation = value.parse()?;
                Ok(())
            }
            _ => Err(Error::UnknownPath),
        }
    }
}
