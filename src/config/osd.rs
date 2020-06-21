use core::fmt::{Display, Formatter, Result, Write};

use btoi::btoi;

use super::yaml::{ByteStream, Entry, FromYAML, ToYAML};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AspectRatio(pub u8, pub u8);

impl Default for AspectRatio {
    fn default() -> Self {
        Self(16, 9)
    }
}

impl FromYAML for AspectRatio {
    fn from_yaml<'a>(&mut self, indent: usize, byte_stream: &mut ByteStream<'a>) {
        loop {
            match byte_stream.next(indent) {
                Some(Entry::KeyValue(key, value)) => match key {
                    b"width" => self.0 = btoi(value).ok().unwrap_or(16),
                    b"height" => self.1 = btoi(value).ok().unwrap_or(9),
                    _ => continue,
                },
                _ => return,
            }
        }
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

impl From<&[u8]> for Standard {
    fn from(bytes: &[u8]) -> Standard {
        match bytes {
            b"NTSC" => Standard::NTSC,
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
    fn from_yaml<'a>(&mut self, indent: usize, byte_stream: &mut ByteStream<'a>) {
        for _ in 0..2 {
            match byte_stream.next(indent) {
                Some(Entry::KeyValue(key, value)) => match key {
                    b"horizental" => self.horizental = btoi(value).ok().unwrap_or(0),
                    b"vertical" => self.vertical = btoi(value).ok().unwrap_or(0),
                    _ => return,
                },
                _ => return,
            }
        }
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
    pub aspect_ratio: AspectRatio,
    pub standard: Standard,
    pub offset: Offset,
}

impl Default for OSD {
    fn default() -> Self {
        Self {
            fov: 120,
            aspect_ratio: AspectRatio(16, 9),
            standard: Standard::default(),
            offset: Offset::default(),
        }
    }
}

impl FromYAML for OSD {
    fn from_yaml<'a>(&mut self, indent: usize, byte_stream: &mut ByteStream<'a>) {
        loop {
            match byte_stream.next(indent) {
                Some(Entry::Key(key)) => match key {
                    b"offset" => self.offset.from_yaml(indent + 1, byte_stream),
                    b"aspect-ratio" => self.aspect_ratio.from_yaml(indent + 1, byte_stream),
                    _ => byte_stream.skip(indent),
                },
                Some(Entry::KeyValue(key, value)) => match key {
                    b"standard" => self.standard = Standard::from(value),
                    b"fov" => self.fov = btoi(value).unwrap_or(150),
                    _ => byte_stream.skip(indent),
                },
                _ => return,
            }
        }
    }
}

impl ToYAML for OSD {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        writeln!(w, "aspect-ratio:")?;
        self.aspect_ratio.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "fov: {}", self.fov)?;

        self.write_indent(indent, w)?;
        writeln!(w, "offset:")?;
        self.offset.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "standard: {}", self.standard)
    }
}

mod test {
    #[cfg(test)]
    extern crate std;

    #[test]
    fn test_write() -> core::fmt::Result {
        use std::string::String;
        use std::string::ToString;

        use super::OSD;
        use crate::config::yaml::ToYAML;

        let mut buf = String::new();
        let osd = OSD::default();
        osd.write_to(0, &mut buf)?;
        let expected = "\
        \naspect-ratio:\
        \n  width: 16\
        \n  height: 9\
        \nfov: 120\
        \noffset:\
        \n  horizental: 0\
        \n  vertical: 0\
        \nstandard: PAL";
        assert_eq!(expected.trim(), buf.to_string().trim());
        Ok(())
    }
}
