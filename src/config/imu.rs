use core::fmt::{Result, Write};

use crate::datastructures::decimal::IntegerDecimal;
use crate::datastructures::measurement::Axes;

use super::yaml::{FromYAML, ToYAML, YamlParser};

#[derive(Default, Debug, Copy, Clone)]
pub struct Accelerometer {
    pub bias: Axes,
    pub gain: Axes,
}

impl FromYAML for Accelerometer {
    fn from_yaml<'a>(parser: &mut YamlParser) -> Self {
        let mut bias = Axes::default();
        let mut gain = Axes::default();

        while let Some(key) = parser.next_entry() {
            match key {
                "bias" => bias = Axes::from_yaml(parser),
                "gain" => gain = Axes::from_yaml(parser),
                _ => continue,
            }
        }
        Self { bias, gain }
    }
}

impl ToYAML for Accelerometer {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        writeln!(w, "bias:")?;
        self.bias.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "gain:")?;
        self.gain.write_to(indent + 1, w)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Mahony {
    pub kp: IntegerDecimal<u8, u8>,
    pub ki: IntegerDecimal<u8, u8>,
}

impl Default for Mahony {
    fn default() -> Self {
        Self { kp: IntegerDecimal::from("0.5"), ki: IntegerDecimal::from("0.0") }
    }
}

impl FromYAML for Mahony {
    fn from_yaml<'a>(parser: &mut YamlParser) -> Self {
        let mut kp = IntegerDecimal::from("0.5");
        let mut ki = IntegerDecimal::from("0.0");

        while let Some((key, value)) = parser.next_key_value() {
            match key {
                "kp" => kp = IntegerDecimal::from(value),
                "ki" => ki = IntegerDecimal::from(value),
                _ => continue,
            }
        }
        Self { ki, kp }
    }
}

impl ToYAML for Mahony {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        writeln!(w, "kp: {}", self.kp)?;

        self.write_indent(indent, w)?;
        writeln!(w, "ki: {}", self.ki)
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct IMU {
    pub accelerometer: Accelerometer,
    pub mahony: Mahony,
}

impl FromYAML for IMU {
    fn from_yaml<'a>(parser: &mut YamlParser) -> Self {
        let mut accelerometer = Accelerometer::default();
        let mut mahony = Mahony::default();

        while let Some(key) = parser.next_entry() {
            match key {
                "accelerometer" => accelerometer = Accelerometer::from_yaml(parser),
                "mahony" => mahony = Mahony::from_yaml(parser),
                _ => continue,
            }
        }
        Self { accelerometer, mahony }
    }
}

impl ToYAML for IMU {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        writeln!(w, "accelerometer:")?;
        self.accelerometer.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "mahony:")?;
        self.mahony.write_to(indent + 1, w)
    }
}
