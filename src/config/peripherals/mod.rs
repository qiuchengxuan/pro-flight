use core::str::Split;

use super::setter::{Error, Setter, Value};

pub mod pwm;
pub mod serial;

use pwm::PWMs;
use serial::Serials;

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Peripherals {
    pub serials: Serials,
    pub pwms: PWMs,
}

impl Peripherals {
    pub fn any(&self) -> bool {
        self.serials.len() > 0 || self.pwms.0.len() > 0
    }
}

impl Setter for Peripherals {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "serials" => self.serials.set(path, value),
            "pwms" => self.pwms.set(path, value),
            _ => Err(Error::MalformedPath),
        }
    }
}
