use core::str::FromStr;

use heapless::Vec;

use crate::config::inputs::command;

#[derive(Copy, Clone, Eq, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum AxisType {
    Throttle = 0,
    Roll,
    Pitch,
    Yaw,
}

impl FromStr for AxisType {
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

pub type RSSI = u16;

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Axes {
    pub throttle: u16,
    pub roll: i16,
    pub pitch: i16,
    pub yaw: i16,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Control {
    pub rssi: RSSI,
    pub axes: Axes,
    pub commands: Vec<command::Id, 8>,
}
