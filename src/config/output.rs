use core::fmt::{Result, Write};

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

#[derive(PartialEq, Copy, Clone)]
pub enum Protocol {
    PWM(u16),
}

#[derive(Copy, Clone)]
pub enum Output {
    Motor(u8, Protocol),
    AileronLeft,
    AileronRight,
    Elevator,
    Rudder,
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
            Self::AileronLeft => "aileron-left",
            Self::AileronRight => "aileron-right",
            Self::Elevator => "elevator",
            Self::Rudder => "rudder",
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
        while let Some((key, value)) = parser.next_key_value() {
            match key {
                "type" => type_string = value,
                "index" => index = value.parse().ok().unwrap_or(0),
                "protocol" => protocol = value,
                "rate" => rate = value.parse().ok().unwrap_or(400),
                _ => continue,
            }
        }
        match type_string {
            "motor" => match protocol {
                "PWM" => Output::Motor(index, Protocol::PWM(rate)),
                _ => Output::Motor(0, Protocol::PWM(400)),
            },
            "aileron-left" => Output::AileronLeft,
            "aileron-right" => Output::AileronRight,
            "elevator" => Output::Elevator,
            "rudder" => Output::Rudder,
            _ => Output::None,
        }
    }
}

impl ToYAML for Output {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        let type_string: &str = (*self).into();
        writeln!(w, "type: {}", type_string)?;
        if let Output::Motor(index, protocol) = self {
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
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
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
