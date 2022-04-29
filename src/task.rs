#[derive(Copy, Clone, Debug)]
pub enum Priority {
    Immediate = 0,
    System = 1,
    Sensor = 2,
    Normal = 3,
    Telemetry = 4,
    Interactive = 5,
}

impl Into<u8> for Priority {
    fn into(self) -> u8 {
        self as u8
    }
}
