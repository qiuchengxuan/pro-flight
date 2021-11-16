use core::{fmt::Write, str::FromStr};

pub mod control;
pub mod coordinate;
pub mod flight;
#[macro_use]
pub mod measurement;
pub mod output;
pub mod vec;
pub mod waypoint;

pub type RSSI = u16;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ratio(pub u8, pub u8);

impl FromStr for Ratio {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, ()> {
        let mut splitted = string.split(':');
        let ratio_0 = splitted.next().ok_or(())?.parse().map_err(|_| ())?;
        let ratio_1 = splitted.next().ok_or(())?.parse().map_err(|_| ())?;
        Ok(Self(ratio_0, ratio_1))
    }
}

impl Default for Ratio {
    fn default() -> Self {
        Self(1, 1)
    }
}

impl serde::Serialize for Ratio {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut string = heapless::String::<7>::new();
        write!(string, "{}:{}", self.0, self.1).ok();
        serializer.serialize_str(string.as_str())
    }
}
