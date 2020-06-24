#[derive(Copy, Clone, PartialEq)]
pub enum InputType {
    Throttle,
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

#[derive(Copy, Clone, PartialEq)]
pub enum Input {
    Throttle(u16),
    Roll(i16),
    Pitch(i16),
    Yaw(i16),
}
