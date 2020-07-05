pub mod aircraft;
pub mod battery;
pub mod osd;
pub mod output;
pub mod receiver;
pub mod sensor;
pub mod serial;
pub mod yaml;

use core::fmt::{Result, Write};

use btoi::btoi;

use crate::hal::io::Read;
use crate::hal::sensors::Axis;

pub use aircraft::Aircraft;
pub use battery::Battery;
pub use osd::{AspectRatio, Offset, Standard, OSD};
pub use output::{Output, Outputs, Protocol};
pub use receiver::Receiver;
pub use sensor::Accelerometer;
pub use serial::{Config as SerialConfig, Serials};
use yaml::{FromYAML, ToYAML, YamlParser};

impl FromYAML for Axis {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut axis = Self::default();
        while let Some((key, value)) = parser.next_key_value() {
            let value = btoi(value.as_bytes()).unwrap_or(0);
            match key {
                "x" => axis.x = value,
                "y" => axis.y = value,
                "z" => axis.z = value,
                _ => continue,
            }
        }
        axis
    }
}

impl ToYAML for Axis {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        writeln!(w, "x: {}", self.x)?;
        self.write_indent(indent, w)?;
        writeln!(w, "y: {}", self.y)?;
        self.write_indent(indent, w)?;
        writeln!(w, "z: {}", self.z)
    }
}

#[derive(Default)]
pub struct Config {
    pub accelerometer: Accelerometer,
    pub battery: Battery,
    pub aircraft: Aircraft,
    pub osd: OSD,
    pub receiver: Receiver,
    pub serials: Serials,
    pub outputs: Outputs,
}

impl FromYAML for Config {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut config = Self::default();
        while let Some(key) = parser.next_entry() {
            match key {
                "accelerometer" => config.accelerometer = Accelerometer::from_yaml(parser),
                "aircraft" => config.aircraft = Aircraft::from_yaml(parser),
                "battery" => config.battery = Battery::from_yaml(parser),
                "osd" => config.osd = OSD::from_yaml(parser),
                "receiver" => config.receiver = Receiver::from_yaml(parser),
                "serials" => config.serials = Serials::from_yaml(parser),
                "outputs" => config.outputs = Outputs::from_yaml(parser),
                _ => parser.skip(),
            };
        }
        config
    }
}

impl ToYAML for Config {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        writeln!(w, "accelerometer:")?;
        self.accelerometer.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "aircraft:")?;
        self.aircraft.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "battery:")?;
        self.battery.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "osd:")?;
        self.osd.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "receiver:")?;
        self.receiver.write_to(indent + 1, w)?;

        if self.serials.len() > 0 {
            self.write_indent(indent, w)?;
            writeln!(w, "serials:")?;
            self.serials.write_to(indent + 1, w)?;
        } else {
            self.write_indent(indent, w)?;
            writeln!(w, "serials: []")?;
        }

        if self.outputs.len() > 0 {
            self.write_indent(indent, w)?;
            writeln!(w, "outputs:")?;
            self.outputs.write_to(indent + 1, w)?;
        } else {
            self.write_indent(indent, w)?;
            writeln!(w, "outputs: []")?;
        }
        Ok(())
    }
}

pub fn read_config<E>(reader: &mut dyn Read<Error = E>) -> Config {
    let mut buffer = [0u8; 4096];
    let size = reader.read(&mut buffer).ok().unwrap_or(0);
    if size > 0 {
        Config::from_yaml(&mut YamlParser::from(&buffer[..size]))
    } else {
        Config::default()
    }
}

mod test {
    #[cfg(test)]
    extern crate std;

    #[test]
    #[serial]
    fn test_read_config() -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Read;
        use std::string::{String, ToString};

        use super::yaml::{FromYAML, ToYAML, YamlParser};
        use super::Config;

        let mut file = File::open("sample.yml")?;
        let mut yaml_string = String::new();
        file.read_to_string(&mut yaml_string)?;
        let config = Config::from_yaml(&mut YamlParser::from(yaml_string.as_ref() as &str));

        let mut buf = String::new();
        config.write_to(0, &mut buf).ok();
        assert_eq!(yaml_string.trim(), buf.to_string().trim());
        Ok(())
    }
}
