use core::fmt::{Display, Formatter, Result, Write};

use crate::alloc;

use super::yaml::{FromYAML, ToYAML, YamlParser};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AspectRatio(pub u8, pub u8);

impl Default for AspectRatio {
    fn default() -> Self {
        Self(16, 9)
    }
}

impl FromYAML for AspectRatio {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut width: u8 = 16;
        let mut height: u8 = 9;
        while let Some((key, value)) = parser.next_key_value() {
            let value = value.parse().ok();
            match key {
                "width" => width = value.unwrap_or(16),
                "height" => height = value.unwrap_or(9),
                _ => continue,
            }
        }
        Self(width, height)
    }
}

impl ToYAML for AspectRatio {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        writeln!(w, "width: {}", self.0)?;
        self.write_indent(indent, w)?;
        writeln!(w, "height: {}", self.1)?;
        Ok(())
    }
}

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
    fn fmt(&self, f: &mut Formatter) -> Result {
        let string = match self {
            Self::PAL => "PAL",
            Self::NTSC => "NTSC",
        };
        f.write_str(string)
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
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        writeln!(w, "horizental: {}", self.horizental)?;
        self.write_indent(indent, w)?;
        writeln!(w, "vertical: {}", self.vertical)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OSD {
    pub fov: u8,
    pub font: &'static str,
    pub aspect_ratio: AspectRatio,
    pub standard: Standard,
    pub offset: Offset,
}

impl Default for OSD {
    fn default() -> Self {
        Self {
            fov: 120,
            font: "",
            aspect_ratio: AspectRatio(16, 9),
            standard: Standard::default(),
            offset: Offset::default(),
        }
    }
}

impl FromYAML for OSD {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> OSD {
        let mut aspect_ratio = AspectRatio::default();
        let mut font = "";
        let mut fov = 120u8;
        let mut standard = Standard::default();
        let mut offset = Offset::default();
        while let Some(key) = parser.next_entry() {
            match key {
                "aspect-ratio" => aspect_ratio = AspectRatio::from_yaml(parser),
                "font" => {
                    if let Some(value) = parser.next_value() {
                        let length = value.as_bytes().len();
                        if let Some(bytes) = alloc::allocate(length, false) {
                            bytes.copy_from_slice(value.as_bytes());
                            font = unsafe { core::str::from_utf8_unchecked(bytes) };
                        }
                    }
                }
                "fov" => {
                    if let Some(value) = parser.next_value() {
                        fov = value.parse().unwrap_or(150);
                    }
                }
                "offset" => offset = Offset::from_yaml(parser),
                "standard" => {
                    if let Some(value) = parser.next_value() {
                        standard = Standard::from(value);
                    }
                }
                _ => parser.skip(),
            }
        }
        OSD { aspect_ratio, font, fov, standard, offset }
    }
}

impl ToYAML for OSD {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        writeln!(w, "aspect-ratio:")?;
        self.aspect_ratio.write_to(indent + 1, w)?;

        if self.font != "" {
            self.write_indent(indent, w)?;
            writeln!(w, "font: {}", self.font)?;
        }

        self.write_indent(indent, w)?;
        writeln!(w, "fov: {}", self.fov)?;

        self.write_indent(indent, w)?;
        writeln!(w, "offset:")?;
        self.offset.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "standard: {}", self.standard)
    }
}
