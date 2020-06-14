use btoi::btoi;

use super::yaml::{ByteIter, Entry, FromYAML};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AspectRatio(pub u8, pub u8);

impl Default for AspectRatio {
    fn default() -> Self {
        Self(16, 9)
    }
}

impl FromYAML for AspectRatio {
    fn from_yaml<'a>(&mut self, indent: usize, byte_iter: &mut ByteIter<'a>) {
        for _ in 0..2 {
            match byte_iter.next(indent) {
                Entry::KeyValue(key, value) => match key {
                    b"width" => self.0 = btoi(value).ok().unwrap_or(16),
                    b"height" => self.1 = btoi(value).ok().unwrap_or(9),
                    _ => return,
                },
                _ => return,
            }
        }
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

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Offset {
    pub horizental: i8,
    pub vertical: i8,
}

impl FromYAML for Offset {
    fn from_yaml<'a>(&mut self, indent: usize, byte_iter: &mut ByteIter<'a>) {
        for _ in 0..2 {
            match byte_iter.next(indent) {
                Entry::KeyValue(key, value) => match key {
                    b"horizental" => self.horizental = btoi(value).ok().unwrap_or(0),
                    b"vertial" => self.vertical = btoi(value).ok().unwrap_or(0),
                    _ => return,
                },
                _ => return,
            }
        }
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
    fn from_yaml<'a>(&mut self, indent: usize, byte_iter: &mut ByteIter<'a>) {
        for _ in 0..4 {
            match byte_iter.next(indent) {
                Entry::Key(key) => match key {
                    b"offset" => self.offset.from_yaml(indent + 2, byte_iter),
                    b"aspect-ratio" => self.aspect_ratio.from_yaml(indent + 2, byte_iter),
                    _ => return,
                },
                Entry::KeyValue(key, value) => match key {
                    b"standard" => self.standard = Standard::from(value),
                    b"fov" => self.fov = btoi(value).unwrap_or(150),
                    _ => return,
                },
                _ => return,
            }
        }
    }
}
