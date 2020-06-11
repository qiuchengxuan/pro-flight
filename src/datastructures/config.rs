#[derive(Default, Serialize, Deserialize)]
pub struct XYZ {
    x: i16,
    y: i16,
    z: i16,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Calibration {
    pub acceleration: XYZ,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub calibration: Calibration,
}

pub fn read_config<F: Fn(&mut [u8]) -> usize>(reader: F) -> Option<Config> {
    let mut buffer = [0u8; 1024];
    let size = reader(&mut buffer);
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
