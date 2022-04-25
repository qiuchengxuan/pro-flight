use core::{
    fmt::Write,
    str::{FromStr, Split},
};

use heapless::LinearMap;
use serde::{de::Error as _, ser::SerializeMap};

use crate::{
    config::setter::{Error, Setter, Value},
    utils::LinearMapVisitor,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Identifier {
    UART(u8),
    USART(u8),
}

impl FromStr for Identifier {
    type Err = ();

    fn from_str(name: &str) -> Result<Self, ()> {
        if name.starts_with("USART") {
            return Ok(Identifier::USART(name[5..].parse().map_err(|_| ())?));
        } else if name.starts_with("UART") {
            return Ok(Identifier::UART(name[4..].parse().map_err(|_| ())?));
        }
        Err(())
    }
}

impl serde::Serialize for Identifier {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = heapless::String::<8>::new();
        match self {
            Self::UART(index) => write!(s, "UART{}", index).ok(),
            Self::USART(index) => write!(s, "USART{}", index).ok(),
        };
        serializer.serialize_str(s.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Identifier {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = <&str>::deserialize(deserializer)?;
        let index = s.find(char::is_numeric).ok_or(D::Error::custom("Malformed serial id"))?;
        let value: u8 = s[index..].parse().map_err(|_| D::Error::custom("Malformed serial id"))?;
        match &s[..index] {
            "UART" => Ok(Self::UART(value)),
            "USART" => Ok(Self::USART(value)),
            _ => Err(D::Error::custom("Malformed serial id")),
        }
    }
}

impl Identifier {
    pub fn equals_str(&self, string: &str) -> bool {
        Self::from_str(string).map(|v| v == *self).ok().unwrap_or(false)
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

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum GNSSProtocol {
    UBX,
    NMEA,
}

impl FromStr for GNSSProtocol {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, ()> {
        match string {
            "UBX" => Ok(Self::UBX),
            "NMEA" => Ok(Self::NMEA),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename = "GNSS")]
pub struct GNSSConfig {
    pub baudrate: u32,
    pub protocol: GNSSProtocol,
}

impl Default for GNSSConfig {
    fn default() -> Self {
        Self { baudrate: 9600, protocol: GNSSProtocol::NMEA }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SbusConfig {
    pub fast: bool,
    pub rx_inverted: bool,
    pub half_duplex: bool,
}

impl Default for SbusConfig {
    fn default() -> Self {
        Self { fast: false, rx_inverted: true, half_duplex: false }
    }
}

impl SbusConfig {
    pub fn baudrate(&self) -> u32 {
        if !self.fast { 100_000 } else { 200_000 }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[repr(u8)]
pub enum RemoteControl {
    SBUS(SbusConfig),
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[repr(u8)]
pub enum Config {
    GNSS(GNSSConfig),
    RC(RemoteControl),
}

impl Setter for Config {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let key = path.next().ok_or(Error::MalformedPath)?;
        if key == "type" {
            *self = match value.0 {
                Some("GNSS") => Self::GNSS(GNSSConfig::default()),
                Some("SBUS") => Self::RC(RemoteControl::SBUS(SbusConfig::default())),
                Some(_) => return Err(Error::UnexpectedValue),
                _ => return Err(Error::ExpectValue),
            };
            return Ok(());
        }
        match self {
            Self::GNSS(ref mut gnss) => match key {
                "baudrate" => gnss.baudrate = value.parse()?.unwrap_or(9600),
                "protocol" => gnss.protocol = value.parse()?.unwrap_or(GNSSProtocol::NMEA),
                _ => return Err(Error::MalformedPath),
            },
            Self::RC(RemoteControl::SBUS(ref mut sbus)) => match key {
                "fast" => sbus.fast = value.parse()?.unwrap_or(false),
                "rx-inverted" => sbus.rx_inverted = value.parse()?.unwrap_or(true),
                "half-duplex" => sbus.half_duplex = value.parse()?.unwrap_or(false),
                _ => return Err(Error::MalformedPath),
            },
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Serials(LinearMap<Identifier, Config, 8>);

impl Serials {
    pub fn get(&self, name: &str) -> Option<&Config> {
        Identifier::from_str(name).ok().map(|id| self.0.get(&id)).flatten()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Setter for Serials {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let id_sring = path.next().ok_or(Error::MalformedPath)?;
        let id = Identifier::from_str(id_sring).map_err(|_| Error::MalformedPath)?;
        if self.0.contains_key(&id) {
            return self.0[&id].set(path, value);
        }
        let mut config = Config::GNSS(GNSSConfig::default());
        config.set(path, value)?;
        self.0.insert(id, config).ok();
        Ok(())
    }
}

impl serde::Serialize for Serials {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (&id, config) in self.0.iter() {
            if id.into() {
                map.serialize_entry(&id, config)?;
            }
        }
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for Serials {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(deserializer.deserialize_map(LinearMapVisitor::new())?))
    }
}
