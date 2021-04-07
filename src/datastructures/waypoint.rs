use super::coordinate::Position;

#[derive(Copy, Clone, Debug, Serialize)]
pub struct Waypoint {
    pub name: &'static str,
    pub position: Position,
}

impl Default for Waypoint {
    fn default() -> Self {
        Self { name: "HOME", position: Default::default() }
    }
}

#[derive(Copy, Clone, Debug, Default, Serialize)]
pub struct Steerpoint {
    pub index: u8,
    pub waypoint: Waypoint,
}
