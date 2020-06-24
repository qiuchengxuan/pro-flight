use core::fmt::{Result, Write};

use super::yaml::{FromYAML, ToYAML, YamlParser};

#[derive(Copy, Clone)]
pub enum Configuration {
    Airplane,
}

impl From<&str> for Configuration {
    fn from(string: &str) -> Self {
        match string {
            "airplane" => Self::Airplane,
            _ => Self::Airplane,
        }
    }
}

impl Into<&str> for Configuration {
    fn into(self) -> &'static str {
        match self {
            Self::Airplane => "airplane",
        }
    }
}

pub struct Aircraft {
    pub configuration: Configuration,
}

impl Default for Aircraft {
    fn default() -> Self {
        Self { configuration: Configuration::Airplane }
    }
}

impl FromYAML for Aircraft {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut configuration: &str = "airplane";
        while let Some((key, value)) = parser.next_key_value() {
            match key {
                "configuration" => configuration = value,
                _ => continue,
            }
        }
        Self { configuration: configuration.into() }
    }
}

impl ToYAML for Aircraft {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        let configuration: &str = self.configuration.into();
        writeln!(w, "configuration: {}", configuration)
    }
}
