pub mod osd;
pub mod sensor;
pub mod yaml;

use btoi::btoi;

use crate::hal::io::Read;
use crate::hal::sensors::Axes;

pub use osd::{AspectRatio, Offset, Standard, OSD};
pub use sensor::Accelerometer;
use yaml::{ByteIter, Entry, FromYAML};

impl FromYAML for Axes {
    fn from_yaml<'a>(&mut self, indent: usize, byte_iter: &mut ByteIter<'a>) {
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
#[derive(Default)]
pub struct Config {
    pub accelerometer: Accelerometer,
    pub aspect_ratio: AspectRatio,
    pub osd: OSD,
}

impl FromYAML for Config {
    fn from_yaml<'a>(&mut self, indent: usize, byte_iter: &mut ByteIter<'a>) {
        for _ in 0..3 {
            match byte_iter.next(indent) {
                Entry::Key(key) => match key {
                    b"accelerometer" => self.accelerometer.from_yaml(indent + 2, byte_iter),
                    b"aspect-ratio" => self.aspect_ratio.from_yaml(indent + 2, byte_iter),
                    b"osd" => self.osd.from_yaml(indent + 2, byte_iter),
                    _ => return,
                },
                _ => return,
            }
        }
    }
}

pub fn read_config<E>(reader: &mut dyn Read<Error = E>) -> Config {
    let mut buffer = [0u8; 1024];
    let size = reader.read(&mut buffer).ok().unwrap_or(0);
    let mut config = Config::default();
    if size > 0 {
        config.from_yaml(0, &mut ByteIter::from(&buffer[..]));
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

        use super::yaml::{ByteIter, FromYAML};
        use super::{AspectRatio, Config, Offset, Standard, OSD};

        use crate::hal::sensors::Axes;

        let mut file = File::open("sample.yml")?;
        let mut buffer = [0; 1000];
        let size = file.read(&mut buffer)?;
        let mut config = Config::default();
        config.from_yaml(0, &mut ByteIter::from(&buffer[..size]));
        assert_eq!(config.accelerometer.bias, Axes { x: 83, y: -1, z: 99 });
        let osd = OSD {
            fov: 150,
            aspect_ratio: AspectRatio(16, 9),
            standard: Standard::PAL,
            offset: Offset { horizental: 8, vertical: 0 },
        };
        assert_eq!(config.osd, osd);
        Ok(())
    }
}
