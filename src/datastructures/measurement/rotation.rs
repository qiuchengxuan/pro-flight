use core::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq)]
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

impl core::fmt::Display for Rotation {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let s = match self {
            Self::NoRotation => "0",
            Self::Degree90 => "90",
            Self::Degree180 => "180",
            Self::Degree270 => "270",
        };
        write!(f, "{}", s)
    }
}
