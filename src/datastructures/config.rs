use crate::hal::io::Read;
use crate::hal::sensors::Axes;

#[derive(Default, Serialize, Deserialize)]
pub struct Calibration {
    pub acceleration: Axes,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub calibration: Calibration,
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
