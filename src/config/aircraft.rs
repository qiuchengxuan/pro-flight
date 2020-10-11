use core::fmt::Write;
use core::str::{FromStr, Split};

use super::setter::{Error, Setter, Value};

use super::yaml::ToYAML;

#[derive(Copy, Clone)]
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

impl Into<&str> for Configuration {
    fn into(self) -> &'static str {
        match self {
            Self::Airplane => "airplane",
            Self::FlyingWing => "flying-wing",
            Self::VTail => "v-tail",
        }
    }
}

#[derive(Copy, Clone)]
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

impl ToYAML for Aircraft {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        let configuration: &str = self.configuration.into();
        writeln!(w, "configuration: {}", configuration)
    }
}
