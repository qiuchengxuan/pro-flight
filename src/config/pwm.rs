use core::cmp::min;
use core::fmt::{Result, Write};

use ascii::AsciiStr;
use btoi::btoi;

use super::yaml::{ByteStream, Entry, FromYAML, ToYAML};

#[derive(PartialEq, Copy, Clone)]
pub struct MotorConfig {
    pub rate: u16,
}

impl Default for MotorConfig {
    fn default() -> Self {
        Self { rate: 400 }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub struct ServoConfig {
    pub reversed: bool,
}

impl Default for ServoConfig {
    fn default() -> Self {
        Self { reversed: false }
    }
}

#[derive(Copy, Clone)]
pub enum PWM {
    Motor(MotorConfig),
    Servo(ServoConfig),
}

impl Default for PWM {
    fn default() -> Self {
        Self::Servo(ServoConfig::default())
    }
}

impl Into<&str> for PWM {
    fn into(self) -> &'static str {
        match self {
            Self::Motor(_) => "motor",
            Self::Servo(_) => "servo",
        }
    }
}

impl PWM {
    pub fn rate(&self) -> u16 {
        match self {
            Self::Motor(motor) => motor.rate,
            _ => 50,
        }
    }
}

const MAX_PWM_NAME_LEN: usize = 4;

const MAX_PWM_CONFIGS: usize = 6;

#[derive(Default)]
pub struct PwmConfig([u8; MAX_PWM_NAME_LEN], PWM);

impl FromYAML for PwmConfig {
    fn from_yaml<'a>(&mut self, indent: usize, byte_stream: &'a mut ByteStream) {
        let mut name_bytes: &[u8] = &[];
        let mut type_string: &[u8] = &[];
        let mut rate = 400;
        let mut reversed = false;
        byte_stream.skip_once_list_mark();
        loop {
            match byte_stream.next(indent) {
                Some(Entry::KeyValue(key, value)) => match key {
                    b"name" => name_bytes = value,
                    b"type" => type_string = value,
                    b"rate" => rate = btoi(value).ok().unwrap_or(400),
                    b"reversed" => reversed = value == b"true",
                    _ => continue,
                },
                Some(Entry::Key(_)) => byte_stream.skip(indent),
                _ => break,
            }
        }
        let size = min(name_bytes.len(), MAX_PWM_NAME_LEN);
        self.0[..size].copy_from_slice(&name_bytes[..size]);
        self.1 = match type_string {
            b"motor" => PWM::Motor(MotorConfig { rate }),
            b"servo" => PWM::Servo(ServoConfig { reversed }),
            _ => PWM::default(),
        };
    }
}

impl ToYAML for PwmConfig {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        let size = self.0.iter().position(|&b| b == 0).unwrap_or(MAX_PWM_NAME_LEN);
        let string = unsafe { AsciiStr::from_ascii_unchecked(&self.0[..size]) };
        writeln!(w, "- name: {}", string)?;
        match self.1 {
            PWM::Motor(motor) => {
                self.write_indent(indent, w)?;
                writeln!(w, "type: motor")?;
                self.write_indent(indent, w)?;
                writeln!(w, "rate: {}", motor.rate)
            }
            PWM::Servo(servo) => {
                self.write_indent(indent, w)?;
                writeln!(w, "type: servo")?;
                self.write_indent(indent, w)?;
                writeln!(w, "reversed: {}", servo.reversed)
            }
        }
    }
}

#[derive(Default)]
pub struct PwmConfigs {
    configs: [PwmConfig; MAX_PWM_CONFIGS],
    num_config: usize,
}

impl PwmConfigs {
    pub fn get(&self, name: &[u8]) -> Option<PWM> {
        for config in self.configs[..self.num_config].iter() {
            let size = config.0.iter().position(|&b| b == 0).unwrap_or(MAX_PWM_NAME_LEN);
            if &config.0[..size] == name {
                return Some(config.1);
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.num_config
    }
}

impl FromYAML for PwmConfigs {
    fn from_yaml<'a>(&mut self, indent: usize, byte_stream: &'a mut ByteStream) {
        loop {
            match byte_stream.next(indent) {
                Some(Entry::ListEntry) => {
                    if self.num_config >= MAX_PWM_CONFIGS {
                        byte_stream.skip(indent);
                    }
                    self.configs[self.num_config].from_yaml(indent + 1, byte_stream);
                    self.num_config += 1;
                }
                _ => return,
            }
        }
    }
}

impl ToYAML for PwmConfigs {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        for config in self.configs[0..self.num_config].iter() {
            self.write_indent(indent, w)?;
            config.write_to(indent + 1, w)?;
        }
        Ok(())
    }
}
