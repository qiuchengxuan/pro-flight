use fixed_point::{fixed, FixedPoint};

use super::pathset::{Error, Path, PathSet, Value};

const DEFAULT_KP: FixedPoint<u16, 3> = fixed!(0.25, 3);

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Speedometer {
    pub kp: FixedPoint<u16, 3>,
}

impl Default for Speedometer {
    fn default() -> Self {
        Self { kp: DEFAULT_KP }
    }
}

impl PathSet for Speedometer {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "kp" => self.kp = value.parse_or(DEFAULT_KP)?,
            _ => return Err(Error::UnknownPath),
        }
        Ok(())
    }
}

#[repr(C, align(4))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct INS {
    pub speedometer: Speedometer,
}

impl PathSet for INS {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "speedometer" => self.speedometer.set(path, value),
            _ => Err(Error::UnknownPath),
        }
    }
}
