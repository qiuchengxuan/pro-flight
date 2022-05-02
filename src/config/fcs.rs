use core::str::FromStr;

use fixed_point::FixedPoint;

use super::pathset::{Error, Path, PathSet, Value};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "kebab-case")]
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

#[derive(Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PID {
    #[serde(rename = "max-rate")]
    pub max_rate: u16,
    pub kp: FixedPoint<u16, 2>,
    pub ki: FixedPoint<u16, 2>,
    pub kd: FixedPoint<u16, 2>,
}

impl PathSet for PID {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "max-rate" => self.max_rate = value.parse()?,
            "kp" => self.kp = value.parse_or(fixed_point::fixed!(1.0))?,
            "ki" => self.ki = value.parse_or(fixed_point::fixed!(1.0))?,
            "kd" => self.kd = value.parse_or(fixed_point::fixed!(1.0))?,
            _ => return Err(Error::UnknownPath),
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PIDs {
    pub roll: PID,
    pub pitch: PID,
    pub yaw: PID,
}

impl Default for PIDs {
    fn default() -> Self {
        Self {
            roll: PID {
                max_rate: 30,
                kp: fixed_point::fixed!(0.44),
                ki: fixed_point::fixed!(0.4),
                kd: fixed_point::fixed!(0.2),
            },
            pitch: PID {
                max_rate: 30,
                kp: fixed_point::fixed!(0.58),
                ki: fixed_point::fixed!(0.5),
                kd: fixed_point::fixed!(0.22),
            },
            yaw: PID {
                max_rate: 30,
                kp: fixed_point::fixed!(0.7),
                ki: fixed_point::fixed!(0.45),
                kd: fixed_point::fixed!(0.2),
            },
        }
    }
}

impl PathSet for PIDs {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "roll" => self.roll.set(path, value),
            "pitch" => self.pitch.set(path, value),
            "yaw" => self.yaw.set(path, value),
            _ => Err(Error::UnknownPath),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Envelop {
    pub max_roll: u8,
    pub min_pitch: i8,
    pub max_pitch: i8,
}

impl Default for Envelop {
    fn default() -> Self {
        Self { max_roll: 67, min_pitch: -15, max_pitch: 30 }
    }
}

impl PathSet for Envelop {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "max-roll" => self.max_roll = value.parse()?,
            "min-pitch" => self.min_pitch = value.parse()?,
            "max-pitch" => self.max_pitch = value.parse()?,
            _ => return Err(Error::UnknownPath),
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FCS {
    pub configuration: Configuration,
    pub envelop: Envelop,
    pub pids: PIDs,
}

impl Default for FCS {
    fn default() -> Self {
        Self {
            configuration: Configuration::Airplane,
            envelop: Default::default(),
            pids: Default::default(),
        }
    }
}

impl PathSet for FCS {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "configuration" => {
                self.configuration = value.parse_or(Configuration::Airplane)?;
                Ok(())
            }
            "envelop" => self.envelop.set(path, value),
            "pids" => self.pids.set(path, value),
            _ => Err(Error::UnknownPath),
        }
    }
}
