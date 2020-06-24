use core::fmt::{Result, Write};

use btoi::btoi;

use super::yaml::{FromYAML, ToYAML, YamlParser};

#[derive(Copy, Clone, PartialEq)]
pub enum Type {
    Throttle,
    Roll,
    Pitch,
    Yaw,
    None,
}

impl From<&str> for Type {
    fn from(string: &str) -> Self {
        match string {
            "throttle" => Self::Throttle,
            "roll" => Self::Roll,
            "pitch" => Self::Pitch,
            "yaw" => Self::Yaw,
            _ => Self::None,
        }
    }
}

impl Into<&str> for Type {
    fn into(self) -> &'static str {
        match self {
            Self::Throttle => "throttle",
            Self::Roll => "roll",
            Self::Pitch => "pitch",
            Self::Yaw => "yaw",
            Self::None => "none",
        }
    }
}

#[derive(Copy, Clone)]
pub struct Channel(Type);

pub const MAX_CHANNEL: usize = 16;

pub struct Channels([Channel; MAX_CHANNEL]);

impl Default for Channels {
    fn default() -> Self {
        Self([Channel(Type::None); MAX_CHANNEL])
    }
}

impl FromYAML for Channels {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut channels = [Channel(Type::None); MAX_CHANNEL];
        while parser.next_list_begin() {
            let mut channel: usize = 0;
            let mut channel_type: Type = Type::None;
            while let Some((key, value)) = parser.next_key_value() {
                match key {
                    "channel" => channel = btoi(value.as_bytes()).ok().unwrap_or(0),
                    "type" => channel_type = Type::from(value),
                    _ => continue,
                }
            }
            if 0 < channel && channel <= MAX_CHANNEL && channel_type != Type::None {
                channels[channel - 1] = Channel(channel_type);
            }
        }
        Self(channels)
    }
}

impl ToYAML for Channels {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        for i in 0..self.0.len() {
            let channel = self.0[i];
            if channel.0 == Type::None {
                continue;
            }
            self.write_indent(indent, w)?;
            writeln!(w, "- channel: {}", i + 1)?;
            self.write_indent(indent, w)?;
            let type_string: &str = self.0[i].0.into();
            writeln!(w, "  type: {}", type_string)?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct Receiver(Channels);

impl FromYAML for Receiver {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut channels = Channels::default();
        while let Some(key) = parser.next_entry() {
            if key == "channels" {
                channels = Channels::from_yaml(parser)
            }
        }
        Self(channels)
    }
}

impl ToYAML for Receiver {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        if (self.0).0.iter().all(|&c| c.0 == Type::None) {
            writeln!(w, "channels: []")
        } else {
            writeln!(w, "channels:")?;
            self.0.write_to(indent + 1, w)
        }
    }
}
