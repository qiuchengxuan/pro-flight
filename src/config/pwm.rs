use core::fmt::{Result, Write};

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

fn pwm_name_to_index(bytes: &[u8]) -> Option<usize> {
    if bytes.starts_with(b"PWM") {
        btoi(&bytes[3..]).ok().map(|x: usize| x - 1)
    } else {
        None
    }
}

const MAX_PWM_CONFIGS: usize = 6;

impl FromYAML for PWM {
    fn from_yaml<'a>(&mut self, indent: usize, byte_stream: &'a mut ByteStream) {
        let mut type_string: &[u8] = &[];
        let mut rate = 400;
        let mut reversed = false;
        loop {
            match byte_stream.next(indent) {
                Some(Entry::KeyValue(key, value)) => match key {
                    b"type" => type_string = value,
                    b"rate" => rate = btoi(value).ok().unwrap_or(400),
                    b"reversed" => reversed = value == b"true",
                    _ => continue,
                },
                Some(Entry::Key(_)) => byte_stream.skip(indent),
                _ => break,
            }
        }
        *self = match type_string {
            b"motor" => PWM::Motor(MotorConfig { rate }),
            b"servo" => PWM::Servo(ServoConfig { reversed }),
            _ => PWM::default(),
        };
    }
}

impl ToYAML for PWM {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        match self {
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
pub struct PwmConfigs([Option<PWM>; MAX_PWM_CONFIGS]);

impl PwmConfigs {
    pub fn get(&self, name: &[u8]) -> Option<PWM> {
        if let Some(index) = pwm_name_to_index(name) {
            if index < self.0.len() {
                return self.0[index];
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.0.iter().filter(|&b| b.is_some()).count()
    }
}

impl FromYAML for PwmConfigs {
    fn from_yaml<'a>(&mut self, indent: usize, byte_stream: &'a mut ByteStream) {
        loop {
            match byte_stream.next(indent) {
                Some(Entry::Key(key)) => {
                    let index = match pwm_name_to_index(key) {
                        Some(index) => index,
                        None => {
                            byte_stream.skip(indent);
                            continue;
                        }
                    };
                    if index >= self.0.len() {
                        byte_stream.skip(indent);
                        continue;
                    }
                    let mut pwm = PWM::default();
                    pwm.from_yaml(indent + 1, byte_stream);
                    self.0[index] = Some(pwm);
                }
                _ => return,
            }
        }
    }
}

impl ToYAML for PwmConfigs {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        for i in 0..self.0.len() {
            if let Some(config) = self.0[i] {
                self.write_indent(indent, w)?;
                writeln!(w, "PWM{}:", i + 1)?;
                config.write_to(indent + 1, w)?;
            }
        }
        Ok(())
    }
}
