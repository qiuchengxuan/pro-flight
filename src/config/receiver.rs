use core::fmt::{Result, Write};

use crate::datastructures::input::InputType;

use super::yaml::{FromYAML, ToYAML, YamlParser};

pub const MAX_CHANNEL: usize = 4;

#[derive(Copy, Clone)]
pub struct Channel {
    pub input_type: InputType,
    pub scale: u8,
}

#[derive(Copy, Clone)]
pub struct Channels(pub [Option<Channel>; MAX_CHANNEL]);

impl Default for Channels {
    fn default() -> Self {
        Self([None; MAX_CHANNEL])
    }
}

impl FromYAML for Channels {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut channels = Channels::default();
        while parser.next_list_begin() {
            let mut channel: usize = 0;
            let mut input_type: Option<InputType> = None;
            let mut scale: u8 = 100;
            while let Some((key, value)) = parser.next_key_value() {
                match key {
                    "channel" => channel = value.parse().ok().unwrap_or(0),
                    "type" => input_type = InputType::from_str(value),
                    "scale" => scale = value.parse().ok().unwrap_or(100),
                    _ => continue,
                }
            }
            if 0 < channel && channel <= MAX_CHANNEL {
                if let Some(input_type) = input_type {
                    channels.0[channel - 1] = Some(Channel { input_type, scale });
                }
            }
        }
        channels
    }
}

impl ToYAML for Channels {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        for i in 0..self.0.len() {
            if let Some(channel) = self.0[i] {
                self.write_indent(indent, w)?;
                writeln!(w, "- channel: {}", i + 1)?;
                self.write_indent(indent, w)?;
                let type_string: &str = channel.input_type.into();
                writeln!(w, "  type: {}", type_string)?;
                self.write_indent(indent, w)?;
                writeln!(w, "  scale: {}", channel.scale)?;
            }
        }
        Ok(())
    }
}

#[derive(Default, Copy, Clone)]
pub struct Receiver {
    pub channels: Channels,
}

impl FromYAML for Receiver {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut channels = Channels::default();
        while let Some(key) = parser.next_entry() {
            if key == "channels" {
                channels = Channels::from_yaml(parser)
            }
        }
        Self { channels }
    }
}

impl ToYAML for Receiver {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        if self.channels.0.iter().all(|&c| c.is_none()) {
            writeln!(w, "channels: []")
        } else {
            writeln!(w, "channels:")?;
            self.channels.write_to(indent + 1, w)
        }
    }
}
