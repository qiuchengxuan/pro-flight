use core::fmt::Write;
use core::str::Split;

use fixed_point::FixedPoint;

use crate::datastructures::measurement::{Axes, Gain};

use super::setter::{Error, Setter, Value};
use super::yaml::ToYAML;

pub type Sensitive = FixedPoint<i32, 2>; // LSB/unit

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Accelerometer {
    pub bias: Axes,
    pub gain: Gain,
    pub sensitive: Sensitive,
}

impl Setter for Accelerometer {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "bias" => return self.bias.set(path, value),
            "gain" => return self.gain.set(path, value),
            "sensitive" => self.sensitive = value.parse()?.unwrap_or_default(),
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

impl ToYAML for Accelerometer {
    fn write_to(&self, indent: usize, w: &mut impl Write) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "bias:")?;
        self.bias.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "gain:")?;
        self.gain.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "sensitive: {}", self.sensitive)
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Magnetometer {
    pub bias: Axes,
    pub gain: Gain,
    pub declination: FixedPoint<i32, 1>,
}

impl Setter for Magnetometer {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "bias" => return self.bias.set(path, value),
            "gain" => return self.gain.set(path, value),
            "declination" => self.declination = value.parse()?.unwrap_or_default(),
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

impl ToYAML for Magnetometer {
    fn write_to(&self, indent: usize, w: &mut impl Write) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "bias:")?;
        self.bias.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "gain:")?;
        self.gain.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "declination: {}", self.declination)
    }
}

const DEFAULT_KP: FixedPoint<i32, 3> = FixedPoint(0_250);
const DEFAULT_KI: FixedPoint<i32, 3> = FixedPoint(0_005);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Mahony {
    pub kp: FixedPoint<i32, 3>,
    pub ki: FixedPoint<i32, 3>,
}

impl Default for Mahony {
    fn default() -> Self {
        Self { kp: DEFAULT_KP, ki: DEFAULT_KI }
    }
}

impl Setter for Mahony {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "kp" => self.kp = value.parse()?.unwrap_or(DEFAULT_KP),
            "ki" => self.ki = value.parse()?.unwrap_or(DEFAULT_KI),
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

impl ToYAML for Mahony {
    fn write_to(&self, indent: usize, w: &mut impl Write) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "kp: {}", self.kp)?;

        self.write_indent(indent, w)?;
        writeln!(w, "ki: {}", self.ki)
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct IMU {
    pub accelerometer: Accelerometer,
    pub magnetometer: Magnetometer,
    pub mahony: Mahony,
}

impl Setter for IMU {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "accelerometer" => self.accelerometer.set(path, value),
            "magnetometer" => self.magnetometer.set(path, value),
            "mahony" => self.mahony.set(path, value),
            _ => return Err(Error::MalformedPath),
        }
    }
}

impl ToYAML for IMU {
    fn write_to(&self, indent: usize, w: &mut impl Write) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "accelerometer:")?;
        self.accelerometer.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "magnetometer:")?;
        self.magnetometer.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "mahony:")?;
        self.mahony.write_to(indent + 1, w)
    }
}
