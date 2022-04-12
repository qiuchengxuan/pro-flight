use core::str::FromStr;

#[derive(Copy, Clone, Eq, Debug, PartialEq, Serialize)]
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

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Axes {
    pub throttle: u16,
    pub roll: i16,
    pub pitch: i16,
    pub yaw: i16,
}

pub type RSSI = u16;

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Control {
    pub rssi: RSSI,
    pub axes: Axes,
}
