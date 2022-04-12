use core::str::{FromStr, Split};

use fixed_point::FixedPoint;

use super::setter::{Error, Setter, Value};

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum Configuration {
    Airplane,
    FlyingWing,
    VTail,
}

impl FromStr for Configuration {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, ()> {
        match string {
            "airplane" => Ok(Self::Airplane),
            "flying-wing" => Ok(Self::FlyingWing),
            "v-tail" => Ok(Self::VTail),
            _ => Err(()),
        }
    }
}

impl serde::Serialize for Configuration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = match self {
            Self::Airplane => "airplane",
            Self::FlyingWing => "flying-wing",
            Self::VTail => "v-tail",
        };
        serializer.serialize_str(s)
    }
}

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
            "max-rate" => self.max_rate = value.parse()?.unwrap_or(30),
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

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct FCS {
    pub configuration: Configuration,
    pub pids: PIDs,
}

impl Default for FCS {
    fn default() -> Self {
        Self { configuration: Configuration::Airplane, pids: Default::default() }
    }
}

impl Setter for FCS {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "configuration" => {
                self.configuration = value.parse()?.unwrap_or(Configuration::Airplane)
            }
            "pids" => self.pids.set(path, value)?,
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}
