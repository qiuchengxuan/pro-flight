use core::str::{FromStr, Split};

use super::setter::{Error, Setter, Value};

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum Configuration {
    Airplane,
    FlyingWing,
    VTail,
}

impl FromStr for Configuration {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, ()> {
        match string {
            "airplane" => Ok(Self::Airplane),
            "flying-wing" => Ok(Self::FlyingWing),
            "v-tail" => Ok(Self::VTail),
            _ => Err(()),
        }
    }
}

impl serde::Serialize for Configuration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = match self {
            Self::Airplane => "airplane",
            Self::FlyingWing => "flying-wing",
            Self::VTail => "v-tail",
        };
        serializer.serialize_str(s)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Aircraft {
    pub configuration: Configuration,
}

impl Default for Aircraft {
    fn default() -> Self {
        Self { configuration: Configuration::Airplane }
    }
}

impl Setter for Aircraft {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "configuration" => {
                self.configuration = value.parse()?.unwrap_or(Configuration::Airplane)
            }
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}
