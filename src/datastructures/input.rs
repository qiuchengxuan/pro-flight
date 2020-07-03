#[derive(Copy, Clone, PartialEq)]
pub enum InputType {
    Throttle = 0,
    Roll,
    Pitch,
    Yaw,
}

impl InputType {
    pub fn from_str(string: &str) -> Option<Self> {
        match string {
            "throttle" => Some(Self::Throttle),
            "roll" => Some(Self::Roll),
            "pitch" => Some(Self::Pitch),
            "yaw" => Some(Self::Yaw),
            _ => None,
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

pub type Throttle = u16;
pub type Roll = i16;
pub type Pitch = i16;
pub type Yaw = i16;
pub enum Flaps {
    Auto,
    Half,
    Full,
}
pub enum LandingGear {
    Up,
    Down,
}
