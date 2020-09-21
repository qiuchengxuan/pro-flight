use core::cmp;
use core::fmt::Write;
use core::str::Split;

use super::setter::{SetError, Setter};
use super::yaml::{FromYAML, ToYAML, YamlParser};

#[derive(PartialEq, Copy, Clone)]
pub enum Identifier {
    PWM(u8),
}

impl From<&str> for Identifier {
    fn from(name: &str) -> Identifier {
        if name.starts_with("PWM") {
            return Identifier::PWM(name[3..].parse::<u8>().ok().map(|v| v - 1).unwrap_or(u8::MAX));
        }
        Identifier::PWM(0)
    }
}

impl Into<bool> for Identifier {
    fn into(self) -> bool {
        match self {
            Self::PWM(index) => index < u8::MAX,
        }
    }
}

impl core::fmt::Display for Identifier {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::PWM(index) => write!(f, "PWM{}", index + 1),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Protocol {
    PWM,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Motor {
    pub protocol: Protocol,
    pub index: u8,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ServoType {
    AileronLeft,
    AileronRight,
    Elevator,
    Rudder,
}

impl Into<&str> for ServoType {
    fn into(self) -> &'static str {
        match self {
            Self::AileronLeft => "aileron-left",
            Self::AileronRight => "aileron-right",
            Self::Elevator => "elevator",
            Self::Rudder => "rudder",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Servo {
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Output {
    Motor(Motor),
    Servo(Servo),
    None,
}

impl Output {
    pub fn rate(self) -> u16 {
        match self {
            Self::Motor(motor) => motor.rate,
            _ => 50,
        }
    }
}

pub const MAX_OUTPUT_CONFIG: usize = 6;

impl FromYAML for Output {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut type_string: &str = &"";
        let mut index = 0;
        let mut protocol: &str = &"";
        let mut rate = 400;
        let mut min = -90;
        let mut max = 90;
        let mut reversed = false;
        while let Some((key, value)) = parser.next_key_value() {
            match key {
                "type" => type_string = value,
                "index" => index = value.parse().ok().unwrap_or(0),
                "protocol" => protocol = value,
                "rate" => rate = value.parse().ok().unwrap_or(400),
                "min-angle" => min = value.parse().ok().unwrap_or(-90),
                "max-angle" => max = value.parse().ok().unwrap_or(90),
                "reversed" => reversed = value == "true",
                _ => continue,
            }
        }
        min = cmp::min(cmp::max(min, -90), 0);
        max = cmp::max(cmp::min(max, 90), 0);
        match type_string {
            "motor" => match protocol {
                "PWM" => Self::Motor(Motor::new(Protocol::PWM, index, rate)),
                _ => Self::Motor(Motor::default()),
            },
            "aileron-left" => Self::Servo(Servo::new(ServoType::AileronLeft, min, max, reversed)),
            "aileron-right" => Self::Servo(Servo::new(ServoType::AileronRight, min, max, reversed)),
            "elevator" => Self::Servo(Servo::new(ServoType::Elevator, min, max, reversed)),
            "rudder" => Self::Servo(Servo::new(ServoType::Rudder, min, max, reversed)),
            _ => Self::None,
        }
    }
}

impl ToYAML for Output {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> core::fmt::Result {
        match self {
            Self::Motor(motor) => {
                self.write_indent(indent, w)?;
                writeln!(w, "type: motor")?;
                self.write_indent(indent, w)?;
                writeln!(w, "index: {}", motor.index)?;
                match motor.protocol {
                    Protocol::PWM => {
                        self.write_indent(indent, w)?;
                        writeln!(w, "protocol: PWM")?;
                    }
                }
                self.write_indent(indent, w)?;
                writeln!(w, "rate: {}", motor.rate)?;
            }
            Self::Servo(servo) => {
                self.write_indent(indent, w)?;
                let servo_type: &str = servo.servo_type.into();
                writeln!(w, "type: {}", servo_type)?;
                if servo.min_angle != -90 {
                    self.write_indent(indent, w)?;
                    writeln!(w, "min-angle: {}", servo.min_angle)?;
                }
                if servo.max_angle != 90 {
                    self.write_indent(indent, w)?;
                    writeln!(w, "max-angle: {}", servo.max_angle)?;
                }
                if servo.reversed {
                    self.write_indent(indent, w)?;
                    writeln!(w, "reversed: true")?;
                }
            }
            _ => (),
        }
        Ok(())
    }
}

impl Setter for Output {
    fn set(&mut self, path: &mut Split<char>, value: Option<&str>) -> Result<(), SetError> {
        let value = match value {
            Some(v) => v,
            None => return Err(SetError::ExpectValue),
        };
        let key = match path.next() {
            Some(key) => key,
            None => return Err(SetError::MalformedPath),
        };
        match key {
            "type" => match value {
                "motor" => *self = Self::Motor(Motor::default()),
                "aileron-left" => *self = Self::Servo(Servo::of(ServoType::AileronLeft)),
                "aileron-right" => *self = Self::Servo(Servo::of(ServoType::AileronRight)),
                "elevator" => *self = Self::Servo(Servo::of(ServoType::Elevator)),
                "rudder" => *self = Self::Servo(Servo::of(ServoType::Rudder)),
                _ => return Err(SetError::UnexpectedValue),
            },
            "min-angle" | "max-angle" => {
                let angle = match value.parse::<i8>() {
                    Ok(angle) => angle,
                    Err(_) => return Err(SetError::UnexpectedValue),
                };
                match self {
                    Self::Servo(servo) => {
                        if key == "min-angle" {
                            servo.min_angle = cmp::min(cmp::max(angle, -90), 0);
                        } else {
                            servo.max_angle = cmp::max(cmp::min(angle, 90), 0);
                        }
                    }
                    _ => return Err(SetError::MalformedPath),
                }
            }
            "reversed" => match self {
                Self::Servo(servo) => servo.reversed = value == "true",
                _ => return Err(SetError::MalformedPath),
            },
            _ => return Err(SetError::MalformedPath),
        };
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct Outputs(pub [(Identifier, Output); MAX_OUTPUT_CONFIG]);

impl Default for Outputs {
    fn default() -> Outputs {
        Outputs([(Identifier::PWM(0), Output::None); MAX_OUTPUT_CONFIG])
    }
}

impl Outputs {
    pub fn get(&self, name: &str) -> Option<Output> {
        let identifier = Identifier::from(name);
        if identifier.into() {
            for &(id, config) in self.0.iter() {
                if id == identifier {
                    return Some(config);
                }
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.0.iter().filter(|&&(id, _)| id.into()).count()
    }
}

impl FromYAML for Outputs {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut outputs = Outputs::default();
        let mut index = 0;
        while let Some(key) = parser.next_entry() {
            let id = Identifier::from(key);
            let config = Output::from_yaml(parser);
            if id.into() {
                outputs.0[index] = (id, config);
                index += 1;
            }
        }
        outputs
    }
}

impl ToYAML for Outputs {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> core::fmt::Result {
        for &(id, config) in self.0.iter() {
            if id.into() {
                self.write_indent(indent, w)?;
                writeln!(w, "{}:", id)?;
                config.write_to(indent + 1, w)?;
            }
        }
        Ok(())
    }
}

impl Setter for Outputs {
    fn set(&mut self, path: &mut Split<char>, value: Option<&str>) -> Result<(), SetError> {
        let id_string = match path.next() {
            Some(token) => token,
            None => return Err(SetError::MalformedPath),
        };
        let index = if id_string.starts_with("PWM") {
            match id_string[3..].parse::<usize>() {
                Ok(index) => index - 1,
                Err(_) => return Err(SetError::MalformedPath),
            }
        } else {
            return Err(SetError::MalformedPath);
        };
        if index >= MAX_OUTPUT_CONFIG {
            return Err(SetError::MalformedPath);
        }
        self.0[index].1.set(path, value)
    }
}
