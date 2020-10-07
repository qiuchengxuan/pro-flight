pub mod aircraft;
pub mod battery;
pub mod imu;
pub mod osd;
pub mod output;
pub mod receiver;
pub mod serial;
pub mod setter;
pub mod yaml;

use core::fmt::Write;
use core::str::Split;

use crate::datastructures::decimal::IntegerDecimal;
use crate::datastructures::measurement::Axes;
use crate::hal::io::Read;

pub use aircraft::Aircraft;
pub use battery::Battery;
pub use imu::IMU;
pub use osd::{Offset, Standard, OSD};
pub use output::{Output, Outputs, Protocol};
pub use receiver::Receiver;
pub use serial::{Config as SerialConfig, Serials};
use setter::{Error, Setter, Value};
use yaml::{ToYAML, YamlParser};

impl Setter for Axes {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let key = path.next().ok_or(Error::MalformedPath)?;
        let value = value.parse()?.unwrap_or_default();
        match key {
            "x" => self.x = value,
            "y" => self.y = value,
            "z" => self.z = value,
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

impl ToYAML for Axes {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "x: {}", self.x)?;
        self.write_indent(indent, w)?;
        writeln!(w, "y: {}", self.y)?;
        self.write_indent(indent, w)?;
        writeln!(w, "z: {}", self.z)
    }
}

const DEFAULT_KP: IntegerDecimal = integer_decimal!(0_25, 2);

#[derive(Clone)]
pub struct Speedometer {
    pub kp: IntegerDecimal,
}

impl Default for Speedometer {
    fn default() -> Self {
        Self { kp: DEFAULT_KP }
    }
}

impl Setter for Speedometer {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "kp" => self.kp = value.parse()?.unwrap_or(DEFAULT_KP),
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

impl ToYAML for Speedometer {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "kp: {}", self.kp)
    }
}

#[derive(Default, Clone)]
pub struct Config {
    pub aircraft: Aircraft,
    pub battery: Battery,
    pub imu: IMU,
    pub osd: OSD,
    pub receiver: Receiver,
    pub serials: Serials,
    pub speedometer: Speedometer,
    pub outputs: Outputs,
}

impl Setter for Config {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "aircraft" => self.aircraft.set(path, value),
            "battery" => self.battery.set(path, value),
            "imu" => self.imu.set(path, value),
            "osd" => self.osd.set(path, value),
            "receiver" => self.receiver.set(path, value),
            "serials" => self.serials.set(path, value),
            "speedometer" => self.speedometer.set(path, value),
            "outputs" => self.outputs.set(path, value),
            _ => Err(Error::MalformedPath),
        }
    }
}

impl ToYAML for Config {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "aircraft:")?;
        self.aircraft.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "battery:")?;
        self.battery.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "imu:")?;
        self.imu.write_to(indent + 1, w)?;

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

        self.write_indent(indent, w)?;
        writeln!(w, "speedometer:")?;
        self.speedometer.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        if self.outputs.len() > 0 {
            writeln!(w, "outputs:")?;
            self.outputs.write_to(indent + 1, w)?;
        } else {
            writeln!(w, "outputs: null")?;
        }
        Ok(())
    }
}

impl core::fmt::Display for Config {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.write_to(0, f)
    }
}

static mut CONFIG: Option<Config> = None;

#[inline]
pub fn get() -> &'static Config {
    unsafe { CONFIG.as_ref().unwrap() }
}

pub fn load<E>(reader: &mut dyn Read<Error = E>) -> &'static Config {
    let mut buffer = [0u8; 2048];
    let size = reader.read(&mut buffer).ok().unwrap_or(0);
    let config = if size > 0 {
        YamlParser::new(unsafe { core::str::from_utf8_unchecked(&buffer[..size]) }).parse()
    } else {
        Config::default()
    };
    unsafe { CONFIG = Some(config) }
    get()
}

pub fn replace(config: Config) {
    unsafe { CONFIG = Some(config) }
}

mod test {
    #[cfg(test)]
    extern crate std;

    #[test]
    #[serial]
    fn test_init_config() -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Read;
        use std::string::{String, ToString};

        use super::yaml::{ToYAML, YamlParser};
        use super::Config;

        let mut file = File::open("sample.yml")?;
        let mut yaml_string = String::new();
        file.read_to_string(&mut yaml_string)?;
        let config: Config = YamlParser::new(yaml_string.as_str()).parse();

        let mut buf = String::new();
        config.write_to(0, &mut buf).ok();
        assert_eq!(yaml_string.trim(), buf.to_string().trim());
        Ok(())
    }

    #[test]
    fn test_set_config() -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Read;
        use std::string::String;

        use super::output::{Output, Servo, ServoType};
        use super::setter::{Setter, Value};
        use super::yaml::YamlParser;
        use super::Config;

        let mut file = File::open("sample.yml")?;
        let mut yaml_string = String::new();
        file.read_to_string(&mut yaml_string)?;
        let mut config: Config = YamlParser::new(yaml_string.as_str()).parse();

        config.set(&mut "outputs.PWM5.min-angle".split('.'), Value::of("-80")).unwrap();
        let expected = Output::Servo(Servo::new(ServoType::AileronLeft, -80, 90, true));
        assert_eq!(config.outputs.get("PWM5").unwrap(), &expected);
        Ok(())
    }
}
