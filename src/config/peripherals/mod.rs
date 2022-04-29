use super::pathset::{Error, Path, PathClear, PathSet, Value};

pub mod pwm;
pub mod serial;

use pwm::PWMs;
use serial::Serials;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Peripherals {
    pub serials: Serials,
    pub pwms: PWMs,
}

impl Peripherals {
    pub fn any(&self) -> bool {
        self.serials.len() > 0 || self.pwms.0.len() > 0
    }
}

impl PathSet for Peripherals {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "serials" => self.serials.set(path, value),
            "pwms" => self.pwms.set(path, value),
            _ => Err(Error::UnknownPath),
        }
    }
}

impl PathClear for Peripherals {
    fn clear(&mut self, mut path: Path) -> Result<(), Error> {
        match path.str()? {
            "serials" => self.serials.clear(path),
            "pwms" => self.pwms.clear(path),
            _ => Err(Error::UnknownPath),
        }
    }
}
