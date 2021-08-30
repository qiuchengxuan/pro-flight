use core::str::Split;

use heapless::LinearMap;
use serde::ser::SerializeMap;

use crate::datastructures::control::ControlType;

use super::setter::{Error, Setter, Value};

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Input {
    pub channel: u8,
    pub scale: u8,
}

impl Setter for Input {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "channel" => {
                self.channel = value.parse()?.ok_or(Error::ExpectValue)?;
            }
            "scale" => self.scale = value.parse()?.unwrap_or(100),
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Inputs(pub LinearMap<ControlType, Input, 18>);

impl Setter for Inputs {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let type_sring = path.next().ok_or(Error::MalformedPath)?;
        let input_type = type_sring.parse().map_err(|_| Error::MalformedPath)?;
        if self.0.contains_key(&input_type) {
            return self.0[&input_type].set(path, value);
        }
        let mut config = Input { channel: u8::MAX, scale: 100 };
        config.set(path, value)?;
        self.0.insert(input_type, config).ok();
        Ok(())
    }
}

impl serde::Serialize for Inputs {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (input_type, config) in self.0.iter() {
            map.serialize_entry(input_type, config)?;
        }
        map.end()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Receiver {
    pub inputs: Inputs,
}

impl Setter for Receiver {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let key = path.next().ok_or(Error::MalformedPath)?;
        if key != "inputs" {
            return Err(Error::MalformedPath);
        }
        self.inputs.set(path, value)
    }
}
