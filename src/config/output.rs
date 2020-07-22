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
            return Identifier::PWM(name[3..].parse().ok().unwrap_or(0));
        }
        Identifier::PWM(0)
    }
}

impl Into<bool> for Identifier {
    fn into(self) -> bool {
        match self {
            Self::PWM(index) => index > 0,
        }
    }
}

impl core::fmt::Display for Identifier {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::PWM(index) => write!(f, "PWM{}", index),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Protocol {
    PWM(u16),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Servo {
    AileronLeft,
    AileronRight,
    Elevator,
    Rudder,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Output {
    Motor(u8, Protocol),
    Servo(Servo, i8),
    None,
}

impl Output {
    pub fn rate(self) -> u16 {
        match self {
            Self::Motor(_, protocol) => match protocol {
                Protocol::PWM(rate) => rate,
            },
            _ => 50,
        }
    }
}

impl Into<&str> for Output {
    fn into(self) -> &'static str {
        match self {
            Self::Motor(_, _) => "motor",
            Self::Servo(Servo::AileronLeft, _) => "aileron-left",
            Self::Servo(Servo::AileronRight, _) => "aileron-right",
            Self::Servo(Servo::Elevator, _) => "elevator",
            Self::Servo(Servo::Rudder, _) => "rudder",
            Self::None => "none",
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
        let mut center_angle = 0;
        while let Some((key, value)) = parser.next_key_value() {
            match key {
                "type" => type_string = value,
                "index" => index = value.parse().ok().unwrap_or(0),
                "protocol" => protocol = value,
                "rate" => rate = value.parse().ok().unwrap_or(400),
                "center-angle" => center_angle = value.parse().ok().unwrap_or(0),
                _ => continue,
            }
        }
        if center_angle < -90 || center_angle > 90 {
            center_angle = 0;
        }
        match type_string {
            "motor" => match protocol {
                "PWM" => Self::Motor(index, Protocol::PWM(rate)),
                _ => Self::Motor(0, Protocol::PWM(400)),
            },
            "aileron-left" => Self::Servo(Servo::AileronLeft, center_angle),
            "aileron-right" => Self::Servo(Servo::AileronRight, center_angle),
            "elevator" => Self::Servo(Servo::Elevator, center_angle),
            "rudder" => Self::Servo(Servo::Rudder, center_angle),
            _ => Self::None,
        }
    }
}

impl ToYAML for Output {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        let type_string: &str = (*self).into();
        writeln!(w, "type: {}", type_string)?;
        match self {
            Self::Motor(index, protocol) => {
                self.write_indent(indent, w)?;
                writeln!(w, "index: {}", index)?;
                match protocol {
                    Protocol::PWM(rate) => {
                        self.write_indent(indent, w)?;
                        writeln!(w, "protocol: PWM")?;
                        self.write_indent(indent, w)?;
                        writeln!(w, "rate: {}", rate)?;
                    }
                }
            }
            Self::Servo(_, center_angle) => {
                if *center_angle != 0 {
                    self.write_indent(indent, w)?;
                    writeln!(w, "center-angle: {}", center_angle)?;
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
        match path.next() {
            Some("type") => match value {
                "motor" => *self = Self::Motor(0, Protocol::PWM(400)),
                "aileron-left" => *self = Self::Servo(Servo::AileronLeft, 0),
                "aileron-right" => *self = Self::Servo(Servo::AileronRight, 0),
                "elevator" => *self = Self::Servo(Servo::Elevator, 0),
                "rudder" => *self = Self::Servo(Servo::Rudder, 0),
                _ => return Err(SetError::UnexpectedValue),
            },
            Some("center-angle") => {
                let angle = match value.parse::<i8>() {
                    Ok(angle) => angle,
                    Err(_) => return Err(SetError::UnexpectedValue),
                };
                if angle < -90 || angle > 90 {
                    return Err(SetError::UnexpectedValue);
                }
                match self {
                    Self::Servo(servo, _) => *self = Self::Servo(*servo, angle),
                    _ => return Err(SetError::MalformedPath),
                }
            }
            _ => return Err(SetError::MalformedPath),
        }
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
    fn set(
        &mut self,
        path: &mut core::str::Split<char>,
        value: Option<&str>,
    ) -> Result<(), SetError> {
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
