use core::{
    fmt::{Display, Formatter},
    str::Split,
    time::Duration,
};

use crate::types::Ratio;

use super::setter::{Error, Setter, Value};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Standard {
    PAL,
    NTSC,
}

impl Standard {
    pub fn refresh_interval(self) -> Duration {
        match self {
            Self::PAL => Duration::from_millis(20),
            Self::NTSC => Duration::from_millis(16),
        }
    }
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

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
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
