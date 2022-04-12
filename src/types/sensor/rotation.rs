use core::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum Rotation {
    NoRotation,
    Degree90,
    Degree180,
    Degree270,
}

impl Default for Rotation {
    fn default() -> Self {
        Self::NoRotation
    }
}

impl FromStr for Rotation {
    type Err = ();

    fn from_str(name: &str) -> Result<Rotation, ()> {
        match name {
            "0" => Ok(Rotation::NoRotation),
            "90" => Ok(Rotation::Degree90),
            "180" => Ok(Rotation::Degree180),
            "270" => Ok(Rotation::Degree270),
            _ => Err(()),
        }
    }
}

impl serde::Serialize for Rotation {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = match self {
            Self::NoRotation => "0",
            Self::Degree90 => "90",
            Self::Degree180 => "180",
            Self::Degree270 => "270",
        };
        serializer.serialize_str(s)
    }
}
