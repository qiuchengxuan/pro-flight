pub mod aircraft;
pub mod battery;
pub mod board;
pub mod imu;
pub mod osd;
pub mod peripherals;
pub mod receiver;
pub mod setter;
pub mod yaml;

use core::mem;
use core::slice;
use core::str::Split;

use fixed_point::{fixed_point, FixedPoint};

use crate::datastructures::measurement::{Axes, Bias, Gain};
use crate::io::Read;
pub use aircraft::Aircraft;
pub use battery::Battery;
pub use board::Board;
pub use imu::IMU;
pub use osd::{Offset, Standard, OSD};
pub use peripherals::pwm::{PWMs, Protocol, PWM};
pub use peripherals::serial::{Config as SerialConfig, Serials};
pub use peripherals::Peripherals;
pub use receiver::Receiver;
use setter::{Error, Setter, Value};
use yaml::YamlParser;

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

impl Setter for Bias {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let key = path.next().ok_or(Error::MalformedPath)?;
        let value = value.parse()?.unwrap_or(fixed_point!(1.0, 5i32));
        match key {
            "x" => self.x = value,
            "y" => self.y = value,
            "z" => self.z = value,
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

impl Setter for Gain {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let key = path.next().ok_or(Error::MalformedPath)?;
        let value = value.parse()?.unwrap_or(fixed_point!(1.0, 4u16));
        match key {
            "x" => self.x = value,
            "y" => self.y = value,
            "z" => self.z = value,
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

const DEFAULT_KP: FixedPoint<u16, 3> = fixed_point!(0.25, 3u16);

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Speedometer {
    pub kp: FixedPoint<u16, 3>,
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

#[repr(C, align(4))]
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Config {
    #[serde(skip)]
    version: u8,
    pub aircraft: Aircraft,
    pub battery: Battery,
    pub board: Board,
    pub imu: IMU,
    pub osd: OSD,
    pub receiver: Receiver,
    pub speedometer: Speedometer,
    pub peripherals: Peripherals,
}

impl Setter for Config {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "aircraft" => self.aircraft.set(path, value),
            "battery" => self.battery.set(path, value),
            "board" => self.board.set(path, value),
            "imu" => self.imu.set(path, value),
            "osd" => self.osd.set(path, value),
            "receiver" => self.receiver.set(path, value),
            "speedometer" => self.speedometer.set(path, value),
            "peripherals" => self.peripherals.set(path, value),
            _ => Err(Error::MalformedPath),
        }
    }
}

impl core::fmt::Display for Config {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        serde_yaml_core::to_fmt(f, self)
    }
}

impl From<&[u32]> for Config {
    fn from(words: &[u32]) -> Self {
        let config: &Config = unsafe { mem::transmute(words.as_ptr() as *const Self) };
        config.clone()
    }
}

impl AsRef<[u32]> for Config {
    fn as_ref(&self) -> &[u32] {
        let length = mem::size_of::<Self>() / mem::size_of::<u32>();
        unsafe { slice::from_raw_parts(self as *const _ as *const u32, length) }
    }
}

static mut CONFIG: Option<Config> = None;

#[inline]
pub fn get() -> &'static Config {
    unsafe { CONFIG.as_ref().unwrap() }
}

static mut CONFIG_ITERATION: usize = 0;

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

pub fn replace(config: &Config) {
    unsafe { CONFIG_ITERATION += 1 }
    unsafe { CONFIG = Some(config.clone()) }
}

pub fn iteration() -> usize {
    unsafe { CONFIG_ITERATION }
}

#[cfg(feature = "default-config")]
extern "Rust" {
    fn default_config() -> Config;
}

pub fn reset() {
    let config = match () {
        #[cfg(feature = "default-config")]
        () => unsafe { default_config() },
        #[cfg(not(feature = "default-config"))]
        () => Config::default(),
    };
    replace(&config);
}

mod test {
    #[cfg(test)]
    extern crate std;

    #[test]
    #[serial]
    fn test_init_config() -> std::io::Result<()> {
        use std::fmt::Write;
        use std::fs::File;
        use std::io::Read;
        use std::string::{String, ToString};

        use super::yaml::YamlParser;
        use super::Config;

        let mut file = File::open("sample.yml")?;
        let mut yaml_string = String::new();
        file.read_to_string(&mut yaml_string)?;
        let config: Config = YamlParser::new(yaml_string.as_str()).parse();

        let mut buf = String::new();
        writeln!(buf, "{}", config).ok();
        assert_eq!(yaml_string.trim(), buf.to_string().trim());
        Ok(())
    }

    #[test]
    #[serial]
    fn test_binary_decode() -> std::io::Result<()> {
        use std::fmt::Write;
        use std::fs::File;
        use std::io::Read;
        use std::mem;
        use std::string::String;

        use super::yaml::YamlParser;
        use super::Config;

        let mut file = File::open("sample.yml")?;
        let mut yaml_string = String::new();
        file.read_to_string(&mut yaml_string)?;
        let config: Config = YamlParser::new(yaml_string.as_str()).parse();
        let bytes: &[u8; mem::size_of::<Config>()] = unsafe { mem::transmute(&config) };
        let mut new_bytes = [0u8; mem::size_of::<Config>()];
        new_bytes[..].copy_from_slice(bytes);
        drop(config);
        let config: &Config = unsafe { mem::transmute(&new_bytes) };

        let mut buf = String::new();
        writeln!(buf, "{}", config).ok();
        assert_eq!(yaml_string.trim(), buf.to_string().trim());
        Ok(())
    }
}
