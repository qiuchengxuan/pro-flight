#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    #[serde(rename = "nav-mode")]
    NAV,
    #[serde(rename = "telemetry-mode")]
    Telemetry,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Id {
    Mode(Mode),
}

impl core::str::FromStr for Id {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = match s {
            "nav-mode" => Self::Mode(Mode::NAV),
            "telemetry-mode" => Self::Mode(Mode::Telemetry),
            _ => return Err(()),
        };
        Ok(id)
    }
}
