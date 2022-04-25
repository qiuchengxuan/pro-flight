use core::str::Split;

use fixed_point::{fixed, FixedPoint};

use super::setter::{Error, Setter, Value};

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

impl Setter for Speedometer {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "kp" => self.kp = value.parse()?.unwrap_or(DEFAULT_KP),
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

#[repr(C, align(4))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct INS {
    pub speedometer: Speedometer,
}

impl Setter for INS {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "speedometer" => self.speedometer.set(path, value),
            _ => Err(Error::MalformedPath),
        }
    }
}
