pub mod battery;
pub mod osd;
pub mod sensor;
pub mod serial;
pub mod yaml;

use core::fmt::{Result, Write};

use btoi::btoi;

use crate::hal::io::Read;
use crate::hal::sensors::Axes;

pub use battery::Battery;
pub use osd::{AspectRatio, Offset, Standard, OSD};
pub use sensor::Accelerometer;
pub use serial::{SerialConfig, Serials};
use yaml::{ByteStream, Entry, FromYAML, ToYAML};

impl FromYAML for Axes {
    fn from_yaml<'a>(&mut self, indent: usize, byte_iter: &mut ByteStream<'a>) {
        for _ in 0..3 {
            match byte_iter.next(indent) {
                Entry::KeyValue(key, value) => {
                    let v = match btoi(value) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    match key {
                        b"x" => self.x = v,
                        b"y" => self.y = v,
                        b"z" => self.z = v,
                        _ => continue,
                    }
                }
                _ => return,
            }
        }
    }
}

impl ToYAML for Axes {
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
    pub osd: OSD,
    pub serials: Serials,
}

impl FromYAML for Config {
    fn from_yaml<'a>(&mut self, indent: usize, byte_iter: &mut ByteStream<'a>) {
        loop {
            match byte_iter.next(indent) {
                Entry::Key(key) => match key {
                    b"accelerometer" => self.accelerometer.from_yaml(indent + 1, byte_iter),
                    b"battery" => self.battery.from_yaml(indent + 1, byte_iter),
                    b"osd" => self.osd.from_yaml(indent + 1, byte_iter),
                    b"serials" => self.serials.from_yaml(indent + 1, byte_iter),
                    _ => byte_iter.skip(indent),
                },
                _ => return,
            }
        }
    }
}

impl ToYAML for Config {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        writeln!(w, "accelerometer:")?;
        self.accelerometer.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "battery:")?;
        self.battery.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "osd:")?;
        self.osd.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "serials:")?;
        self.serials.write_to(indent + 1, w)
    }
}

pub fn read_config<E>(reader: &mut dyn Read<Error = E>) -> Config {
    let mut buffer = [0u8; 4096];
    let size = reader.read(&mut buffer).ok().unwrap_or(0);
    let mut config = Config::default();
    if size > 0 {
        config.from_yaml(0, &mut ByteStream::from(&buffer[..]));
    }
    config
}

mod test {
    #[cfg(test)]
    extern crate std;

    #[test]
    fn test_read_config() -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Read;
        use std::string::{String, ToString};

        use super::yaml::{ByteStream, FromYAML, ToYAML};
        use super::{AspectRatio, Config, Offset, Standard, OSD};

        use crate::hal::sensors::Axes;

        let mut file = File::open("sample.yml")?;
        let mut yaml_string = String::new();
        file.read_to_string(&mut yaml_string)?;
        let mut config = Config::default();
        config.from_yaml(0, &mut ByteStream::from(yaml_string.as_bytes()));
        assert_eq!(config.accelerometer.bias, Axes { x: 83, y: -1, z: 99 });
        let osd = OSD {
            fov: 145,
            aspect_ratio: AspectRatio(16, 9),
            standard: Standard::PAL,
            offset: Offset { horizental: 8, vertical: 0 },
        };
        assert_eq!(config.osd, osd);

        let mut buf = String::new();
        config.write_to(0, &mut buf).ok();
        assert_eq!(yaml_string.trim(), buf.to_string().trim());
        Ok(())
    }
}
