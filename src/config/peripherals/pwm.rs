use core::{cmp, fmt::Write, str::FromStr};

use heapless::LinearMap;
use serde::{de::Error as _, ser::SerializeMap};

use crate::{
    config::pathset::{Error, Path, PathClear, PathSet, Value},
    utils::LinearMapVisitor,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Identifier(pub u8);

impl FromStr for Identifier {
    type Err = ();

    fn from_str(name: &str) -> Result<Identifier, ()> {
        if name.starts_with("PWM") {
            return Ok(Identifier(name[3..].parse::<u8>().map_err(|_| ())? - 1));
        }
        Err(())
    }
}

impl Identifier {
    pub fn equals_str(&self, string: &str) -> bool {
        Self::from_str(string).map(|v| v == *self).ok().unwrap_or(false)
    }
}

impl serde::Serialize for Identifier {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = heapless::String::<6>::new();
        write!(s, "PWM{}", self.0 + 1).ok();
        serializer.serialize_str(s.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Identifier {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = <&str>::deserialize(deserializer)?;
        if !s.starts_with("PWM") {
            return Err(D::Error::custom("Malformed PWM id"));
        }
        let value: u8 = s[3..].parse().map_err(|_| D::Error::custom("Malformed PWM id"))?;
        Ok(Self(value))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Protocol {
    PWM,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename = "motor")]
pub struct Motor {
    pub index: u8,
    pub protocol: Protocol,
    pub rate: u16,
}

impl Motor {
    pub fn new(protocol: Protocol, index: u8, rate: u16) -> Self {
        Self { protocol, index, rate }
    }
}

impl Default for Motor {
    fn default() -> Self {
        Self { protocol: Protocol::PWM, index: 0, rate: 400 }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServoType {
    AileronLeft,
    AileronRight,
    Elevator,
    ElevonLeft,
    ElevonRight,
    Rudder,
    RuddervatorLeft,
    RuddervatorRight,
}

impl Into<&str> for ServoType {
    fn into(self) -> &'static str {
        match self {
            Self::AileronLeft => "aileron-left",
            Self::AileronRight => "aileron-right",
            Self::Elevator => "elevator",
            Self::ElevonLeft => "elevon-left",
            Self::ElevonRight => "elevon-right",
            Self::Rudder => "rudder",
            Self::RuddervatorLeft => "ruddervator-left",
            Self::RuddervatorRight => "ruddervator-right",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Servo {
    #[serde(rename = "type")]
    pub servo_type: ServoType,
    pub min_angle: i8,
    pub max_angle: i8,
    pub reversed: bool,
}

impl Servo {
    pub fn new(servo_type: ServoType, min_angle: i8, max_angle: i8, reversed: bool) -> Self {
        Self { servo_type, min_angle, max_angle, reversed }
    }

    pub fn of(servo_type: ServoType) -> Self {
        Self { servo_type, min_angle: -90, max_angle: 90, reversed: false }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum PWM {
    Motor(Motor),
    Servo(Servo),
}

impl PWM {
    pub fn rate(self) -> u16 {
        match self {
            Self::Motor(motor) => motor.rate,
            _ => 50,
        }
    }
}

impl PathSet for PWM {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        let key = path.str()?;
        if key == "type" {
            let output_type = value.str()?;
            *self = match output_type {
                "motor" => Self::Motor(Motor::default()),
                "aileron-left" => Self::Servo(Servo::of(ServoType::AileronLeft)),
                "aileron-right" => Self::Servo(Servo::of(ServoType::AileronRight)),
                "elevator" => Self::Servo(Servo::of(ServoType::Elevator)),
                "rudder" => Self::Servo(Servo::of(ServoType::Rudder)),
                "elevon-left" => Self::Servo(Servo::of(ServoType::ElevonLeft)),
                "elevon-right" => Self::Servo(Servo::of(ServoType::ElevonRight)),
                "ruddervator-left" => Self::Servo(Servo::of(ServoType::RuddervatorLeft)),
                "ruddervator-right" => Self::Servo(Servo::of(ServoType::RuddervatorRight)),
                _ => return Err(Error::InvalidValue),
            };
            return Ok(());
        }
        match self {
            Self::Motor(ref mut motor) => match key {
                "index" => motor.index = value.parse()?,
                "protocol" => match value.0 {
                    Some("PWM") => motor.protocol = Protocol::PWM,
                    Some(_) => return Err(Error::InvalidValue),
                    _ => motor.protocol = Protocol::PWM,
                },
                "rate" => motor.rate = value.parse_or(400)?,
                _ => return Err(Error::UnknownPath),
            },
            Self::Servo(ref mut servo) => match key {
                "min-angle" => {
                    let min = value.parse_or(-90)?;
                    servo.min_angle = cmp::min(cmp::max(min, -90), 0)
                }
                "max-angle" => {
                    let max = value.parse_or(90)?;
                    servo.max_angle = cmp::max(cmp::min(max, 90), 0)
                }
                "reversed" => servo.reversed = value.parse()?,
                _ => return Err(Error::UnknownPath),
            },
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PWMs(pub LinearMap<Identifier, PWM, 8>);

impl PWMs {
    pub fn get(&self, name: &str) -> Option<&PWM> {
        Identifier::from_str(name).ok().map(|id| self.0.get(&id)).flatten()
    }
}

impl serde::Serialize for PWMs {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (id, config) in self.0.iter() {
            map.serialize_entry(id, config)?;
        }
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for PWMs {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(deserializer.deserialize_map(LinearMapVisitor::new())?))
    }
}

impl PathSet for PWMs {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        let id = path.parse()?;
        if self.0.contains_key(&id) {
            return self.0[&id].set(path, value);
        }
        let mut config = PWM::Motor(Motor::default());
        config.set(path, value)?;
        self.0.insert(id, config).ok();
        Ok(())
    }
}

impl PathClear for PWMs {
    fn clear(&mut self, mut path: Path) -> Result<(), Error> {
        let id: Identifier = path.parse()?;
        self.0.remove(&id);
        Ok(())
    }
}
