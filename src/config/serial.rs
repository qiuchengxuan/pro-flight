use core::fmt::{Result, Write};

use ascii::AsciiStr;
use btoi::btoi;

use super::yaml::{ByteStream, Entry, FromYAML, ToYAML};

#[derive(PartialEq, Copy, Clone)]
pub struct SbusConfig {
    pub fast: bool,
    pub rx_inverted: bool,
    pub half_duplex: bool,
}

impl SbusConfig {
    pub fn baudrate(&self) -> u32 {
        if !self.fast {
            100_000
        } else {
            200_000
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum SerialConfig {
    None,
    GNSS(u32),
    SBUS(SbusConfig),
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self::None
    }
}

impl ToYAML for SerialConfig {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        match self {
            SerialConfig::None => writeln!(w, "type: NONE"),
            SerialConfig::GNSS(baudrate) => {
                writeln!(w, "type: GNSS")?;
                self.write_indent(indent, w)?;
                writeln!(w, "baudrate: {}", baudrate)
            }
            SerialConfig::SBUS(sbus) => {
                writeln!(w, "type: SBUS")?;
                self.write_indent(indent, w)?;
                writeln!(w, "fast: {}", sbus.fast)?;
                self.write_indent(indent, w)?;
                writeln!(w, "rx-inverted: {}", sbus.rx_inverted)?;
                self.write_indent(indent, w)?;
                writeln!(w, "half-duplex: {}", sbus.half_duplex)
            }
        }
    }
}

const MAX_SERIAL_CONFIGS: usize = 5;

pub struct Serials {
    name_buffer: [u8; 6 * MAX_SERIAL_CONFIGS],
    configs: [(usize, SerialConfig); MAX_SERIAL_CONFIGS],
    num_config: u8,
}

impl Default for Serials {
    fn default() -> Self {
        Self {
            name_buffer: [0u8; 6 * MAX_SERIAL_CONFIGS],
            configs: [(0, SerialConfig::default()); MAX_SERIAL_CONFIGS],
            num_config: 0u8,
        }
    }
}

impl Serials {
    pub fn get(&self, name: &'static [u8]) -> Option<SerialConfig> {
        let mut index = 0;
        for i in 0..self.num_config as usize {
            let (length, config) = self.configs[i];
            if &self.name_buffer[index..index + length] == name {
                return Some(config);
            }
            index += length;
        }
        None
    }
}

fn to_serial_config<'a>(indent: usize, byte_iter: &'a mut ByteStream) -> SerialConfig {
    let mut type_string: &[u8] = &[];
    let mut baudrate = 0;
    let mut fast = false;
    let mut rx_inverted = false;
    let mut half_duplex = false;
    loop {
        match byte_iter.next(indent) {
            Entry::KeyValue(key, value) => match key {
                b"type" => type_string = value,
                b"baudrate" => baudrate = btoi(value).ok().unwrap_or(0),
                b"fast" => fast = value == b"true",
                b"rx-inverted" => rx_inverted = value == b"true",
                b"half-duplex" => half_duplex = value == b"true",
                _ => continue,
            },
            Entry::Key(_) => byte_iter.skip(indent),
            _ => break,
        }
    }
    match type_string {
        b"SBUS" => SerialConfig::SBUS(SbusConfig { fast, rx_inverted, half_duplex }),
        b"GNSS" => SerialConfig::GNSS(baudrate),
        _ => SerialConfig::None,
    }
}

impl FromYAML for Serials {
    fn from_yaml<'a>(&mut self, indent: usize, byte_iter: &'a mut ByteStream) {
        let mut index = 0;
        loop {
            match byte_iter.next(indent) {
                Entry::Key(key) => {
                    if self.num_config as usize >= MAX_SERIAL_CONFIGS {
                        byte_iter.skip(indent);
                    }
                    if key.len() > self.name_buffer.len() - index {
                        byte_iter.skip(indent);
                    }
                    let name = &mut self.name_buffer[index..index + key.len()];
                    name.copy_from_slice(key);
                    let config = to_serial_config(indent + 1, byte_iter);
                    self.configs[self.num_config as usize] = (key.len(), config);

                    index += key.len();
                    self.num_config += 1;
                }
                _ => return,
            }
        }
    }
}

impl ToYAML for Serials {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        let mut index = 0;
        for i in 0..self.num_config {
            self.write_indent(indent, w)?;
            let (size, config) = self.configs[i as usize];
            let name_bytes = &self.name_buffer[index..index + size];
            let string = unsafe { AsciiStr::from_ascii_unchecked(name_bytes) };
            writeln!(w, "{}:", string)?;
            config.write_to(indent + 1, w)?;
            index += size;
        }
        Ok(())
    }
}

mod test {
    #[cfg(test)]
    extern crate std;

    #[test]
    fn test_write() -> core::fmt::Result {
        use std::string::String;
        use std::string::ToString;

        use super::{SbusConfig, SerialConfig, Serials};
        use crate::config::yaml::ToYAML;

        let mut buf = String::new();
        let mut serials = Serials::default();
        serials.name_buffer[..12].copy_from_slice(b"USART1USART6");
        serials.configs[0] = (6, SerialConfig::GNSS(38400));
        let sbus_config = SbusConfig { fast: false, rx_inverted: true, half_duplex: false };
        serials.configs[1] = (6, SerialConfig::SBUS(sbus_config));
        serials.num_config = 2;
        serials.write_to(0, &mut buf)?;
        let expected = "\
        \nUSART1:\
        \n  type: GNSS\
        \n  baudrate: 38400\
        \nUSART6:\
        \n  type: SBUS\
        \n  fast: false\
        \n  rx-inverted: true\
        \n  half-duplex: false";
        assert_eq!(expected.trim(), buf.to_string().trim());
        Ok(())
    }
}
