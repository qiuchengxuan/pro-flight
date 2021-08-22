use core::str::FromStr;

pub type RSSI = u16;

#[derive(Copy, Clone, Eq, Debug, PartialEq)]
#[repr(u8)]
pub enum InputType {
    Throttle = 0,
    Roll,
    Pitch,
    Yaw,
}

impl FromStr for InputType {
    type Err = ();
    fn from_str(string: &str) -> Result<Self, ()> {
        match string {
            "throttle" => Ok(Self::Throttle),
            "roll" => Ok(Self::Roll),
            "pitch" => Ok(Self::Pitch),
            "yaw" => Ok(Self::Yaw),
            _ => Err(()),
        }
    }
}

impl Into<&str> for InputType {
    fn into(self) -> &'static str {
        match self {
            Self::Throttle => "throttle",
            Self::Roll => "roll",
            Self::Pitch => "pitch",
            Self::Yaw => "yaw",
        }
    }
}

impl core::fmt::Display for InputType {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let s: &str = (*self).into();
        write!(f, "{}", s)
    }
}

#[derive(Copy, Clone, Debug, Default, Serialize)]
pub struct ControlInput {
    pub throttle: u16,
    pub roll: i16,
    pub pitch: i16,
    pub yaw: i16,
}
