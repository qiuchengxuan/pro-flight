use core::fmt::{Display, Formatter, Write};
use core::str::Split;

use crate::datastructures::Ratio;

use super::setter::{Error, Setter, Value};
use super::yaml::ToYAML;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Standard {
    PAL,
    NTSC,
}

impl Default for Standard {
    fn default() -> Self {
        Self::PAL
    }
}

impl From<&str> for Standard {
    fn from(string: &str) -> Standard {
        match string {
            "NTSC" => Standard::NTSC,
            _ => Standard::PAL,
        }
    }
}

impl Display for Standard {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let string = match self {
            Self::PAL => "PAL",
            Self::NTSC => "NTSC",
        };
        f.write_str(string)
    }
}

impl Into<Ratio> for Standard {
    fn into(self) -> Ratio {
        match self {
            Self::PAL => Ratio(5, 4),
            Self::NTSC => Ratio(16, 9),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Offset {
    pub horizental: i8,
    pub vertical: i8,
}

impl Setter for Offset {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let value = value.parse()?.unwrap_or_default();
        match path.next().ok_or(Error::MalformedPath)? {
            "horizental" => self.horizental = value,
            "vertical" => self.vertical = value,
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

impl ToYAML for Offset {
    fn write_to(&self, indent: usize, w: &mut impl Write) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "horizental: {}", self.horizental)?;
        self.write_indent(indent, w)?;
        writeln!(w, "vertical: {}", self.vertical)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OSD {
    pub aspect_ratio: Ratio,
    pub fov: u8,
    pub offset: Offset,
    pub refresh_rate: u8,
    pub standard: Standard,
}

impl Default for OSD {
    fn default() -> Self {
        Self {
            aspect_ratio: Ratio::default(),
            fov: 120,
            offset: Offset::default(),
            refresh_rate: 50,
            standard: Standard::default(),
        }
    }
}

impl Setter for OSD {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "aspect-ratio" => self.aspect_ratio = value.parse()?.unwrap_or_default(),
            "fov" => self.fov = value.parse()?.unwrap_or_default(),
            "offset" => return self.offset.set(path, value),
            "refresh-rate" => self.refresh_rate = value.parse()?.unwrap_or_default(),
            "standard" => self.standard = value.0.map(|v| Standard::from(v)).unwrap_or_default(),
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

impl ToYAML for OSD {
    fn write_to(&self, indent: usize, w: &mut impl Write) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "aspect-ratio: '{}'", self.aspect_ratio)?;

        self.write_indent(indent, w)?;
        writeln!(w, "fov: {}", self.fov)?;

        self.write_indent(indent, w)?;
        writeln!(w, "offset:")?;
        self.offset.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "refresh-rate: {}", self.refresh_rate)?;

        self.write_indent(indent, w)?;
        writeln!(w, "standard: {}", self.standard)
    }
}
