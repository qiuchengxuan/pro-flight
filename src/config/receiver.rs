use core::fmt::{Result, Write};

use crate::datastructures::input::InputType;

use super::yaml::{FromYAML, ToYAML, YamlParser};

pub const MAX_CHANNEL: usize = 4;

#[derive(Copy, Clone)]
pub struct Channels(pub [Option<InputType>; MAX_CHANNEL]);

impl Default for Channels {
    fn default() -> Self {
        Self([
            Some(InputType::Roll),
            Some(InputType::Pitch),
            Some(InputType::Throttle),
            Some(InputType::Yaw),
        ])
    }
}

impl FromYAML for Channels {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut channels = Channels::default();
        while parser.next_list_begin() {
            let mut channel: usize = 0;
            let mut input_type: Option<InputType> = None;
            while let Some((key, value)) = parser.next_key_value() {
                match key {
                    "channel" => channel = value.parse().ok().unwrap_or(0),
                    "type" => input_type = InputType::from_str(value),
                    _ => continue,
                }
            }
            if 0 < channel && channel <= MAX_CHANNEL {
                channels.0[channel - 1] = input_type;
            }
        }
        channels
    }
}

impl ToYAML for Channels {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        for i in 0..self.0.len() {
            let channel = self.0[i];
            if let Some(input_type) = channel {
                self.write_indent(indent, w)?;
                writeln!(w, "- channel: {}", i + 1)?;
                self.write_indent(indent, w)?;
                let type_string: &str = input_type.into();
                writeln!(w, "  type: {}", type_string)?;
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
