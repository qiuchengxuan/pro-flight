use super::coordinate::Position;

#[derive(Copy, Clone, Value)]
pub struct Waypoint {
    pub name: &'static str,
    pub position: Position,
}

impl Default for Waypoint {
    fn default() -> Self {
        Self { name: "HOME", position: Default::default() }
    }
}

#[derive(Copy, Clone, Default, Value)]
pub struct Steerpoint {
    pub index: u8,
    pub waypoint: Waypoint,
}
