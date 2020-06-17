use btoi::btoi;

use super::yaml::{ByteIter, Entry, FromYAML};

#[derive(PartialEq, Copy, Clone)]
pub struct SbusConfig {
    pub fast: bool,
    pub rx_inverted: bool,
    pub half_duplex: bool,
}

impl SbusConfig {
    pub fn baudrate(&self) -> u32 {
        if !self.fast {
            100_000
        } else {
            200_000
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum SerialConfig {
    None,
    GNSS(u32),
    SBUS(SbusConfig),
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self::None
    }
}

const MAX_SERIAL_CONFIGS: usize = 5;

pub struct Serials {
    name_buffer: [u8; 6 * MAX_SERIAL_CONFIGS],
    configs: [(usize, SerialConfig); MAX_SERIAL_CONFIGS],
    num_config: u8,
}

impl Default for Serials {
    fn default() -> Self {
        Self {
            name_buffer: [0u8; 6 * MAX_SERIAL_CONFIGS],
            configs: [(0, SerialConfig::default()); MAX_SERIAL_CONFIGS],
            num_config: 0u8,
        }
    }
}

impl Serials {
    pub fn get(&self, name: &'static [u8]) -> Option<SerialConfig> {
        let mut index = 0;
        for i in 0..self.num_config as usize {
            let (length, config) = self.configs[i];
            if &self.name_buffer[index..index + length] == name {
                return Some(config);
            }
            index += length;
        }
        None
    }
}

fn to_serial_config<'a>(indent: usize, byte_iter: &'a mut ByteIter) -> SerialConfig {
    let mut type_string: &[u8] = &[];
    let mut baudrate = 0;
    let mut fast = false;
    let mut rx_inverted = false;
    let mut half_duplex = false;
    loop {
        match byte_iter.next(indent) {
            Entry::KeyValue(key, value) => match key {
                b"type" => type_string = value,
                b"baudrate" => baudrate = btoi(value).ok().unwrap_or(0),
                b"fast" => fast = value == b"true",
                b"rx-inverted" => rx_inverted = value == b"true",
                b"half-duplex" => half_duplex = value == b"true",
                _ => continue,
            },
            Entry::Key(_) => byte_iter.skip(indent),
            _ => break,
        }
    }
    match type_string {
        b"SBUS" => SerialConfig::SBUS(SbusConfig { fast, rx_inverted, half_duplex }),
        b"GNSS" => SerialConfig::GNSS(baudrate),
        _ => SerialConfig::None,
    }
}

impl FromYAML for Serials {
    fn from_yaml<'a>(&mut self, indent: usize, byte_iter: &'a mut ByteIter) {
        let mut index = 0;
        loop {
            match byte_iter.next(indent) {
                Entry::Key(key) => {
                    if self.num_config as usize >= MAX_SERIAL_CONFIGS {
                        byte_iter.skip(indent);
                    }
                    if key.len() > self.name_buffer.len() - index {
                        byte_iter.skip(indent);
                    }
                    let name = &mut self.name_buffer[index..index + key.len()];
                    name.copy_from_slice(key);
                    let config = to_serial_config(indent + 2, byte_iter);
                    self.configs[self.num_config as usize] = (key.len(), config);

                    index += key.len();
                    self.num_config += 1;
                }
                _ => return,
            }
        }
    }
}
