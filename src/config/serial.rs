use core::fmt::{Result, Write};

use super::yaml::{FromYAML, ToYAML, YamlParser};

#[derive(PartialEq, Copy, Clone)]
pub enum Identifier {
    UART(u8),
    USART(u8),
}

impl From<&str> for Identifier {
    fn from(name: &str) -> Identifier {
        if name.starts_with("USART") {
            return Identifier::USART(name[5..].parse().ok().unwrap_or(0));
        } else if name.starts_with("UART") {
            return Identifier::UART(name[4..].parse().ok().unwrap_or(0));
        }
        Identifier::UART(0)
    }
}

impl Into<bool> for Identifier {
    fn into(self) -> bool {
        match self {
            Self::UART(index) => index > 0,
            Self::USART(index) => index > 0,
        }
    }
}

impl core::fmt::Display for Identifier {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::USART(index) => write!(f, "USART{}", index),
            Self::UART(index) => write!(f, "UART{}", index),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum GNSSProtocol {
    UBX,
}

impl core::fmt::Display for GNSSProtocol {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "UBX")
    }
}

#[derive(PartialEq, Copy, Clone)]
pub struct GNSSConfig {
    pub baudrate: u32,
    pub protocol: GNSSProtocol,
}

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
pub enum Config {
    None,
    GNSS(GNSSConfig),
    SBUS(SbusConfig),
}

impl FromYAML for Config {
    fn from_yaml<'a>(parser: &mut YamlParser) -> Self {
        let mut type_string: &str = &"";
        let mut baudrate = 0;
        let mut fast = false;
        let mut rx_inverted = false;
        let mut half_duplex = false;
        while let Some((key, value)) = parser.next_key_value() {
            match key {
                "type" => type_string = value,
                "baudrate" => baudrate = value.parse().ok().unwrap_or(0),
                "fast" => fast = value == "true",
                "rx-inverted" => rx_inverted = value == "true",
                "half-duplex" => half_duplex = value == "true",
                "protocol" => continue,
                _ => continue,
            }
        }
        match type_string {
            "SBUS" => Config::SBUS(SbusConfig { fast, rx_inverted, half_duplex }),
            "GNSS" => Config::GNSS(GNSSConfig { baudrate, protocol: GNSSProtocol::UBX }),
            _ => Config::None,
        }
    }
}

impl ToYAML for Config {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        match self {
            Self::None => writeln!(w, "type: NONE"),
            Self::GNSS(gnss) => {
                writeln!(w, "type: GNSS")?;
                self.write_indent(indent, w)?;
                writeln!(w, "baudrate: {}", gnss.baudrate)?;
                self.write_indent(indent, w)?;
                writeln!(w, "protocol: {}", gnss.protocol)
            }
            Self::SBUS(sbus) => {
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

#[derive(Copy, Clone)]
pub struct Serials([(Identifier, Config); MAX_SERIAL_CONFIGS]);

impl Default for Serials {
    fn default() -> Self {
        Self([(Identifier::UART(0), Config::None); MAX_SERIAL_CONFIGS])
    }
}

impl Serials {
    pub fn get(&self, name: &str) -> Option<Config> {
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

impl FromYAML for Serials {
    fn from_yaml<'a>(parser: &mut YamlParser) -> Self {
        let mut serials = Self::default();
        let mut index = 0;
        while let Some(key) = parser.next_entry() {
            if index >= MAX_SERIAL_CONFIGS {
                parser.skip();
            }
            let id = Identifier::from(key);
            let config = Config::from_yaml(parser);
            if id.into() {
                serials.0[index] = (id, config);
                index += 1
            }
        }
        serials
    }
}

impl ToYAML for Serials {
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
