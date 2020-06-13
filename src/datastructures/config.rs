use log::Level;

use crate::hal::io::Read;
use crate::hal::sensors::Axes;

#[derive(Default, Serialize, Deserialize)]
pub struct Calibration {
    pub acceleration: Axes,
}

#[repr(usize)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Error = 1,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Trace
    }
}

impl Into<Level> for LogLevel {
    fn into(self) -> Level {
        match self {
            LogLevel::Error => Level::Error,
            LogLevel::Warn => Level::Warn,
            LogLevel::Info => Level::Info,
            LogLevel::Debug => Level::Debug,
            LogLevel::Trace => Level::Trace,
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub log_level: LogLevel,
    pub calibration: Calibration,
    pub fov: u8,
    pub aspect_ratio: (u8, u8),
}

pub fn read_config<E>(reader: &mut dyn Read<Error = E>) -> Option<Config> {
    let mut buffer = [0u8; 1024];
    let size = reader.read(&mut buffer).ok().unwrap_or(0);
    if size == 0 {
        return None;
    }
    serde_json::from_slice(&buffer[..size]).ok()
}

impl Config {
    pub fn write_config<F: Fn(&[u8])>(&self, writer: F) {
        if let Some(string) = serde_json::to_string_pretty(self).ok() {
            writer(string.as_bytes())
        }
    }
}
