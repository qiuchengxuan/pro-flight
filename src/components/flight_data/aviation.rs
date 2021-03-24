use crate::datastructures::measurement::euler::Euler;
use crate::datastructures::measurement::Altitude;

#[derive(Debug, Default, Copy, Clone)]
pub struct Attitude {
    pub roll: i16,
    pub pitch: i16,
}

impl From<Euler> for Attitude {
    fn from(euler: Euler) -> Self {
        let roll = (-euler.roll * 10.0) as i16;
        let pitch = (-euler.pitch * 10.0) as i16;
        Self { roll, pitch }
    }
}

impl sval::value::Value for Attitude {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.map_begin(Some(2))?;
        stream.map_key("roll")?;
        stream.map_value(self.roll / 10)?;
        stream.map_key("pitch")?;
        stream.map_value(self.pitch / 10)?;
        stream.map_end()
    }
}

#[derive(Copy, Clone, Default, Debug, Value)]
pub struct Aviation {
    pub altitude: Altitude,
    pub attitude: Attitude,
    pub heading: u16,
    pub height: Altitude,
    pub g_force: u8,
    pub airspeed: u16,
    pub vario: i16,
}
