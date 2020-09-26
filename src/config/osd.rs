use alloc::rc::Rc;
use alloc::string::String;
use core::fmt::{Display, Formatter, Write};
use core::str::Split;

use crate::datastructures::Ratio;

use super::setter::{SetError, Setter};
use super::yaml::{FromYAML, ToYAML, YamlParser};

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

impl FromYAML for Offset {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut horizental: i8 = 0;
        let mut vertical: i8 = 0;
        while let Some((key, value)) = parser.next_key_value() {
            let value = value.parse().ok().unwrap_or(0);
            match key {
                "horizental" => horizental = value,
                "vertical" => vertical = value,
                _ => continue,
            };
        }
        Self { horizental, vertical }
    }
}

impl ToYAML for Offset {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "horizental: {}", self.horizental)?;
        self.write_indent(indent, w)?;
        writeln!(w, "vertical: {}", self.vertical)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OSD {
    pub aspect_ratio: Ratio,
    pub font: Rc<String>,
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
            font: Rc::new(String::default()),
            offset: Offset::default(),
            refresh_rate: 50,
            standard: Standard::default(),
        }
    }
}

impl FromYAML for OSD {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> OSD {
        let mut aspect_ratio = Ratio::default();
        let mut font = Rc::new(String::default());
        let mut fov = 120u8;
        let mut offset = Offset::default();
        let mut refresh_rate = 50;
        let mut standard = Standard::default();
        while let Some(key) = parser.next_entry() {
            match key {
                "aspect-ratio" => {
                    if let Some(value) = parser.next_value() {
                        if let Some(ratio) = Ratio::from_str(value) {
                            aspect_ratio = ratio;
                        }
                    }
                }
                "font" => {
                    if let Some(value) = parser.next_value() {
                        font = Rc::new(String::from(value));
                    }
                }
                "fov" => {
                    if let Some(value) = parser.next_value() {
                        fov = value.parse().unwrap_or(150);
                    }
                }
                "offset" => offset = Offset::from_yaml(parser),
                "refresh-rate" => {
                    if let Some(value) = parser.next_value() {
                        refresh_rate = value.parse().unwrap_or(50);
                    }
                }
                "standard" => {
                    if let Some(value) = parser.next_value() {
                        standard = Standard::from(value);
                    }
                }
                _ => parser.skip(),
            }
        }
        OSD { aspect_ratio, font, fov, offset, refresh_rate, standard }
    }
}

impl ToYAML for OSD {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "aspect-ratio: '{}'", self.aspect_ratio)?;

        if self.font.as_str() != "" {
            self.write_indent(indent, w)?;
            writeln!(w, "font: {}", self.font.as_str())?;
        }

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

impl Setter for OSD {
    fn set(&mut self, path: &mut Split<char>, value: Option<&str>) -> Result<(), SetError> {
        if path.next() == Some("refresh-rate") {
            let value = match value.map(|v| v.parse::<u8>().ok()).flatten() {
                Some(value) => value,
                None => return Err(SetError::UnexpectedValue),
            };
            self.refresh_rate = value;
            return Ok(());
        }
        Err(SetError::MalformedPath)
    }
}
