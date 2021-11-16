use core::{
    cmp,
    fmt::Write,
    str::{FromStr, Split},
};

use heapless::LinearMap;
use serde::ser::SerializeMap;

use crate::config::setter::{Error, Setter, Value};

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

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub enum Protocol {
    PWM,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Motor {
    pub index: u8,
    pub protocol: Protocol,
    pub rate: u16,
}

impl serde::Serialize for Motor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(4))?;
        map.serialize_entry("type", "motor")?;
        map.serialize_entry("index", &self.index)?;
        map.serialize_entry("protocol", &self.protocol)?;
        map.serialize_entry("rate", &self.rate)?;
        map.end()
    }
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

impl serde::Serialize for ServoType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str((*self).into())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
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

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
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

impl Setter for PWM {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let key = path.next().ok_or(Error::MalformedPath)?;
        if key == "type" {
            let output_type = value.0.ok_or(Error::ExpectValue)?;
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
                _ => return Err(Error::UnexpectedValue),
            };
            return Ok(());
        }
        match self {
            Self::Motor(ref mut motor) => match key {
                "index" => motor.index = value.parse()?.unwrap_or(0),
                "protocol" => match value.0 {
                    Some("PWM") => motor.protocol = Protocol::PWM,
                    Some(_) => return Err(Error::UnexpectedValue),
                    _ => motor.protocol = Protocol::PWM,
                },
                "rate" => motor.rate = value.parse()?.unwrap_or(400),
                _ => return Err(Error::MalformedPath),
            },
            Self::Servo(ref mut servo) => match key {
                "min-angle" => {
                    let min = value.parse()?.unwrap_or(-90);
                    servo.min_angle = cmp::min(cmp::max(min, -90), 0)
                }
                "max-angle" => {
                    let max = value.parse()?.unwrap_or(90);
                    servo.max_angle = cmp::max(cmp::min(max, 90), 0)
                }
                "reversed" => servo.reversed = value.parse()?.unwrap_or_default(),
                _ => return Err(Error::MalformedPath),
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

impl Setter for PWMs {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let id_string = path.next().ok_or(Error::MalformedPath)?;
        if !id_string.starts_with("PWM") {
            return Err(Error::MalformedPath);
        }
        let id = id_string.parse().map_err(|_| Error::MalformedPath)?;
        if self.0.contains_key(&id) {
            return self.0[&id].set(path, value);
        }
        let mut config = PWM::Motor(Motor::default());
        config.set(path, value)?;
        self.0.insert(id, config).ok();
        Ok(())
    }
}
