use core::fmt::{Display, Formatter};

use fugit::NanosDurationU64 as Duration;

use super::pathset::{Error, Path, PathSet, Value};
use crate::types::Ratio;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Standard {
    PAL,
    NTSC,
}

impl Standard {
    pub fn refresh_interval(self) -> Duration {
        match self {
            Self::PAL => Duration::millis(20),
            Self::NTSC => Duration::millis(16),
        }
    }
}

impl Default for Standard {
    fn default() -> Self {
        Self::PAL
    }
}

impl core::str::FromStr for Standard {
    type Err = ();

    fn from_str(string: &str) -> Result<Standard, Self::Err> {
        match string {
            "NTSC" => Ok(Standard::NTSC),
            "PAL" => Ok(Standard::PAL),
            _ => Err(()),
        }
    }
}

impl Display for Standard {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        f.write_str(match self {
            Self::PAL => "PAL",
            Self::NTSC => "NTSC",
        })
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

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Offset {
    pub horizental: i8,
    pub vertical: i8,
}

impl PathSet for Offset {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        let value = value.parse()?;
        match path.str()? {
            "horizental" => self.horizental = value,
            "vertical" => self.vertical = value,
            _ => return Err(Error::UnknownPath),
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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

impl PathSet for OSD {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "aspect-ratio" => self.aspect_ratio = value.parse()?,
            "fov" => self.fov = value.parse()?,
            "offset" => return self.offset.set(path, value),
            "refresh-rate" => self.refresh_rate = value.parse()?,
            "standard" => self.standard = value.parse()?,
            _ => return Err(Error::UnknownPath),
        }
        Ok(())
    }
}
