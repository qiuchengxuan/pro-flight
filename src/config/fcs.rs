use core::str::{FromStr, Split};

use fixed_point::FixedPoint;

use super::setter::{Error, Setter, Value};

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
